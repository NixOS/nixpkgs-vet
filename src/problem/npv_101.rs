use std::fmt;

use derive_new::new;
use relative_path::RelativePathBuf;

use crate::structure;

#[derive(Clone, Debug, new)]
pub struct ByNameNonDerivation {
    #[new(into)]
    by_name_subpath: RelativePathBuf,
    #[new(into)]
    full_attr_path: Vec<String>,
}

impl fmt::Display for ByNameNonDerivation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            by_name_subpath,
            full_attr_path,
        } = self;
        let relative_package_file =
            // we should be able to safely unwrap here because the full_attr_path should always contain at least the attribute name
            structure::relative_file_for_package(by_name_subpath, full_attr_path.last().unwrap());
        write!(
            f,
            "- {}: This attribute defined by {relative_package_file} is not a derivation",
            full_attr_path.join(".")
        )
    }
}
