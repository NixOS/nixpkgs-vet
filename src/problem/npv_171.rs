use std::fmt;

use derive_new::new;
use indoc::writedoc;

use crate::location::Location;

#[derive(Clone, new)]
pub struct MutableGitHubFetchpatch {
    location: Location,
    current: String,
    fixed: String,
}

impl fmt::Display for MutableGitHubFetchpatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            location,
            current,
            fixed,
        } = self;
        writedoc!(
            f,
            "
            - {}: line {}, column {} has the following patch URL: \"{}\".
              This is a problem because the number of characters in the included commit hashes can change at any time, causing the fixed-output derivation to fail.
              To fix this, change the URL to \"{}\", and update the hash to its new value.
            ",
            location.file,
            location.line,
            location.column,
            current,
            fixed,
        )
    }
}
