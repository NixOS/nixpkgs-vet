self: super: {
  foo1 = self.callPackage ({ someDrv }: someDrv) { };
  foo2 = self.callPackage ./../../without-config.nix { };
}
