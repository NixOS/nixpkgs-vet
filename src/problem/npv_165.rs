use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

#[derive(Clone, new)]
pub struct TopLevelPackageDisabledStrictDeps {
    #[new(into)]
    package_name: String,
    #[new(into)]
    file: RelativePathBuf,
}

impl fmt::Display for TopLevelPackageDisabledStrictDeps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { package_name, file } = self;
        writedoc!(
            f,
            "
            - Attribute `pkgs.{package_name}` previously evaluated with `strictDeps = true`, but now evaluates with `strictDeps = false`.
              Please re-enable `strictDeps = true;` in {file}.
            ",
        )
    }
}
