use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

use crate::attr_path::AttrPathBuf;

#[derive(Clone, Debug, new)]
pub struct NewTopLevelPackageMustEnableStructuredAttrs {
    #[new(into)]
    package_path: AttrPathBuf,
    #[new(into)]
    file: RelativePathBuf,
}

impl fmt::Display for NewTopLevelPackageMustEnableStructuredAttrs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { package_path, file } = self;
        writedoc!(
            f,
            "
            - Attribute `{package_path}` is a new package with `__structuredAttrs` unset or set to `false`.
              Please enable `__structuredAttrs = true;` in {file}.
            ",
        )
    }
}
