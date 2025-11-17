use std::fmt;

use derive_new::new;

#[derive(Clone, new, Debug)]
pub struct ByNameCannotDetermineAttributeLocation {
    #[new(into)]
    attribute_name: String,
}

impl fmt::Display for ByNameCannotDetermineAttributeLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { attribute_name } = self;
        write!(
            f,
            "- {attribute_name}: Cannot determine the location of this attribute using `builtins.unsafeGetAttrPos`.",
        )
    }
}
