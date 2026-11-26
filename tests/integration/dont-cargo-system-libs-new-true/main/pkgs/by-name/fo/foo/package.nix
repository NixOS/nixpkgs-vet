{ someDrv }: someDrv.overrideAttrs (_: _: { dontCargoUseSystemLibs = true; })
