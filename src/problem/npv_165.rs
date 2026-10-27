use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

#[derive(Clone, Debug, new)]
pub struct TopLevelPackageDisabledStrictDeps {
    #[new(into)]
    package_path: Vec<String>,
    #[new(into)]
    file: RelativePathBuf,
}

impl fmt::Display for TopLevelPackageDisabledStrictDeps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { package_path, file } = self;
        writedoc!(
            f,
            "
            - Attribute `{}` previously evaluated with `strictDeps = true`, but now evaluates with `strictDeps = false`.
              Please re-enable `strictDeps = true;` in {file}.
            ",
            package_path.join("."),
        )
    }
}
