use std::fmt;

use derive_new::new;
use indoc::writedoc;

use crate::location::Location;

/// One `"`-string that spells out an executable path by hand, and the call that replaces it.
#[derive(Clone, Debug)]
pub struct ExecutableReference {
    pub location: Location,
    /// The source text of the string with whitespace collapsed.
    pub text: String,
    pub fix: ExecutableReferenceFix,
}

#[derive(Clone, Debug)]
pub enum ExecutableReferenceFix {
    /// `"${pkg}/bin/${pkg.meta.mainProgram}"` should be `lib.getExe pkg`
    GetExe { package: String },
    /// `"${pkg}/bin/name"` should be `lib.getExe' pkg "name"`
    GetExePrime { package: String, program: String },
    /// `"${lib.getExe pkg}"` should be `lib.getExe pkg`
    Unwrap { call: String },
}

#[derive(Clone, Debug, new)]
pub struct NixFileContainsExecutablePath {
    reference: ExecutableReference,
    /// Instances of the same string in the base version of the file, which stay allowed.
    base_count: usize,
    head_count: usize,
}

impl fmt::Display for NixFileContainsExecutablePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ExecutableReference {
            location,
            text,
            fix,
        } = &self.reference;
        write!(
            f,
            "- {}: line {}, column {} ",
            location.file, location.line, location.column
        )?;
        match fix {
            ExecutableReferenceFix::GetExe { package } => writedoc!(
                f,
                "
                introduces the executable path {text}.
                  Use `lib.getExe {package}` instead.
                "
            ),
            ExecutableReferenceFix::GetExePrime { package, program } => writedoc!(
                f,
                "
                introduces the executable path {text}.
                  Use `lib.getExe' {package} \"{program}\"` instead, or `lib.getExe {package}` if \"{program}\" is its `meta.mainProgram`.
                "
            ),
            ExecutableReferenceFix::Unwrap { call } => writedoc!(
                f,
                "
                wraps `{call}` in a string.
                  `{call}` already is a string, so use it directly without the surrounding \"${{ }}\".
                "
            ),
        }?;

        // the reported location may be a grandfathered instance rather than the added one
        let base_count = self.base_count;
        if base_count == 0 {
            return Ok(());
        }
        let plural = if base_count == 1 { "" } else { "s" };
        let verb = if base_count == 1 { "stays" } else { "stay" };
        writeln!(
            f,
            "  This file had {base_count} instance{plural} of this string before, which {verb} allowed, but now has {}.",
            self.head_count
        )
    }
}
