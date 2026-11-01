{ coreutils, lib }:
{
  description = "an unrelated edit that moves the old line";
  env = lib.getExe' coreutils "env";
  cat = "${coreutils}/bin/cat";
}
