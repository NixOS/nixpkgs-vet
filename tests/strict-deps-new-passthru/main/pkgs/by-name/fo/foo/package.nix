{ mkFakeDrv }:
mkFakeDrv (finalAttrs: {
  __structuredAttrs = true;
  # No strictDeps
})
// {
  strictDeps = true;
}
