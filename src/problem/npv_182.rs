use std::fmt;

use derive_new::new;
use indoc::writedoc;

#[derive(Clone, Debug, new)]
pub struct DependentsAttrsSetWithoutDependents {
    #[new(into)]
    package_path: Vec<String>,
}

impl fmt::Display for DependentsAttrsSetWithoutDependents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writedoc!(
            f,
            "
            - {}: Has `meta.hasNoMaintainersButDependents = true` set, but there are no
            dependents. Please unset the attribute
            ",
            self.package_path.join("."),
        )
    }
}
