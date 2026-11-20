use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

#[derive(Clone, Debug, new)]
pub struct NewTopLevelPackageUsesLicenseList {
    #[new(into)]
    package_path: Vec<String>,
    #[new(into)]
    file: RelativePathBuf,
}

impl fmt::Display for NewTopLevelPackageUsesLicenseList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { package_path, file } = self;
        writedoc!(
            f,
            "
            - Attribute `{}` is a new package with a list for `meta.license`.
              Please use a single license, or combine licenses with `lib.licenses.OR` or `lib.licenses.AND` in {file}.
            ",
            package_path.join("."),
        )
    }
}
