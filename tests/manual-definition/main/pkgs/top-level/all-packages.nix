self: super: {
  nonAttributeSet = self.callPackage ({ someDrv }: someDrv) { };
  nonCallPackage = self.callPackage ({ someDrv }: someDrv) { };
  internalCallByName = self.callPackage ({ someDrv }: someDrv) { };
  nonDerivation = self.callPackage ({ someDrv }: someDrv) { };

  onlyMove = self.callPackage ./../by-name/on/onlyMove/package.nix { };

  noEval = self.callPackage ./../by-name/no/noEval/package.nix { };
}
