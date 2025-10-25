/*
  This file returns a mocked version of Nixpkgs' default.nix for testing purposes.
  It does not depend on Nixpkgs itself for the sake of simplicity.

  It takes one attribute as an argument:
  - `root`: The root of Nixpkgs to read other files from, including:
    - `./pkgs/by-name`: The `pkgs/by-name` directory to test
    - `./all-packages.nix`: A file containing an overlay to mirror the real `pkgs/top-level/all-packages.nix`.
      This allows adding overrides on top of the auto-called packages in `pkgs/by-name`.

  It returns a Nixpkgs-like function that can be auto-called and evaluates to an attribute set.
*/
{
  root,
}:
# The arguments for the Nixpkgs function
{
  # Passed by the checker to modify `callPackage`
  overlays ? [ ],
  # Passed by the checker to make sure a real Nixpkgs isn't influenced by impurities
  config ? { },
  byNameConfig,
  ...
}:
let

  # Simplified versions of lib functions
  lib = import <test-nixpkgs/lib>;

  # The base fixed-point function to populate the resulting attribute set
  pkgsFun =
    self:
    {
      inherit lib;
      newScope = extra: lib.callPackageWith (self // extra);
      callPackage = self.newScope { };
      callPackages = lib.callPackagesWith self;
    }
    # This mapAttrs is a very hacky workaround necessary because for all attributes defined in Nixpkgs,
    # the files that define them are verified to be within Nixpkgs.
    # This is usually a very safe assumption, but it fails in these tests for someDrv,
    # because it's technically defined outside the Nixpkgs directories of each test case.
    # By using `mapAttrs`, `builtins.unsafeGetAttrPos` just returns `null`,
    # which then doesn't trigger this check
    // lib.mapAttrs (name: value: value) {
      someDrv = {
        type = "derivation";
      };
    };

  byNameDirs =
    let
      mapped = map (dir: {
        inherit dir;
        path = lib.concatStrings [
          (builtins.toString root)
          "/"
          dir.path
        ];
      }) byNameConfig.by_name_dirs;
      filtered = builtins.filter ({ path, ... }: lib.pathIsDirectory path) mapped;
    in
    map (elem: elem.dir) filtered;

  # Generates { <name> = <file>; } entries mapping package names to their `package.nix` files in their `by-name` directory.
  # Could be more efficient, but this is only for testing.
  autoCalledPackageFilesForByNameDir =
    byNameDir:
    let
      entries = builtins.readDir (
        builtins.trace
          "mock-nixpkgs.nix:73: full path = ${
            (lib.concatStringsSep "/" [
              (builtins.toString root)
              byNameDir.path
            ])
          }"
          (
            lib.concatStringsSep "/" [
              (builtins.toString root)
              byNameDir.path
            ]
          )
      );

      namesForShard =
        shard:
        if entries.${shard} != "directory" then
          # Only README.md is allowed to be a file, but it's not this code's job to check for that
          { }
        else
          let
            fileNames = builtins.attrNames (
              builtins.readDir (
                lib.concatStringsSep "/" [
                  (builtins.toString root)
                  byNameDir.path
                  shard
                ]
              )
            );
            fileNameToAttrPath =
              name:
              if byNameDir.unversioned_attr_prefix != "" then
                [
                  byNameDir.unversioned_attr_prefix
                  name
                ]
              else
                [ name ];
            attrSetList = map (
              fileName:
              lib.setAttrByPath (fileNameToAttrPath fileName) (
                lib.concatStringsSep "/" [
                  (builtins.toString root)
                  byNameDir.path
                  shard
                  fileName
                  "package.nix"
                ]
              )
            ) fileNames;
            merged = lib.mergeAttrsList attrSetList;
          in
          merged;
    in
    builtins.foldl' (acc: el: acc // el) { } (map namesForShard (builtins.attrNames entries));

  # Turns autoCalledPackageFiles into an overlay that `callPackage`'s all of them
  autoCalledPackages =
    self: super:
    {
      # Needed to be able to detect empty arguments in all-packages.nix
      # See a more detailed description in pkgs/top-level/by-name-overlay.nix
      _internalCallByNamePackageFile = file: self.callPackage (builtins.toString file) { };
    }
    // lib.mapAttrsRecursiveCond (as: !(as ? "type" && as.type == "derivation")) (
      name: self._internalCallByNamePackageFile
    ) (lib.mergeAttrsList (map autoCalledPackageFilesForByNameDir byNameDirs));

  # A list optionally containing the `all-packages.nix` file from the test case as an overlay
  optionalAllPackagesOverlays =
    let
      filteredByNameDirs = builtins.filter (
        byNameDir:
        (byNameDir ? "all_packages_path")
        && (byNameDir.all_packages_path != null)
        && (builtins.pathExists (root + byNameDir.all_packages_path))
      ) byNameDirs;
      paths = map (byNameDir: byNameDir.all_packages_path) filteredByNameDirs;
      forEachPath = relativePath: import (root + relativePath);
    in
    map forEachPath paths;

  # A list optionally containing the `aliases.nix` file from the test case as an overlay
  # But only if config.allowAliases is not false
  optionalAliasesOverlays =
    if (config.allowAliases or true) then
      let
        filteredByNameDirs = (
          builtins.filter (
            byNameDir:
            (byNameDir ? "aliases_path")
            && (byNameDir.aliases_path != null)
            && (builtins.pathExists (root + byNameDir.aliases_path))
          ) byNameDirs
        );
        paths = map (byNameDir: byNameDir.aliases_path) filteredByNameDirs;
        forEachPath = relativePath: import (root + relativePath);
      in
      map forEachPath paths
    else
      [ ];

  # All the overlays in the right order, including the user-supplied ones
  allOverlays = [
    autoCalledPackages
  ]
  ++ optionalAllPackagesOverlays
  ++ optionalAliasesOverlays
  ++ overlays;

  # Apply all the overlays in order to the base fixed-point function pkgsFun
  f = builtins.foldl' (f: overlay: lib.extends overlay f) pkgsFun allOverlays;
in
# Evaluate the fixed-point
lib.fix f
