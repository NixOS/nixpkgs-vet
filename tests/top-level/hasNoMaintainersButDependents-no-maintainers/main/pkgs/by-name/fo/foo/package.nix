{ someDrv }:
someDrv
// {
  meta.hasNoMaintainersButDependents = true;
  meta.maintainers = someDrv.meta.maintainers;
}
