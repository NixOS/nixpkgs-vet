use std::fmt;

use crate::gh_write::{Options, gh_write};
use derive_new::new;
use relative_path::RelativePathBuf;

#[derive(Clone, new)]
pub struct NixFileContainsHomeRelativePath {
    #[new(into)]
    relative_package_dir: RelativePathBuf,
    #[new(into)]
    subpath: RelativePathBuf,
    line: usize,
    #[new(into)]
    text: String,
}

impl fmt::Display for NixFileContainsHomeRelativePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            relative_package_dir,
            subpath,
            line,
            text,
        } = self;
        gh_write(
            f,
            format!(
                "- {relative_package_dir}: File {subpath} at line {line} contains the home-relative path expression \"{text}\", which is not allowed in nixpkgs."
            ),
            Options {
                file: Some(&relative_package_dir.join(subpath)),
                start_line: Some(*line),
                ..Default::default()
            },
        )
    }
}
