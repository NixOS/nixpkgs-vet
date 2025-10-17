use std::fmt;

use derive_new::new;
use relative_path::RelativePathBuf;

#[derive(Clone, new, Debug)]
pub struct PackageContainsSymlinkPointingOutside {
    #[new(into)]
    relative_package_dir: RelativePathBuf,
    #[new(into)]
    subpath: RelativePathBuf,
}

impl fmt::Display for PackageContainsSymlinkPointingOutside {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            relative_package_dir,
            subpath,
        } = self;
        write!(
            f,
            "- {relative_package_dir}: Path {subpath} is a symlink pointing to a path outside the directory of that package.",
        )
    }
}
