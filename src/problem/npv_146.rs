use std::fmt;

use crate::gh_write::{Options, gh_write};
use derive_new::new;
use relative_path::RelativePathBuf;

#[derive(Clone, new)]
pub struct NixFileHasShebangButNotExecutable {
    #[new(into)]
    relative_path: RelativePathBuf,
}

impl fmt::Display for NixFileHasShebangButNotExecutable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { relative_path } = self;
        gh_write(
            f,
            format!("- {relative_path}: Nix files with a shebang (`#!`) line must be executable."),
            Options {
                file: Some(relative_path),
                ..Default::default()
            },
        )
    }
}
