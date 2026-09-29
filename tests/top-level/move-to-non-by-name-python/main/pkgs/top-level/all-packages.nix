self: super: {
  python3Packages = {
    foo1 = self.callPackage ({ someDrv }: someDrv) { };
    foo2 = self.callPackage ./../../without-config.nix { };
  };
}
