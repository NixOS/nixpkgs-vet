use std::fmt;

use derive_new::new;
use indoc::writedoc;

use crate::location::Location;

#[derive(Clone, new)]
pub struct NixFileContainsUselessEscape {
    location: Location,
    current_escape: String,
    without_escape: String,
    fixed_escape: Option<String>,
}

impl fmt::Display for NixFileContainsUselessEscape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            location,
            current_escape,
            without_escape,
            fixed_escape,
        } = self;
        let fixed_escape_text = match fixed_escape {
            None => String::from("Change it to that."),
            Some(fixed_escape) => format!(
                "Depending on your intention, either change it to that, or to the non-equivalent \"{}\".",
                fixed_escape,
            ),
        };
        writedoc!(
            f,
            "
            - {}: line {}, column {} contains the escape \"{}\".
              This escape has no effect; it is equivalent to \"{}\".
              {}
            ",
            location.file,
            location.line,
            location.column,
            current_escape,
            without_escape,
            fixed_escape_text,
        )
    }
}
