use std::fmt;

use derive_new::new;

use crate::gh_write::{Options, gh_write};
use crate::structure;

#[derive(Clone, new)]
pub struct ByNameUndefinedAttribute {
    #[new(into)]
    attribute_name: String,
}

impl fmt::Display for ByNameUndefinedAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { attribute_name } = self;
        let relative_package_file = structure::relative_file_for_package(attribute_name);
        gh_write(
            f,
            format!(
                "- pkgs.{attribute_name}: This attribute is not defined but it should be defined automatically as {relative_package_file}"
            ),
            Options {
                file: Some(&relative_package_file),
                ..Default::default()
            },
        )
    }
}
