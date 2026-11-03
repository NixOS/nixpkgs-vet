self: super: {
  foo = self.callPackage ../foo { };
}
