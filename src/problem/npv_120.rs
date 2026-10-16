use std::fmt;

use derive_new::new;
use relative_path::RelativePathBuf;

#[derive(Clone, Debug, new)]
pub struct NixEvalError {
    #[new(into)]
    by_name_subpath: RelativePathBuf,
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
