use std::fmt;

use derive_new::new;

#[derive(Clone, new)]
pub struct NixEvalError {
    #[new(into)]
    by_name_subpath: String,
    #[new(into)]
    stderr: String,
}

impl fmt::Display for NixEvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.stderr)?;
        write!(
            f,
            "- Nix evaluation failed for some package in `{}`, see error above",
            self.by_name_subpath,
        )
    }
}
