use std::fmt;

use derive_new::new;

use crate::gh_write::{Options, gh_write};

#[derive(Clone, new)]
pub struct ByNameCannotDetermineAttributeLocation {
    #[new(into)]
    attribute_name: String,
}

impl fmt::Display for ByNameCannotDetermineAttributeLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { attribute_name } = self;
        gh_write(
            f,
            format!(
                "- pkgs.{attribute_name}: Cannot determine the location of this attribute using `builtins.unsafeGetAttrPos`.",
            ),
            Options {
                ..Default::default()
            },
        )
    }
}
