{
  lib,
  runCommand,
  nixpkgs-vet,
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
    runCommand "test-nixpkgs-vet-with-${nix.name}"
      {
        nativeBuildInputs = [
          nixpkgs-vet
          nix
        ];

        env.NIXPKGS_VET_NIX_PACKAGE = lib.getBin nix;

        passthru = {
          # Allow running against all other Nix versions.
          nixVersions = lib.mapAttrs mkNixpkgsCheck (derivationsFromAttrset nixVersions);

          # Allow running against all other Lix versions.
          lixVersions = lib.mapAttrs mkNixpkgsCheck (derivationsFromAttrset lixVersions);
        };
      }
      ''
        ${initNix}
        # This is what nixpkgs-vet uses
        export NIXPKGS_VET_NIX_PACKAGE=${lib.getBin nix}
        time ${nixpkgs-vet}/bin/.nixpkgs-vet-wrapped --base "${nixpkgs}" "${nixpkgs}"
        touch $out
      '';
in
mkNixpkgsCheck nix.name nix
