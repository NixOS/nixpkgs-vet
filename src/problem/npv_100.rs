use std::fmt;

use derive_new::new;

use crate::structure;

#[derive(Clone, new)]
pub struct ByNameUndefinedAttribute {
    #[new(into)]
    by_name_subpath: String,
    #[new(into)]
    attribute_name: String,
}

impl fmt::Display for ByNameUndefinedAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            by_name_subpath,
            attribute_name,
        } = self;
        let relative_package_file =
            structure::relative_file_for_package(by_name_subpath, attribute_name);
        write!(
            f,
            "- pkgs.{attribute_name}: This attribute is not defined but it should be defined automatically as {relative_package_file}",
        )
    }
}
