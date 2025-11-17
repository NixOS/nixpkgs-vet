use std::fmt;

use derive_new::new;
use relative_path::RelativePathBuf;

use crate::structure::{self, ByNameDir};

#[derive(Clone, new, Debug)]
pub struct PackageInWrongShard {
    #[new(into)]
    package_name: String,
    #[new(into)]
    relative_package_dir: RelativePathBuf,
    by_name_dir: ByNameDir,
}

impl fmt::Display for PackageInWrongShard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            package_name,
            relative_package_dir,
            by_name_dir,
        } = self;
        let correct_relative_package_dir =
            structure::relative_dir_for_package(package_name, &by_name_dir.path);
        write!(
            f,
            "- {relative_package_dir}: Incorrect directory location, should be {correct_relative_package_dir} instead.",
        )
    }
}
