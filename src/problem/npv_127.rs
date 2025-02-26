use relative_path::FromPathError;
use relative_path::RelativePathBuf;
use std::fmt;

use derive_new::new;

#[derive(Clone, new)]
pub struct PackageContainsInvalidReference {
    #[new(into)]
    relative_package_dir: RelativePathBuf,
    #[new(into)]
    subpath: RelativePathBuf,
    line: usize,
    #[new(into)]
    text: String,
    #[new(into)]
    err: FromPathError,
}

impl fmt::Display for PackageContainsInvalidReference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            relative_package_dir,
            subpath,
            line,
            text,
            err,
        } = self;
        write!(
            f,
            "- {relative_package_dir}: File {subpath} at line {line} contains the path expression \"{text}\" which is not valid: {err}.",
        )
    }
}
