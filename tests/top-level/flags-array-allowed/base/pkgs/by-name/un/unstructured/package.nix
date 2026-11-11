{ someDrv }:
someDrv.overrideAttrs (
  _: _: {
    __structuredAttrs = false;
    preCheck = ''
      checkFlagsArray+=(one)
    '';
  }
)
