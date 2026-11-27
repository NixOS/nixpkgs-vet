# Lives next to foo's package.nix, but defines bar, which depends on foo
{ someDrv, foo }:
someDrv
// {
  passthru.foo = foo;
}
