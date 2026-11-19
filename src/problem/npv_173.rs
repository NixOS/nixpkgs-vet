use std::fmt;

use derive_new::new;
use indoc::writedoc;

#[derive(Clone, Debug, new)]
pub struct TopLevelPackageAddedCrossPackageRef {
    #[new(into)]
    package_name: String,
}

impl fmt::Display for TopLevelPackageAddedCrossPackageRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { package_name } = self;
        writedoc!(
            f,
            "
            - Attribute `{package_name}` was previously had no references to other package sets (pkgsStatic, pkgsCross, ...), but now does.
            Please rearrange it so that it does not.
            ",
        )
    }
}
