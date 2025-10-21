self: super: {
  foo = self.callPackage ../development/tcl-modules/foo.nix { };
}
