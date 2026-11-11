{ someDrv }:
someDrv.overrideAttrs (
  _: _: {
    preCheck = ''
      checkFlagsArray+=(one)
      customFlagsArray+=(two)
      checkFlagsArray+=(three)
    '';
  }
)
