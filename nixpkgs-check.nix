{
  lib,
  runCommand,
  nixpkgs-check-by-name,
  initNix,
  nixpkgs,
  nix,
  nixVersions,
  lixVersions,
}:
let
  # Given an attrset, return only the values that both eval and are derivations.
  #
  # We do this to avoid encoding information about which names are in present in the attrset.
  # For instance, `nixVersions` contains `nix_2_10`, which throws, and `lixVersions` does not
  # contain `minimum` or `git`, but `nixVersions` does.
  #
  # Let's just map over those attrsets and return what's useful from there.
  derivationsFromAttrset =
    attrset:
    lib.filterAttrs (
      name: value:
      let
        eval = builtins.tryEval value;
      in
      eval.success && lib.isDerivation eval.value
    ) attrset;

  mkNixpkgsCheck =
    name: nix:
    runCommand "test-nixpkgs-check-by-name-with-${nix.name}"
      {
        nativeBuildInputs = [
          nixpkgs-check-by-name
          nix
        ];

        env.NIX_CHECK_BY_NAME_NIX_PACKAGE = lib.getBin nix;

        passthru = {
          # Allow running against all other Nix versions.
          nixVersions = lib.mapAttrs mkNixpkgsCheck (derivationsFromAttrset nixVersions);

          # Allow running against all other Lix versions.
          lixVersions = lib.mapAttrs mkNixpkgsCheck (derivationsFromAttrset lixVersions);
        };
      }
      ''
        ${initNix}
        # This is what nixpkgs-check-by-name uses
        export NIX_CHECK_BY_NAME_NIX_PACKAGE=${lib.getBin nix}
        ${nixpkgs-check-by-name}/bin/.nixpkgs-check-by-name-wrapped --base "${nixpkgs}" "${nixpkgs}"
        touch $out
      '';
in
mkNixpkgsCheck nix.name nix
