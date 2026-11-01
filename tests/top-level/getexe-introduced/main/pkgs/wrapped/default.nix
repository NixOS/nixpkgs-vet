{ lib, pkgs }:
{
  a = "${lib.getExe pkgs.hello}";
  b = "${lib.getExe' pkgs.hello "hello"}";
  c = [ "${getExe pkgs.hello}" ];
  d = "${lib.meta.getExe pkgs.hello}";
}
