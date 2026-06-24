use std::fmt;

use derive_new::new;

use crate::gh_write::{Options, gh_write};
use crate::structure;

#[derive(Clone, new)]
pub struct PackageDirectoryIsNotDirectory {
    #[new(into)]
    package_name: String,
}

impl fmt::Display for PackageDirectoryIsNotDirectory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { package_name } = self;
        let relative_package_dir = structure::relative_dir_for_package(package_name);
        gh_write(
            f,
            format!("- {relative_package_dir}: This path is a file, but it should be a directory."),
            Options {
                ..Default::default()
            },
        )
    }
}
