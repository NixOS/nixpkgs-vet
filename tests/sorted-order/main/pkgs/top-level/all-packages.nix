self: super: {
  a = self.callPackage ./../by-name/a/a/package.nix { };
  b = self.callPackage ({ someDrv }: someDrv) { };
  c = self.callPackage ./../by-name/c/c/package.nix { };
  d = self.callPackage ({ someDrv }: someDrv) { };
}
