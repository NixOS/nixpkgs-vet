use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

#[derive(Clone, Debug, new)]
pub struct NewTopLevelPackageMustEnableStrictDeps {
    #[new(into)]
    package_path: Vec<String>,
    #[new(into)]
    file: RelativePathBuf,
}

impl fmt::Display for NewTopLevelPackageMustEnableStrictDeps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { package_path, file } = self;
        writedoc!(
            f,
            "
            - Attribute `{}` is a new package with `strictDeps` unset or set to `false`.
              Please enable `strictDeps = true;` in {file}.
            ",
            package_path.join("."),
        )
    }
}
