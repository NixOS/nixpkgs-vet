{ someDrv }:
someDrv
// {
  __structuredAttrs = true;
  drvAttrs = removeAttrs someDrv.drvAttrs [ "__structuredAttrs" ];
}
