use std::sync::Arc;
use std::{fmt, io};

use derive_new::new;
use relative_path::RelativePathBuf;

#[derive(Clone, new, Debug)]
pub struct PackageContainsUnresolvableSymlink {
    #[new(into)]
    relative_package_dir: RelativePathBuf,
    #[new(into)]
    subpath: RelativePathBuf,
    #[new(into)]
    io_error: Arc<io::Error>,
}

impl fmt::Display for PackageContainsUnresolvableSymlink {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            relative_package_dir,
            subpath,
            io_error,
        } = self;
        write!(
            f,
            "- {relative_package_dir}: Path {subpath} is a symlink which cannot be resolved: {io_error}.",
        )
    }
}
