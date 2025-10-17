use std::fmt;

use derive_new::new;
use relative_path::RelativePath;

use crate::structure;

#[derive(Clone, new, Debug)]
pub struct ByNameNonDerivation {
    #[new(into)]
    attribute_name: String,
}

impl fmt::Display for ByNameNonDerivation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { attribute_name } = self;
        let relative_package_file =
            structure::relative_file_for_package(attribute_name, RelativePath::new("pkgs/by-name"));
        write!(
            f,
            "- pkgs.{attribute_name}: This attribute defined by {relative_package_file} is not a derivation",
        )
    }
}
