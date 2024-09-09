use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

use crate::structure::PACKAGE_NIX_FILENAME;

#[derive(Clone, new)]
pub struct NixFileContainsPathOutsideDirectory {
    #[new(into)]
    relative_package_dir: RelativePathBuf,
    #[new(into)]
    subpath: RelativePathBuf,
    line: usize,
    #[new(into)]
    text: String,
}

impl fmt::Display for NixFileContainsPathOutsideDirectory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            relative_package_dir,
            subpath,
            line,
            text,
        } = self;
        writedoc!(
            f,
            "
            - {relative_package_dir}: File {subpath} at line {line} contains the path expression \"{text}\" which may point outside the directory of that package.
              This is undesirable because it creates dependencies between internal paths, making it harder to reorganise Nixpkgs in the future.
              Alternatives include:
              - If you are creating a new version of a package with a common file between versions, consider following the recommendation in https://github.com/NixOS/nixpkgs/tree/master/pkgs/by-name#recommendation-for-new-packages-with-multiple-versions.
              - If the path being referenced could be considered a stable interface with multiple uses, consider exposing it via a `pkgs` attribute, then taking it as a attribute argument in {PACKAGE_NIX_FILENAME}.
              - If the path being referenced is internal and has multiple uses, consider passing the file as an explicit `callPackage` argument in `pkgs/top-level/all-packages.nix`.
              - If the path being referenced is internal and will need to be modified independently of the original, consider copying it into the {relative_package_dir} directory.
            "
        )
    }
}
