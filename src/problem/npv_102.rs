use std::fmt;

use derive_new::new;

#[derive(Clone, new)]
pub struct ByNameInternalCallPackageUsed {
    #[new(into)]
    attribute_name: String,
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
