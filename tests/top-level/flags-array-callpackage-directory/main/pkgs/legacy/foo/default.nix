{ someDrv }:
someDrv.overrideAttrs (
  _: _: {
    preCheck = "checkFlagsArray+=(one)";
  }
)
