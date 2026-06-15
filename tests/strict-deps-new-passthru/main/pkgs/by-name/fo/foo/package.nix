{ someDrv }:
someDrv
// {
  strictDeps = true;
  drvAttrs = removeAttrs someDrv.drvAttrs [ "strictDeps" ];
}
