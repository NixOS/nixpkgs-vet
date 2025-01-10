{
  stdenv,
  lib,
}:

stdenv.mkDerivation {
  pname = "test";
  version = "1.0";

  src = ./.;

  meta = with lib; {
    maintainers = [ maintainers.johndoe ];
  };
}
