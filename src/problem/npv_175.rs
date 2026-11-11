use std::fmt;

use derive_new::new;
use indoc::writedoc;

use crate::location::Location;

#[derive(Clone, Debug, new)]
pub struct StructuredAttrsFlagsArray {
    #[new(into)]
    package_name: String,
    #[new(into)]
    variable: String,
    location: Location,
}

impl fmt::Display for StructuredAttrsFlagsArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            package_name,
            variable,
            location,
        } = self;
        let replacement = variable.strip_suffix("Array").expect("FlagsArray suffix");
        writedoc!(
            f,
            "
            - Attribute `{package_name}` introduces a use of `{variable}` with `__structuredAttrs = true` in {}: line {}, column {}.
              Please use `{replacement}` instead; it is already a Bash array with structured attributes.
            ",
            location.file,
            location.line,
            location.column,
        )
    }
}
