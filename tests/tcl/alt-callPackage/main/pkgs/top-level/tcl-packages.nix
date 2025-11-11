self: super:
let
  newScope =
    _: fn: args:
    addVariantInfo (super.callPackage fn args) {
      ManualDefinition.is_semantic_call_package = true;
    };
  addVariantInfo =
    value: variant:
    if builtins.isAttrs value then value // { _callPackageVariant = variant; } else value;
in
{
  tclPackages = self.lib.makeScope newScope (
    tclSelf:
    (
      self.lib.recurseIntoAttrs (super.tclPackages or { })
      // {
        alt.callPackage = self.lib.callPackageWith tclSelf;
        foo = tclSelf.alt.callPackage ({ }: self.someDrv) { };
      }
    )
  );
}
