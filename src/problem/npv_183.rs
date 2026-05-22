use std::fmt;

use derive_new::new;
use indoc::writedoc;

#[derive(Clone, Debug, new)]
pub struct DependentsAttrsSetWithoutNeed {
    #[new(into)]
    package_path: Vec<String>,
}

impl fmt::Display for DependentsAttrsSetWithoutNeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writedoc!(
            f,
            "
            - {}: Has `meta.hasNoMaintainersButDependents = true` set, but it's not needed. Please unset the attribute
            ",
            self.package_path.join("."),
        )
    }
}
