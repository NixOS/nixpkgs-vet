use std::fmt;

use derive_new::new;

use crate::structure::ByNameDir;

#[derive(Clone, new, Debug)]
pub struct NixEvalError {
    #[new(into)]
    stderr: String,
    by_name_dir: ByNameDir,
}

impl fmt::Display for NixEvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.stderr)?;
        write!(
            f,
            "- Nix evaluation failed for some package in `{}`, see error above",
            &self.by_name_dir.path
        )
    }
}
