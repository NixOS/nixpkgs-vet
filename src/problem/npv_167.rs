use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

#[derive(Clone, new)]
pub struct TopLevelPackageDisabledStructuredAttrs {
    #[new(into)]
    package_name: String,
    #[new(into)]
    file: RelativePathBuf,
}

impl fmt::Display for TopLevelPackageDisabledStructuredAttrs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { package_name, file } = self;
        writedoc!(
            f,
            "
            - Attribute `pkgs.{package_name}` previously evaluated with `__structuredAttrs = true`, but now evaluates with `__structuredAttrs = false`.
              Please re-enable `__structuredAttrs = true;` in {file}.
            ",
        )
    }
}
