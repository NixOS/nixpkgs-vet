args:
let
  original = import <test-nixpkgs> { root = ./.; } args;
in
builtins.removeAttrs original [ "foo" ]
// {
  fooPackages = builtins.removeAttrs original.fooPackages [ "foo" ];
}
