use std::sync::Arc;
use std::{fmt, io};

use crate::gh_write::{Options, gh_write};
use derive_new::new;
use relative_path::RelativePathBuf;

#[derive(Clone, new)]
pub struct NixFileContainsUnresolvablePath {
    #[new(into)]
    relative_package_dir: RelativePathBuf,
    #[new(into)]
    subpath: RelativePathBuf,
    line: usize,
    #[new(into)]
    text: String,
    #[new(into)]
    io_error: Arc<io::Error>,
}

impl fmt::Display for NixFileContainsUnresolvablePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            relative_package_dir,
            subpath,
            line,
            text,
            io_error,
        } = self;
        gh_write(
            f,
            format!(
                "- {relative_package_dir}: File {subpath} at line {line} contains the path expression \"{text}\" which cannot be resolved: {io_error}."
            ),
            Options {
                file: Some(&relative_package_dir.join(subpath)),
                start_line: Some(*line),
                ..Default::default()
            },
        )
    }
}
