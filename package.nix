{
  lib,
  rustPlatform,
  nix,
  rustfmt,
  clippy,
  makeWrapper,

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
    nix
    rustfmt
    clippy
    makeWrapper
  ];
  env.NIX_CHECK_BY_NAME_EXPR_PATH = "${runtimeExprPath}";
  env.NIX_PATH = "test-nixpkgs=${testNixpkgsPath}:test-nixpkgs/lib=${nixpkgsLibPath}";
  preCheck = initNix;
  postCheck = ''
    cargo fmt --check
    # --tests or --all-targets include tests for linting
    cargo clippy --all-targets -- -D warnings
  '';
  postInstall = ''
    wrapProgram $out/bin/nixpkgs-check-by-name \
      --set NIX_CHECK_BY_NAME_EXPR_PATH "$NIX_CHECK_BY_NAME_EXPR_PATH"
  '';

}
