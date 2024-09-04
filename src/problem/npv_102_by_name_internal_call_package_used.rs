use std::fmt;

use super::{ByNameError, ByNameErrorKind};

#[derive(Clone)]
pub struct ByNameInternalCallPackageUsed(ByNameError);

impl ByNameInternalCallPackageUsed {
    pub fn new(attribute_name: impl Into<String>) -> Self {
        Self(ByNameError {
            attribute_name: attribute_name.into(),
            kind: ByNameErrorKind::InternalCallPackageUsed,
        })
    }
}

impl fmt::Display for ByNameInternalCallPackageUsed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self(ByNameError { attribute_name, .. }) = self;
        write!(
            f,
            "- pkgs.{attribute_name}: This attribute is defined using `_internalCallByNamePackageFile`, which is an internal function not intended for manual use.",
        )
    }
}
