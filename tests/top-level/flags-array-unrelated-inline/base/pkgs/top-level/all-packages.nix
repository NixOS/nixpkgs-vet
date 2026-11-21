self: super: {
  foo = super.someDrv;
  bar = super.someDrv.overrideAttrs (
    _: _: {
      __structuredAttrs = false;
      preCheck = "checkFlags+=(one)";
    }
  );
}
