use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

#[derive(Clone, Debug, new)]
pub struct TopLevelPackageEnabledCargoSystemLibs {
    #[new(into)]
    package_name: String,
    #[new(into)]
    file: RelativePathBuf,
}

impl fmt::Display for TopLevelPackageEnabledCargoSystemLibs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { package_name, file } = self;
        writedoc!(
            f,
            "
            - Attribute `{package_name}` sets `dontCargoUseSystemLibs` to `true`.
              Please remove this and link to the nixpkgs-provided library, or if this is truly not possible,
              please add a target fix for the specific library that needs to be allowed in {file}.

              `dontCargoUseSystemLibs` is for out-of-tree usage, and is not allowed within Nixpkgs.
            ",
        )
    }
}
