final: prev: {
  bar = final.callPackage ({ mkFakeDrv }: mkFakeDrv (_: { })) { };
}
