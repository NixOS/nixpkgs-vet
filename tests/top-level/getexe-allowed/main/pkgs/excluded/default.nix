{
  lib,
  pkgs,
  coreutils,
  qtbase,
  testPath,
  placeholder,
  pname,
}:
{
  indentedMultiLine = ''
    ${coreutils}/bin/cat
  '';
  withArgs = "${coreutils}/bin/cat --help";
  withInterpolatedName = "${coreutils}/bin/${pname}";
  binDir = "${coreutils}/bin";
  binDirSlash = "${coreutils}/bin/";
  explicitOutput = "${qtbase.dev}/bin/qmake";
  explicitBin = "${pkgs.ffmpeg.bin}/bin/ffmpeg";
  outPath = "${coreutils.outPath}/bin/cat";
  notADerivation = "${testPath}/bin/cat";
  otherFunction = "${placeholder "out"}/bin/cat";
  getDev = "${lib.getDev pkgs.krb5}/bin/krb5-config";
  withPackages = "${pkgs.perl.withPackages (p: [ ])}/bin/perl";
  dynamicAttr = "${pkgs."coreutils"}/bin/cat";
  withDefault = "${pkgs.coreutils or null}/bin/cat";
  nestedPath = "${coreutils}/bin/sub/cat";
  shellQuoted = "${coreutils}/bin/cat\" -r";
  spaceInName = "${coreutils}/bin/cat program";
  differentMainProgram = "${coreutils}/bin/${pkgs.hello.meta.mainProgram}";
  notMainProgram = "${coreutils}/bin/${coreutils.pname}";
  partialGetExe = "${lib.getExe coreutils} --help";
  notGetExe = "${lib.getBin coreutils}";
  unsaturatedGetExe = "${lib.getExe' coreutils}";
  oversaturatedGetExe = "${lib.getExe coreutils "cat"}";
  "${coreutils}/bin/cat" = true;
  alreadyFixed = lib.getExe' coreutils "cat";
  alreadyFixedMain = lib.getExe coreutils;
}
