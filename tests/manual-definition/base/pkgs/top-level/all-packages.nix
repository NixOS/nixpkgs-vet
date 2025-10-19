self: super: {
  nonAttributeSet = null;
  nonCallPackage = self.someDrv;
  internalCallByName = self._internalCallByNamePackageFile ./../../some-pkg.nix;
  nonDerivation = self.callPackage ({ }: { }) { };
  nonDerivation2 = { foo = "bar"; };

  onlyMove = self.callPackage ({ someDrv }: someDrv) { };
  noEval = throw "foo";
}
