let
  sources = import ./npins;
in
{
  system ? builtins.currentSystem,
  nixpkgs ? sources.nixpkgs,
  treefmt-nix ? sources.treefmt-nix,
}:
let
  pkgs = import nixpkgs {
    inherit system;
    config = { };
    overlays = [ ];
  };
  inherit (pkgs) lib;

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

  # Determine version from Cargo.toml
  version = (lib.importTOML ./Cargo.toml).package.version;

  treefmtEval = (import treefmt-nix).evalModule pkgs {
    # Used to find the project root
    projectRootFile = ".git/config";

    programs.rustfmt.enable = true;
    programs.nixfmt-rfc-style.enable = true;
    programs.shfmt.enable = true;
    settings.formatter.shfmt.options = [ "--space-redirects" ];
  };

  buildUnfiltered = pkgs.callPackage ./package.nix {
    inherit
      nixpkgsLibPath
      initNix
      runtimeExprPath
      testNixpkgsPath
      version
      ;
  };

  results = {
    # We're using this value as the root result. By default, derivations expose all of their
    # internal attributes, which is very messy. We prevent this using lib.lazyDerivation
    build = lib.lazyDerivation { derivation = buildUnfiltered; };

    shell = pkgs.mkShell {
      env.NIX_CHECK_BY_NAME_EXPR_PATH = toString runtimeExprPath;
      env.NIX_PATH = "test-nixpkgs=${toString testNixpkgsPath}:test-nixpkgs/lib=${toString nixpkgsLibPath}";
      env.RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
      # inputsFrom relies on .nativeBuildInputs and co. to exist, which are filtered out by lazyDerivation
      inputsFrom = [ buildUnfiltered ];
      nativeBuildInputs = with pkgs; [
        npins
        rust-analyzer
        treefmtEval.config.build.wrapper
      ];
    };

    # This checks that all Git-tracked files are formatted appropriately
    treefmt = treefmtEval.config.build.check (
      lib.fileset.toSource {
        root = ./.;
        fileset = lib.fileset.gitTracked ./.;
      }
    );

    # Run regularly by CI and turned into a PR
    autoPrUpdate =
      let
        updateScripts = {
          npins = pkgs.writeShellApplication {
            name = "update-npins";
            runtimeInputs = with pkgs; [ npins ];
            text = ''
              echo "<details><summary>npins changes</summary>"
              # Needed because GitHub's rendering of the first body line breaks down otherwise
              echo ""
              echo '```'
              npins update --directory "$1/npins" 2>&1
              echo  '```'
              echo "</details>"
            '';
          };
          cargo = pkgs.writeShellApplication {
            name = "update-cargo";
            runtimeInputs = with pkgs; [ cargo ];
            text = ''
              echo "<details><summary>cargo changes</summary>"
              # Needed because GitHub's rendering of the first body line breaks down otherwise
              echo ""
              echo '```'
              cargo update --manifest-path "$1/Cargo.toml" 2>&1
              echo  '```'
              echo "</details>"
            '';
          };
          githubActions = pkgs.writeShellApplication {
            name = "update-github-actions";
            runtimeInputs = with pkgs; [
              dependabot-cli
              jq
              github-cli
              coreutils
            ];
            text = builtins.readFile ./scripts/update-github-actions.sh;
          };
        };
      in
      pkgs.writeShellApplication {
        name = "auto-pr-update";
        text = ''
          # Prevent impurities
          unset PATH
          ${lib.concatMapStringsSep "\n" (script: ''
            echo >&2 "Running ${script}"
            ${lib.getExe script} "$1"
          '') (lib.attrValues updateScripts)}
        '';
      };

    # Tests the tool on the pinned Nixpkgs tree, this is a good sanity check
    nixpkgsCheck =
      pkgs.runCommand "test-nixpkgs-check-by-name"
        {
          nativeBuildInputs = [
            results.build
            pkgs.nix
          ];
          nixpkgsPath = nixpkgs;
        }
        ''
          ${initNix}
          nixpkgs-check-by-name --base "$nixpkgsPath" "$nixpkgsPath"
          touch $out
        '';
  };
in
results.build
// results
// {

  # Good for debugging
  inherit pkgs;

  # Built by CI
  ci = pkgs.linkFarm "ci" results;

  # Used by CI to determine whether a new version should be released
  inherit version;
}
