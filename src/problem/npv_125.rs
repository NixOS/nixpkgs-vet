use std::fmt;

use crate::gh_write::{Options, gh_write};
use derive_new::new;
use relative_path::RelativePathBuf;

#[derive(Clone, new)]
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
        gh_write(
            f,
            format!(
                "- {relative_package_dir}: Path {subpath} is a symlink pointing to a path outside the directory of that package."
            ),
            Options {
                file: Some(&relative_package_dir.join(subpath)),
                ..Default::default()
            },
        )
    }
}
