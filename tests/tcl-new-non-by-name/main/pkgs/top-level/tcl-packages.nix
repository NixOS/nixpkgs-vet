self: super:
{
  # TODO: this SHOULDN'T need the rec.
  # but it does seem to: replacing the bare "callPackage" in foo with tclSelf.callPackage doesn't work.

  # TODO: it seems builtins.unsafeGetAttrPos can't get the location of tclPackages.foo, causing this test to fail.
  tclPackages = self.lib.makeScope self.newScope (tclSelf: self.lib.recurseIntoAttrs rec {
    callPackage =
      fn: args:
      # TODO: uninline from addVariantInfo in eval.nix when done debugging
      if builtins.trace "builtins.isAttrs (tclSelf.callPackage fn args) is ${self.lib.boolToString (builtins.isAttrs (tclSelf.callPackage fn args))}" (builtins.isAttrs (tclSelf.callPackage fn args)) then
      (tclSelf.callPackage fn args) // {
        _callPackageVariant = { ManualDefinition.is_semantic_call_package = (builtins.trace "setting is_semantic_call_package to true" true); };
      } else (tclSelf.callPackage fn args) ;

    foo = callPackage ../development/tcl-modules/foo.nix {};

    recurseForDerivations = true;
  });
}
