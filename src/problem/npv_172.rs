use std::fmt;

use derive_new::new;
use indoc::writedoc;

#[derive(Clone, Debug, new)]
pub struct NewTopLevelPackageShouldNotRefOtherPackageSet {
    #[new(into)]
    package_name: String,
}

impl fmt::Display for NewTopLevelPackageShouldNotRefOtherPackageSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { package_name } = self;
        writedoc!(
            f,
            "
            - Attribute `{package_name}` is a new top-level package referencing other package sets (pkgsStatic, pkgsCross, ...).
            Rather than doing this, the package should either have a derivation that allows being called via `pgksStatic.{package_name}`,
            or if defining plugins or other items, be combined in a passthru attribute.
            ",
        )
    }
}
