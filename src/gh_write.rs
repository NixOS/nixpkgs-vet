use std::{env, fmt};

use relative_path::RelativePath;

#[derive(Default, Debug)]
pub enum Severity {
    #[default]
    Error,
    // These aren't used but are included for completeness.
    #[allow(dead_code)]
    Warning,
    #[allow(dead_code)]
    Notice,
}

#[derive(Default, Debug)]
pub struct Options<'a> {
    pub file: Option<&'a RelativePath>,
    pub start_line: Option<usize>, // 1-indexed
    pub start_col: Option<usize>,  // 1-indexed
    pub end_line: Option<usize>,   // 1-indexed
    pub end_col: Option<usize>,    // 1-indexed
    pub title: Option<&'a str>,
    pub severity: Severity,
}

/// Write the formatted string "input" to "f", using the specified GitHub workflow commands
/// See https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands
pub fn gh_write(
    f: &mut impl fmt::Write,
    input: String,
    options: Options,
) -> Result<(), fmt::Error> {
    // Guess at final needed amount
    let result: String = match env::var_os("GITHUB_ACTIONS") {
        None => input,
        Some(..) => {
            let mut result = String::with_capacity(input.len() + 128);
            match options.severity {
                Severity::Notice => result.push_str("::notice"),
                Severity::Warning => result.push_str("::warning"),
                Severity::Error => result.push_str("::error"),
            };

            let mut params = vec![];
            if let Some(title) = options.title {
                params.push(format!("title={}", escape_property(title.to_string())))
            }
            if let Some(file) = options.file {
                params.push(format!("file={}", escape_property(file.to_string())))
            }
            if let Some(start_line) = options.start_line {
                params.push(format!("line={start_line}"))
            }
            if let Some(start_col) = options.start_col {
                params.push(format!("col={start_col}"))
            }
            if let Some(end_line) = options.end_line {
                params.push(format!("endLine={end_line}"))
            }
            if let Some(end_col) = options.end_col {
                params.push(format!("endColumn={end_col}"))
            }

            if !params.is_empty() {
                result.push_str(&format!(" {}", params.join(",")));
            }

            result.push_str("::");
            result.push_str(&escape_data(input));
            result
        }
    };

    write!(f, "{}", result)
}

/// See https://github.com/actions/toolkit/blob/f31c2921c1228a97be08cdb38b919a83077354d9/packages/core/src/command.ts#L103-L117
fn escape_data(input: String) -> String {
    input
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

/// See https://github.com/actions/toolkit/blob/f31c2921c1228a97be08cdb38b919a83077354d9/packages/core/src/command.ts#L103-L117
fn escape_property(input: String) -> String {
    input
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
        .replace(':', "%3A")
        .replace(',', "%2C")
}

#[cfg(test)]
mod tests {
    use crate::gh_write::{Options, gh_write};
    use pretty_assertions::assert_str_eq;
    use relative_path::RelativePath;

    fn test_gh_write(input: &str, options: Options, expected: impl ToString) {
        let mut actual = String::new();
        temp_env::with_var("GITHUB_ACTIONS", Some("1"), || {
            gh_write(&mut actual, input.to_string(), options).unwrap()
        });
        assert_str_eq!(expected.to_string(), actual);
    }

    #[test]
    fn simple_error_default() {
        test_gh_write(
            "hi",
            Options {
                ..Default::default()
            },
            "::error::hi",
        );
    }

    #[test]
    fn multiline_error_with_file_and_line() {
        test_gh_write(
            indoc::indoc!("
            - Because pkgs/by-name/fo/foo exists, the attribute `pkgs.foo` must be defined like

                foo = callPackage ./../by-name/fo/foo/package.nix { /* ... */ };

            However, in this PR, it isn't defined that way. See the definition in pkgs/top-level/all-packages.nix:4

                foo = self.bar;
            (https://github.com/NixOS/nixpkgs-vet/wiki/NPV-104)"),
            Options {
                file: Some(RelativePath::new("pkgs/top-level/all-packages.nix")),
                start_line: Some(4),
                ..Default::default()
            },
            "::error file=pkgs/top-level/all-packages.nix,line=4::".to_owned() +
                "- Because pkgs/by-name/fo/foo exists, the attribute `pkgs.foo` must be defined like%0A%0A" +
                "    foo = callPackage ./../by-name/fo/foo/package.nix { /* ... */ };%0A%0A" +
                "However, in this PR, it isn't defined that way. " +
                "See the definition in pkgs/top-level/all-packages.nix:4%0A%0A" +
                "    foo = self.bar;%0A" +
                "(https://github.com/NixOS/nixpkgs-vet/wiki/NPV-104)",
        );
    }
}
