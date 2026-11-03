{ lib, someDrv }:
someDrv.overrideAttrs (
  _: _: {
    meta.license = [
      lib.licenses.mit
      lib.licenses.asl20
    ];
  }
)
