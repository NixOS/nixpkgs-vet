use std::fmt;

use derive_new::new;
use indoc::writedoc;

use crate::location::Location;
use crate::structure;

use super::{create_path_expr, indent_definition};

#[derive(Clone, new)]
pub struct ByNameOverrideContainsEmptyArgument {
    #[new(into)]
    package_name: String,
    location: Location,
    #[new(into)]
    definition: String,
}

impl fmt::Display for ByNameOverrideContainsEmptyArgument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            package_name,
            location,
            definition,
        } = self;
        let Location { file, line, column } = location;
        let expected_package_path = structure::relative_file_for_package(package_name);
        let expected_path_expr = create_path_expr(file, expected_package_path);
        let relative_package_dir = structure::relative_dir_for_package(package_name);
        let indented_definition = indent_definition(*column, definition);

        writedoc!(
            f,
            "
            - Because {relative_package_dir} exists, the attribute `pkgs.{package_name}` must be defined like

                {package_name} = callPackage {expected_path_expr} {{ /* ... */ }};

              However, in this PR, the second argument is empty. See the definition in {file}:{line}:

            {indented_definition}

              Such a definition is provided automatically and therefore not necessary. Please remove it.
            ",
        )
    }
}
