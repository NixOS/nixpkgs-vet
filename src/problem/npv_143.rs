use std::fmt;

use derive_new::new;
use relative_path::RelativePath;

use crate::structure::{self, PACKAGE_NIX_FILENAME};

#[derive(Clone, new, Debug)]
pub struct PackageNixMissing {
    #[new(into)]
    package_name: String,
}

impl fmt::Display for PackageNixMissing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { package_name } = self;
        let relative_package_dir =
            structure::relative_dir_for_package(package_name, RelativePath::new("pkgs/by-name"));
        write!(
            f,
            "- {relative_package_dir}: Missing required \"{PACKAGE_NIX_FILENAME}\" file.",
        )
    }
}
