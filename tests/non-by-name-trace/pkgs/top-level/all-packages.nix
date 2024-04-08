self: super: {
  foo = self.lib.warn "foo should not be used anymore" null;
}
