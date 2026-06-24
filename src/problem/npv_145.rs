use std::fmt;

use crate::gh_write::{Options, gh_write};
use derive_new::new;
use relative_path::RelativePathBuf;

#[derive(Clone, new)]
pub struct NixFileIsExecutableWithoutShebang {
    #[new(into)]
    relative_path: RelativePathBuf,
}

impl fmt::Display for NixFileIsExecutableWithoutShebang {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { relative_path } = self;
        gh_write(
            f,
            format!(
                "- {relative_path}: Nix files must not be executable unless they have a shebang (`#!`) line."
            ),
            Options {
                file: Some(relative_path),
                ..Default::default()
            },
        )
    }
}
