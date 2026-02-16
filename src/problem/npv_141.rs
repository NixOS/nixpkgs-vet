use std::fmt;

use derive_new::new;
use relative_path::RelativePathBuf;

#[derive(Clone, new)]
pub struct InvalidPackageDirectoryName {
    #[new(into)]
    package_name: String,
    #[new(into)]
    relative_package_dir: RelativePathBuf,
}

impl fmt::Display for InvalidPackageDirectoryName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            package_name,
            relative_package_dir,
        } = self;
        write!(
            f,
            "- {relative_package_dir}: Invalid package directory name \"{package_name}\", must start with a letter (a-z, A-Z) or \"_\", followed by ASCII characters a-z, A-Z, 0-9, \"-\" or \"_\".",
        )
    }
}
