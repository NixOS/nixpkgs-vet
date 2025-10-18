use std::fmt;

use derive_new::new;

use crate::structure::{self, ByNameDir};

#[derive(Clone, new, Debug)]
pub struct PackageDirectoryIsNotDirectory {
    #[new(into)]
    package_name: String,
    by_name_dir: ByNameDir,
}

impl fmt::Display for PackageDirectoryIsNotDirectory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            package_name,
            by_name_dir,
        } = self;
        let relative_package_dir =
            structure::relative_dir_for_package(package_name, &by_name_dir.path);
        write!(
            f,
            "- {relative_package_dir}: This path is a file, but it should be a directory.",
        )
    }
}
