use std::fmt;

use crate::structure;

#[derive(Clone)]
pub struct ByNameUndefinedAttribute {
    attribute_name: String,
}

impl ByNameUndefinedAttribute {
    pub fn new(attribute_name: impl Into<String>) -> Self {
        Self {
            attribute_name: attribute_name.into(),
        }
    }
}

impl fmt::Display for ByNameUndefinedAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { attribute_name } = self;
        let relative_package_file = structure::relative_file_for_package(attribute_name);
        write!(
            f,
            "- pkgs.{attribute_name}: This attribute is not defined but it should be defined automatically as {relative_package_file}",
        )
    }
}
