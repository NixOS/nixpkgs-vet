use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

#[derive(Clone, Debug, new)]
pub struct NewTopLevelPackageMustEnableStructuredAttrs {
    #[new(into)]
    full_attr_path: Vec<String>,
    #[new(into)]
    file: RelativePathBuf,
}

impl fmt::Display for NewTopLevelPackageMustEnableStructuredAttrs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            full_attr_path,
            file,
        } = self;
        writedoc!(
            f,
            "
            - Attribute `{}` is a new package with `__structuredAttrs` unset or set to `false`.
              Please enable `__structuredAttrs = true;` in {file}.
            ",
            full_attr_path.join(".")
        )
    }
}
