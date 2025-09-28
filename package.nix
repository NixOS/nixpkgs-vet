{
  lib,
  rustPlatform,
  path,
  nix,
  nixVersions,
  lixPackageSets,
  clippy,
  makeWrapper,
  versionCheckHook,

  nixVersionsToTest ? [
    nix
    nixVersions.stable
    nixVersions.latest
    lixPackageSets.stable.lix
    lixPackageSets.latest.lix
  ],

  initNix,
  version,
}:
let
  fs = lib.fileset;
in
rustPlatform.buildRustPackage {
  pname = "nixpkgs-vet";
  inherit version;

  src = fs.toSource {
    root = ./.;
    fileset = fs.unions [
      ./Cargo.lock
      ./Cargo.toml
      ./src
      ./tests
    ];
  };

  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = [
    clippy
    makeWrapper
  ];

  doInstallCheck = true;
  nativeInstallCheckInputs = [ versionCheckHook ];

  env.NIXPKGS_VET_NIX_PACKAGE = lib.getBin nix;
  env.NIXPKGS_VET_NIXPKGS_LIB = "${path}/lib";

  checkPhase = ''
    # This path will be symlinked to the current version that is being tested
    nixPackage=$(mktemp -d)/nix

    # For initNix
    export PATH=$nixPackage/bin:$PATH

    # This is what nixpkgs-vet uses
    export NIXPKGS_VET_NIX_PACKAGE=$nixPackage

    ${lib.concatMapStringsSep "\n" (nix: ''
      ln -s ${lib.getBin nix} "$nixPackage"
      echo "Testing with $(nix --version)"
      ${initNix}
      runHook cargoCheckHook
      rm "$nixPackage"
    '') (lib.unique nixVersionsToTest)}

    # --tests or --all-targets include tests for linting
    cargo clippy --all-targets -- -D warnings
  '';
  postInstall = ''
    wrapProgram $out/bin/nixpkgs-vet \
      --set NIXPKGS_VET_NIX_PACKAGE ${lib.getBin nix}
  '';

  # silence a warning when building
  meta.mainProgram = "nixpkgs-vet";
}
