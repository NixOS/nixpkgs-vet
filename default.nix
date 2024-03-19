let
  sources = import ./npins;
in
{
  system ? builtins.currentSystem,
  nixpkgs ? sources.nixpkgs,
}:
let
  pkgs = import nixpkgs {
    inherit system;
    config = {};
    overlays = [];
  };

  runtimeExprPath = ./src/eval.nix;
  testNixpkgsPath = ./tests/mock-nixpkgs.nix;
  nixpkgsLibPath = nixpkgs + "/lib";

  # Needed to make Nix evaluation work inside nix builds
  initNix = ''
    export TEST_ROOT=$(pwd)/test-tmp
    export NIX_CONF_DIR=$TEST_ROOT/etc
    export NIX_LOCALSTATE_DIR=$TEST_ROOT/var
    export NIX_LOG_DIR=$TEST_ROOT/var/log/nix
    export NIX_STATE_DIR=$TEST_ROOT/var/nix
    export NIX_STORE_DIR=$TEST_ROOT/store

    # Ensure that even if tests run in parallel, we don't get an error
    # We'd run into https://github.com/NixOS/nix/issues/2706 unless the store is initialised first
    nix-store --init
  '';

  build = pkgs.callPackage ./package.nix {
    inherit nixpkgsLibPath initNix runtimeExprPath testNixpkgsPath;
  };

in
build // {

  inherit build;

  # Used by CI and good for debugging
  inherit pkgs;

  shell = pkgs.mkShell {
    env.NIX_CHECK_BY_NAME_EXPR_PATH = toString runtimeExprPath;
    env.NIX_PATH = "test-nixpkgs=${toString testNixpkgsPath}:test-nixpkgs/lib=${toString nixpkgsLibPath}";
    env.RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
    inputsFrom = [ build ];
    nativeBuildInputs = with pkgs; [
      npins
      rust-analyzer
    ];
  };

  # Tests the tool on the pinned Nixpkgs tree, this is a good sanity check
  checks.nixpkgs = pkgs.runCommand "test-nixpkgs-check-by-name" {
    nativeBuildInputs = [
      build
      pkgs.nix
    ];
    nixpkgsPath = nixpkgs;
  } ''
    ${initNix}
    nixpkgs-check-by-name --base "$nixpkgsPath" "$nixpkgsPath"
    touch $out
  '';
}
