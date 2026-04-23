{ someDrv }:

let
  fetchpatch2 = { url, hash }: builtins.fetchurl { inherit url hash; };
in
someDrv
// {
  patches = [
    (fetchpatch2 {
      url = "https://github.com/AndyLavr/amd-ucodegen/compare/0d34b54e396ef300d0364817e763d2c7d1ffff02...dobo90:amd-ucodegen:7a3c51e821df96910ecb05b22f3e4866b4fb85b2.patch?some=param";
      hash = "sha256-jvsvu9QgXikwsxjPiTaRff+cOg/YQmKg1MYKyBoMRQI=";
    })
  ];
}
