{ coreutils, lib }:
{
  cat = lib.getExe' coreutils "cat";
}
