{ mkFakeDrv }:
mkFakeDrv (finalAttrs: {
  strictDeps = true;
  __structuredAttrs = true;
  # don't set dontCargoUseSystemLibs
})
