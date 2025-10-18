use std::fmt;

use derive_new::new;

use crate::structure::{self, ByNameDir};

#[derive(Clone, new, Debug)]
pub struct ByNameUndefinedAttribute {
    #[new(into)]
    attribute_name: String,
    by_name_dir: ByNameDir,
}

impl fmt::Display for ByNameUndefinedAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            attribute_name,
            by_name_dir,
        } = self;
        let relative_package_file =
            structure::relative_file_for_package(attribute_name, &by_name_dir.path);
        write!(
            f,
            "- {attribute_name}: This attribute is not defined but it should be defined automatically as {}",
            relative_package_file
                .into_string()
                .replace(&(by_name_dir.unversioned_attr_prefix.clone() + "."), "")
        )
    }
}
