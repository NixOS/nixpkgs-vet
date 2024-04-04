self: super: {

  bar = (x: x) self.callPackage ./../by-name/fo/foo/package.nix { someFlag = true; };
  foo = self.bar;

}
