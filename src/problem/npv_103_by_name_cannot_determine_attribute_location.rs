use std::fmt;

use super::{ByNameError, ByNameErrorKind};

#[derive(Clone)]
pub struct ByNameCannotDetermineAttributeLocation(ByNameError);

impl ByNameCannotDetermineAttributeLocation {
    pub fn new(attribute_name: impl Into<String>) -> Self {
        Self(ByNameError {
            attribute_name: attribute_name.into(),
            kind: ByNameErrorKind::CannotDetermineAttributeLocation,
        })
    }
}

impl fmt::Display for ByNameCannotDetermineAttributeLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self(ByNameError { attribute_name, .. }) = self;
        write!(
            f,
            "- pkgs.{attribute_name}: Cannot determine the location of this attribute using `builtins.unsafeGetAttrPos`.",
        )
    }
}
