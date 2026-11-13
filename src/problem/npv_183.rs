use std::fmt;

use derive_new::new;
use indoc::writedoc;

#[derive(Clone, Debug, new)]
pub struct RequiresMaintainersButNoMaintainers {
    #[new(into)]
    package_path: Vec<String>,
}
// requiresMaintainers = maintainers != []
// requiresMaintainers == false && hasNoMaintainersButDependents == true &&
impl fmt::Display for RequiresMaintainersButNoMaintainers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writedoc!(
            f,
            "
            - {}: Has `meta.requiresMaintainers = true` set, but no maintainers listed.
            ",
            self.package_path.join("."),
        )
    }
}
