{
  description = "nixpkge-vet inputs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";

    flake-compat.url = "github:NixOS/flake-compat";
    flake-compat.flake = false;
  };

  # This flake is used only for its inputs
  outputs = inputs: { };
}
