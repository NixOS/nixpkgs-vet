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
        foo1 = tclSelf.callPackage ({ someDrv }: someDrv) { };
        foo2 = tclSelf.callPackage ./../../without-config.nix { };
        foo3 = tclSelf.callPackage ({ someDrv, enableFoo }: someDrv) { enableFoo = null; };
        foo4 = tclSelf.callPackage ./../../with-config.nix { enableFoo = null; };
      }
    )
  );
}
