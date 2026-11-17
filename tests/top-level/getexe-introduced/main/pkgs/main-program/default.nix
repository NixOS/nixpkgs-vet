{ pkgs, lib }:
{
  path = "${pkgs.hello}/bin/${pkgs.hello.meta.mainProgram}";
  viaGetBin = "${lib.getBin pkgs.hello}/bin/${pkgs.hello.meta.mainProgram}";
}
