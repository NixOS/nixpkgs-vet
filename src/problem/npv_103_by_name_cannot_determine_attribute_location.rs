use std::fmt;

#[derive(Clone)]
pub struct ByNameCannotDetermineAttributeLocation {
    attribute_name: String,
}

impl ByNameCannotDetermineAttributeLocation {
    pub fn new(attribute_name: impl Into<String>) -> Self {
        Self {
            attribute_name: attribute_name.into(),
        }
    }
}

impl fmt::Display for ByNameCannotDetermineAttributeLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { attribute_name } = self;
        write!(
            f,
            "- pkgs.{attribute_name}: Cannot determine the location of this attribute using `builtins.unsafeGetAttrPos`.",
        )
    }
}
