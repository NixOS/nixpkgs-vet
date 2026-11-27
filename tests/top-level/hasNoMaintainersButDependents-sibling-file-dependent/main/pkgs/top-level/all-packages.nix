self: super: {
  bar = self.callPackage ../by-name/fo/foo/other.nix { };
}
