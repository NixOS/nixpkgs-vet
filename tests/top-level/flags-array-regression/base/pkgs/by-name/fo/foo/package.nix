{ someDrv }:
someDrv.overrideAttrs (
  _: _: {
    preCheck = ''
      checkFlags+=(one)
    '';
  }
)
