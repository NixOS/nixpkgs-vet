{ lib, someDrv }:
someDrv.overrideAttrs (
  _: _: {
    meta.license = {
      licenseType = "compound";
      operator = "OR";
      licenses = [
        lib.licenses.mit
        lib.licenses.asl20
      ];
    };
  }
)
