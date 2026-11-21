self: super: {
  foo = super.someDrv.overrideAttrs (
    _: _: {
      preCheck = ''
        checkFlags+=(one)
      '';
    }
  );
}
