use std::fmt;

use derive_new::new;
use indoc::writedoc;

use crate::location::Location;

#[derive(Clone, Debug, new)]
pub struct NixFileContainsOptionalList {
    location: Location,
    singular: &'static str,
    plural: &'static str,
}

impl fmt::Display for NixFileContainsOptionalList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            location,
            singular,
            plural,
        } = self;
        writedoc!(
            f,
            "
            - {}: line {}, column {} conditionally includes a list with `{singular}`.
              Use `{plural}` when conditionally including a list.
            ",
            location.file,
            location.line,
            location.column,
        )
    }
}
