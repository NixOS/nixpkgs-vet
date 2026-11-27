use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

use crate::attr_path::AttrPathBuf;

#[derive(Clone, Debug, new)]
pub struct TopLevelPackageSwitchedToLicenseList {
    #[new(into)]
    package_path: AttrPathBuf,
    #[new(into)]
    file: RelativePathBuf,
}

impl fmt::Display for TopLevelPackageSwitchedToLicenseList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { package_path, file } = self;
        writedoc!(
            f,
            "
            - Attribute `{package_path}` previously did not use a list for `meta.license`, but now does.
              Please use a single license, or combine licenses with `lib.licenses.OR` or `lib.licenses.AND` in {file}.
            ",
        )
    }
}
