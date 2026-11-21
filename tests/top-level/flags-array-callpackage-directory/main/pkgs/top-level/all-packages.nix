self: super: {
  foo = self.callPackage ../legacy/foo { };
}
