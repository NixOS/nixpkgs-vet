use std::fmt;

use crate::structure;

use super::{ByNameError, ByNameErrorKind};

#[derive(Clone)]
pub struct ByNameUndefinedAttribute(ByNameError);

impl ByNameUndefinedAttribute {
    pub fn new(attribute_name: impl Into<String>) -> Self {
        Self(ByNameError {
            attribute_name: attribute_name.into(),
            kind: ByNameErrorKind::UndefinedAttr,
        })
    }
}

impl fmt::Display for ByNameUndefinedAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self(ByNameError { attribute_name, .. }) = self;
        let relative_package_file = structure::relative_file_for_package(attribute_name);
        write!(
            f,
            "- pkgs.{attribute_name}: This attribute is not defined but it should be defined automatically as {relative_package_file}",
        )
    }
}
