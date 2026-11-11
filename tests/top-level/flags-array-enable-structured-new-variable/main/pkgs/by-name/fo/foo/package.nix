{ someDrv }:
someDrv.overrideAttrs (
  _: _: {
    preCheck = ''
      checkFlagsArray+=(one)
      makeFlagsArray+=(two)
    '';
  }
)
