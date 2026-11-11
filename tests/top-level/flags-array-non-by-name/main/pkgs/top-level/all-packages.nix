self: super: {
  foo = super.someDrv.overrideAttrs (
    _: _: {
      preCheck = ''
        checkFlagsArray+=(one)
      '';
    }
  );
}
