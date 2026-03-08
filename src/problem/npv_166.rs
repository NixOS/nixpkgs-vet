use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

#[derive(Clone, new)]
pub struct NewTopLevelPackageMustEnableStructuredAttrs {
    #[new(into)]
    package_name: String,
    #[new(into)]
    file: RelativePathBuf,
}

impl fmt::Display for NewTopLevelPackageMustEnableStructuredAttrs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { package_name, file } = self;
        writedoc!(
            f,
            "
            - Attribute `pkgs.{package_name}` is a new package that evaluates with `__structuredAttrs = false`.
              Please enable `__structuredAttrs = true;` in {file}.
            ",
        )
    }
}
