use std::fmt;

#[derive(Clone)]
pub struct ByNameInternalCallPackageUsed {
    attribute_name: String,
}

impl ByNameInternalCallPackageUsed {
    pub fn new(attribute_name: impl Into<String>) -> Self {
        Self {
            attribute_name: attribute_name.into(),
        }
    }
}

impl fmt::Display for ByNameInternalCallPackageUsed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { attribute_name } = self;
        write!(
            f,
            "- pkgs.{attribute_name}: This attribute is defined using `_internalCallByNamePackageFile`, which is an internal function not intended for manual use.",
        )
    }
}
