{
  lib,
  rustPlatform,
  nix,
  nixVersions,
  clippy,
  makeWrapper,

  nixVersionsToTest ? [
    nix
    nixVersions.stable
  ],

  nixpkgsLibPath,
  initNix,
  runtimeExprPath,
  testNixpkgsPath,
  version,
}:
let
  fs = lib.fileset;
in
rustPlatform.buildRustPackage {
  pname = "nixpkgs-check-by-name";
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
  env.NIX_CHECK_BY_NAME_EXPR_PATH = "${runtimeExprPath}";
  env.NIX_CHECK_BY_NAME_NIX_PACKAGE = lib.getBin nix;
  env.NIX_PATH = "test-nixpkgs=${testNixpkgsPath}:test-nixpkgs/lib=${nixpkgsLibPath}";
  checkPhase = ''
    # This path will be symlinked to the current version that is being tested
    nixPackage=$(mktemp -d)/nix
    # For initNix
    export PATH=$nixPackage/bin:$PATH
    # This is what nixpkgs-check-by-name uses
    export NIX_CHECK_BY_NAME_NIX_PACKAGE=$nixPackage

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
    wrapProgram $out/bin/nixpkgs-check-by-name \
      --set NIX_CHECK_BY_NAME_EXPR_PATH "$NIX_CHECK_BY_NAME_EXPR_PATH" \
      --set NIX_CHECK_BY_NAME_NIX_PACKAGE ${lib.getBin nix}
  '';
}
