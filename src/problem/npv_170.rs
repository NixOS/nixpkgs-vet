use std::fmt;

use derive_new::new;
use relative_path::RelativePathBuf;

#[derive(Clone, new)]
pub struct ByNamePackegPrefixedWithNumber {
    #[new(into)]
    package_name: String,
    #[new(into)]
    relative_package_dir: RelativePathBuf,
}

impl fmt::Display for ByNamePackegPrefixedWithNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            package_name,
            relative_package_dir,
        } = self;
        write!(
            f,
            "- {relative_package_dir}: Attribute `{package_name}` should not be number-prefixed. Prefix with `_`, or wrap in quotes"
        )
    }
}
