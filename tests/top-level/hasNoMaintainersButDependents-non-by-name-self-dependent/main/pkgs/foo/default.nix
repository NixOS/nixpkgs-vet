{ someDrv, foo }:
someDrv
// {
  passthru.self = foo;
  meta.maintainers = [ ];
}
