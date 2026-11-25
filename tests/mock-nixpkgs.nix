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
{ root }:
# The arguments for the Nixpkgs function
{
  # Passed by the checker to modify `callPackage`
  overlays ? [ ],
  # Passed by the checker to make sure a real Nixpkgs isn't influenced by impurities
  config ? { },
  # Passed by the checker to make sure a real Nixpkgs isn't influenced by impurities
  system ? null,
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
      mkFakeDrv =
        args:
        lib.makeExtensibleWithCustomName "overrideAttrs" (
          finalAttrs:
          {
            type = "derivation";
          }
          // args finalAttrs
        );
    }
    # This mapAttrs is a very hacky workaround necessary because for all attributes defined in Nixpkgs,
    # the files that define them are verified to be within Nixpkgs.
    # This is usually a very safe assumption, but it fails in these tests for someDrv,
    # because it's technically defined outside the Nixpkgs directories of each test case.
    # By using `mapAttrs`, `builtins.unsafeGetAttrPos` just returns `null`,
    # which then doesn't trigger this check
    // lib.mapAttrs (name: value: value) {
      someDrv = self.mkFakeDrv (finalAttrs: {
        strictDeps = true;
        __structuredAttrs = true;
      });
    };

  # Generates { <name> = <file>; } entries mapping package names to their `package.nix` files in the `baseDirectory`
  # Could be more efficient, but this is only for testing.
  autoCalledPackageFilesFor =
    baseDirectory:
    let
      entries = if builtins.pathExists baseDirectory then builtins.readDir baseDirectory else { };

      namesForShard =
        shard:
        if entries.${shard} != "directory" then
          # Only README.md is allowed to be a file, but it's not this code's job to check for that
          { }
        else
          builtins.mapAttrs (name: _: baseDirectory + "/${shard}/${name}/package.nix") (
            builtins.readDir (baseDirectory + "/${shard}")
          );
    in
    builtins.foldl' (acc: el: acc // el) { } (map namesForShard (builtins.attrNames entries));

  # Turns autoCalledPackagesFor into an overlay that `callPackage`'s all of them
  autoCalledPackagesFor =
    by_name_subpth: final: prev:
    builtins.mapAttrs (name: file: final.callPackage file { }) (
      autoCalledPackageFilesFor (root + "/${by_name_subpth}")
    );

  autoCalledPackages = autoCalledPackagesFor "pkgs/by-name";

  # A list optionally containing an overlay imported from the given path.
  importIfExists =
    path: if builtins.pathExists (root + "/${path}") then [ (import (root + "/${path}")) ] else [ ];

  # A list optionally containing the `all-packages.nix` file from the test case as an overlay
  optionalAllPackagesOverlay = importIfExists "pkgs/top-level/all-packages.nix";

  # A list optionally containing the `aliases.nix` file from the test case as an overlay
  # But only if config.allowAliases is not false
  optionalAliasesOverlay =
    if (config.allowAliases or true) then (importIfExists "aliases.nix") else [ ];

  # A list of top level package sets in pkgs/sets.
  # For now the tests only cover top level package sets.
  packageSets =
    if builtins.pathExists (root + "/pkgs/sets") then
      builtins.attrNames (builtins.readDir (root + "/pkgs/sets"))
    else
      [ ];

  # A list of overlays, each containing a package set with their autocalled packages,
  # manual packages and aliases.
  packageSetOverlays = lib.flip map packageSets (
    name:
    let
      # Like the outer scope autoCalledPackages, but for the package set
      autoCalledPackages = autoCalledPackagesFor "pkgs/sets/${name}/by-name";

      # Like the outer scope optionalAllPackagesOverlay, but for the package set
      optionalManualPackagesOverlay = importIfExists "pkgs/sets/${name}/manual-packages.nix";

      # Like the outer scope optionalAliasesOverlay, but for the package set
      optionalAliasesOverlay =
        if (config.allowAliases or true) then (importIfExists "pkgs/sets/${name}/aliases.nix") else [ ];

      # Like the outer scope allOverlays but for the package set
      allOverlays = [
        autoCalledPackages
      ]
      ++ optionalManualPackagesOverlay
      ++ optionalAliasesOverlay;

      # Like the outer scope f but for the package set
      f = lib.foldl' (lib.flip lib.extends) (self: { }) allOverlays;
    in
    rootfinal: rootprev: {
      ${name} = lib.makeScope rootfinal.newScope f;
    }
  );

  # All the overlays in the right order, including the user-supplied ones
  allOverlays = [
    autoCalledPackages
  ]
  ++ optionalAllPackagesOverlay
  ++ optionalAliasesOverlay
  ++ packageSetOverlays
  ++ overlays;

  # Apply all the overlays in order to the base fixed-point function pkgsFun
  f = builtins.foldl' (f: overlay: lib.extends overlay f) pkgsFun allOverlays;
in
# Evaluate the fixed-point
lib.fix f
