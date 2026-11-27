use std::fmt;

use derive_new::new;
use indoc::writedoc;

use crate::attr_path::AttrPathBuf;

#[derive(Clone, Debug, new)]
pub struct DependentsAttrsSetWithMaintainers {
    #[new(into)]
    package_path: AttrPathBuf,
}

impl fmt::Display for DependentsAttrsSetWithMaintainers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writedoc!(
            f,
            "
            - {}: Has `meta.hasNoMaintainersButDependents = true` set, but there are maintainers. Please unset the attribute
            ",
            self.package_path,
        )
    }
}
