self: super: {
  foo-variant-unvarianted = self.callPackage ./../../package.nix { };

  foo-variant-new = self.callPackage ./../by-name/fo/foo/package.nix { };
}
