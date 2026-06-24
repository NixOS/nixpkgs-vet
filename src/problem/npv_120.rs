use std::fmt;

use crate::gh_write::{Options, gh_write};
use derive_new::new;

#[derive(Clone, new)]
pub struct NixEvalError {
    #[new(into)]
    stderr: String,
}

impl fmt::Display for NixEvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.stderr)?;
        gh_write(
            f,
            "- Nix evaluation failed for some package in `pkgs/by-name`, see error above"
                .to_string(),
            Options {
                ..Default::default()
            },
        )
    }
}
