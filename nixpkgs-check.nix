{
  lib,
  runCommand,
  nixpkgs-vet,
  initNix,
  nixpkgs,
  nix,
  nixVersions,
  lixPackageSets,
}:
let
  # Given an attrset, return only the values that both eval and are derivations.
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
          lixVersions = lib.mapAttrs mkNixpkgsCheck {
            stable = lixPackageSets.stable.lix;
            latest = lixPackageSets.latest.lix;
          };
        };
      }
      ''
        ${initNix}
        # This is what nixpkgs-vet uses
        export NIXPKGS_VET_NIX_PACKAGE=${lib.getBin nix}
        ${nixpkgs-vet}/bin/.nixpkgs-vet-wrapped --base "${nixpkgs}" "${nixpkgs}"
        touch $out
      '';
in
mkNixpkgsCheck nix.name nix
