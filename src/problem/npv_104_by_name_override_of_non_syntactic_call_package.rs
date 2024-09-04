use std::fmt;

use indoc::writedoc;

use crate::location::Location;
use crate::structure;

use super::{create_path_expr, indent_definition};

#[derive(Clone)]
pub struct ByNameOverrideOfNonSyntacticCallPackage {
    package_name: String,
    location: Location,
    definition: String,
}

impl ByNameOverrideOfNonSyntacticCallPackage {
    pub fn new(
        attribute_name: impl Into<String>,
        location: impl Into<Location>,
        definition: impl Into<String>,
    ) -> Self {
        Self {
            package_name: attribute_name.into(),
            location: location.into(),
            definition: definition.into(),
        }
    }
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

        writedoc!(
            f,
            "
            - Because {relative_package_dir} exists, the attribute `pkgs.{package_name}` must be defined like

                {package_name} = callPackage {expected_path_expr} {{ /* ... */ }};

              However, in this PR, it isn't defined that way. See the definition in {file}:{line}

            {indented_definition}
            ",
        )
    }
}
