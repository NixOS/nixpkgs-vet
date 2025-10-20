{
  lib,
  rustPlatform,
  path,
  nix,
  nixVersions,
  lixPackageSets,
  clippy,
  makeWrapper,
  writeText,
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
  configFile = writeText "by-name-config-generated.json" (builtins.toJSON (import ./by-name-config.nix));
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
    nix
    makeWrapper
  ];

  doInstallCheck = true;
  nativeInstallCheckInputs = [ versionCheckHook ];
  versionCheckProgramArg = "--version";

  env.NIXPKGS_VET_NIX_PACKAGE = lib.getBin nix;
  env.NIXPKGS_VET_NIXPKGS_LIB = "${path}/lib";

  checkPhase = ''
    # This path will be symlinked to the current version that is being tested
    nixPackage=$(mktemp -d)/nix

    # For initNix
    export PATH=$nixPackage/bin:$PATH

    # This is what nixpkgs-vet uses
    export NIXPKGS_VET_NIX_PACKAGE=$nixPackage
    export NIXPKGS_VET_CONFIG_FILE=${configFile}

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
      --set NIXPKGS_VET_NIX_PACKAGE ${lib.getBin nix} \
      --set NIXPKGS_VET_CONFIG_FILE ${configFile}
  '';

  # silence a warning when building
  meta.mainProgram = "nixpkgs-vet";
}
