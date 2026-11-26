{ mkFakeDrv }:
mkFakeDrv (finalAttrs: {
  strictDeps = true;
  # No __structuredAttrs
})
