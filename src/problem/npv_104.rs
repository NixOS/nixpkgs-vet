use std::fmt;

use derive_new::new;
use indoc::formatdoc;

use super::{create_path_expr, indent_definition};
use crate::gh_write::{Options, gh_write};
use crate::location::Location;
use crate::structure;

#[derive(Clone, new)]
pub struct ByNameOverrideOfNonSyntacticCallPackage {
    #[new(into)]
    package_name: String,
    location: Location,
    #[new(into)]
    definition: String,
}

impl fmt::Display for ByNameOverrideOfNonSyntacticCallPackage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            package_name,
            location,
            definition,
        } = self;
        let Location { file, line, column } = location;
        let expected_package_path = structure::relative_file_for_package(package_name);
        let relative_package_dir = structure::relative_dir_for_package(package_name);
        let expected_path_expr = create_path_expr(file, expected_package_path);
        let indented_definition = indent_definition(*column, definition);

        gh_write(
            f,
            formatdoc!(
                "
                - Because {relative_package_dir} exists, the attribute `pkgs.{package_name}` must be defined like

                    {package_name} = callPackage {expected_path_expr} {{ /* ... */ }};

                  However, in this PR, it isn't defined that way. See the definition in {file}:{line}

                {indented_definition}
                ",
            ),
            Options {
                file: Some(file),
                start_line: Some(*line),
                start_col: Some(*column),
                ..Default::default()
            }
        )
    }
}
