{
  stdenv,
  lib,
}:

stdenv.mkDerivation {
  pname = "test";
  version = "1.0";

  src = ./.;

  meta = {
    maintainers = with lib.maintainers; [ johndoe ];
  };
}
