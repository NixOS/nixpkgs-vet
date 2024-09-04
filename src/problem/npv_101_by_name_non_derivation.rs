use std::fmt;

use crate::structure;

#[derive(Clone)]
pub struct ByNameNonDerivation {
    attribute_name: String,
}

impl ByNameNonDerivation {
    pub fn new(attribute_name: impl Into<String>) -> Self {
        Self {
            attribute_name: attribute_name.into(),
        }
    }
}

impl fmt::Display for ByNameNonDerivation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { attribute_name } = self;
        let relative_package_file = structure::relative_file_for_package(attribute_name);
        write!(
            f,
            "- pkgs.{attribute_name}: This attribute defined by {relative_package_file} is not a derivation",
        )
    }
}
