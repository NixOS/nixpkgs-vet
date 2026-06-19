{ mkFakeDrv }:
mkFakeDrv (finalAttrs: {
  strictDeps = true;
  # No __structuredAttrs
})
// {
  __structuredAttrs = true;
}
