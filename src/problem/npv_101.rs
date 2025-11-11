use std::fmt;

use derive_new::new;

use crate::structure::{self, ByNameDir};

#[derive(Clone, new, Debug)]
pub struct ByNameNonDerivation {
    #[new(into)]
    attribute_name: String,
    by_name_dir: ByNameDir,
}

impl fmt::Display for ByNameNonDerivation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            attribute_name,
            by_name_dir,
        } = self;
        let relative_package_file =
            structure::relative_file_for_package(attribute_name, &by_name_dir.path);
        write!(
            f,
            "- {attribute_name}: This attribute defined by {relative_package_file} is not a derivation",
        )
    }
}
