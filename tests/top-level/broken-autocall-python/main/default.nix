args:
builtins.removeAttrs (import <test-nixpkgs> { root = ./.; } args) [
  "python3Packages"
  "foo"
]
