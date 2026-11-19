self: super: {
  python3Packages = {
    before = self.callPackage ({ someDrv }: someDrv) { };
  };
}
