use itertools::Itertools;
use relative_path::RelativePathBuf;
use std::collections::BTreeSet;
use std::fmt;

use derive_new::new;
use indoc::writedoc;

#[derive(Clone, Debug, new)]
pub struct DependentsAttrsShouldBeSet {
    #[new(into)]
    package: String,
    #[new(into)]
    package_file: RelativePathBuf,
    #[new(into)]
    referenced_by_files: BTreeSet<RelativePathBuf>,
}

impl fmt::Display for DependentsAttrsShouldBeSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writedoc!(
            f,
            "
            - {} (defined by {}): `meta.hasNoMaintainersButDependents` should be set, because:
              - There are no maintainers, and
              - The package might be depended on by potentially {} files, including: {}
            ",
            self.package,
            self.package_file,
            self.referenced_by_files.len(),
            self.referenced_by_files.iter().take(5).join(", "),
        )
    }
}
