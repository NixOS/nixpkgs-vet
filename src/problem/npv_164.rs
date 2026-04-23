use std::fmt;

use crate::gh_write::{Options, gh_write};
use derive_new::new;
use indoc::formatdoc;
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
        gh_write(
            f,
            formatdoc!("
            - Attribute `pkgs.{package_name}` is a new package with `strictDeps` unset or set to `false`.
              Please enable `strictDeps = true;` in {file}.
            "),
            Options {
                file: Some(file),
                ..Default::default()
            },
        )
    }
}
