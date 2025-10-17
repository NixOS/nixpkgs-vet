use std::fmt;

use derive_new::new;
use relative_path::{RelativePath, RelativePathBuf};

use crate::structure;

#[derive(Clone, new, Debug)]
pub struct PackageInWrongShard {
    #[new(into)]
    package_name: String,
    #[new(into)]
    relative_package_dir: RelativePathBuf,
}

impl fmt::Display for PackageInWrongShard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            package_name,
            relative_package_dir,
        } = self;
        let correct_relative_package_dir =
            structure::relative_dir_for_package(package_name, RelativePath::new("pkgs/by-name"));
        write!(
            f,
            "- {relative_package_dir}: Incorrect directory location, should be {correct_relative_package_dir} instead.",
        )
    }
}
