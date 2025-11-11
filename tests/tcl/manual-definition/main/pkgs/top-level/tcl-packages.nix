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
        someDrv = self.someDrv;
        nonAttributeSet = tclSelf.callPackage ({ someDrv }: someDrv) { };
        nonCallPackage = tclSelf.callPackage ({ someDrv }: someDrv) { };
        internalCallByName = tclSelf.callPackage ({ someDrv }: someDrv) { };
        nonDerivation = tclSelf.callPackage ({ someDrv }: someDrv) { };

        onlyMove = tclSelf.callPackage ./../development/tcl-modules/by-name/on/onlyMove/package.nix { };

        noEval = tclSelf.callPackage ./../development/tcl-modules/by-name/no/noEval/package.nix { };
      }
    )
  );
}
