{
  lib,
  pkgs,
  bash,
  getBin,
}:
{
  interpreter = "${bash}/bin/bash";
  a = "${lib.getBin pkgs.coreutils}/bin/env";
  b = "${getBin pkgs.coreutils}/bin/env";
}
