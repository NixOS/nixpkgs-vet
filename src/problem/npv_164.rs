use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

#[derive(Clone, new)]
pub struct NewTopLevelPackageMustEnableStrictDeps {
    #[new(into)]
    package_name: String,
    #[new(into)]
    file: RelativePathBuf,
}

impl fmt::Display for NewTopLevelPackageMustEnableStrictDeps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { package_name, file } = self;
        writedoc!(
            f,
            "
            - Attribute `pkgs.{package_name}` is a new package that evaluates with `strictDeps = false`.
              Please enable `strictDeps = true;` in {file}.
            ",
        )
    }
}
