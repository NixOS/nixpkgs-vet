use std::fmt;

use derive_new::new;
use indoc::writedoc;

use crate::structure;
use crate::{location::Location, structure::ByNameDir};

use super::{create_path_expr, indent_definition};

#[derive(Clone, new, Debug)]
pub struct ByNameOverrideOfNonTopLevelPackage {
    #[new(into)]
    package_name: String,
    location: Location,
    #[new(into)]
    definition: String,
    by_name_dir: ByNameDir,
}

impl fmt::Display for ByNameOverrideOfNonTopLevelPackage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            package_name,
            location,
            definition,
            by_name_dir,
        } = self;
        let Location { file, line, column } = location;
        let relative_package_dir =
            structure::relative_dir_for_package(package_name, &by_name_dir.path);
        let expected_package_path =
            structure::relative_file_for_package(package_name, &by_name_dir.path);
        let expected_path_expr = create_path_expr(file, expected_package_path);
        let indented_definition = indent_definition(*column, definition);

        writedoc!(
            f,
            "
            - Because {relative_package_dir} exists, the attribute `{package_name}` must be defined like

                {package_name} = callPackage {expected_path_expr} {{ /* ... */ }};

              However, in this PR, a different `callPackage` is used. See the definition in {file}:{line}:

            {indented_definition}
            ",
        )
    }
}
