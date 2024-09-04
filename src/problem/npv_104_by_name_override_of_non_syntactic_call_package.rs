use std::fmt;

use indoc::writedoc;
use relative_path::RelativePathBuf;

use crate::eval::Location;
use crate::structure;

use super::{create_path_expr, indent_definition, ByNameOverrideError, ByNameOverrideErrorKind};

#[derive(Clone)]
pub struct ByNameOverrideOfNonSyntacticCallPackage(ByNameOverrideError);

impl ByNameOverrideOfNonSyntacticCallPackage {
    pub fn new(
        attribute_name: impl Into<String>,
        file: impl Into<RelativePathBuf>,
        location: impl Into<Location>,
        definition: impl Into<String>,
    ) -> Self {
        let attribute_name = attribute_name.into();
        let expected_package_path = structure::relative_file_for_package(&attribute_name);
        let location = location.into();
        Self(ByNameOverrideError {
            package_name: attribute_name,
            expected_package_path: expected_package_path,
            file: file.into(),
            line: location.line,
            column: location.column,
            definition: definition.into(),
            kind: ByNameOverrideErrorKind::NonSyntacticCallPackage,
        })
    }
}

impl fmt::Display for ByNameOverrideOfNonSyntacticCallPackage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self(ByNameOverrideError {
            package_name,
            expected_package_path,
            file,
            line,
            column,
            definition,
            ..
        }) = self;
        let relative_package_dir = structure::relative_dir_for_package(package_name);
        let expected_path_expr = create_path_expr(file, expected_package_path);
        let indented_definition = indent_definition(*column, definition.clone());

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
