use std::fmt;

use derive_new::new;
use indoc::formatdoc;
use relative_path::RelativePathBuf;

use crate::gh_write::{Options, gh_write};
use crate::location::Location;
use crate::structure;

use super::create_path_expr;

#[derive(Clone, new)]
pub struct ByNameOverrideContainsWrongCallPackagePath {
    #[new(into)]
    package_name: String,
    #[new(into)]
    actual_path: RelativePathBuf,
    location: Location,
}

impl fmt::Display for ByNameOverrideContainsWrongCallPackagePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            package_name,
            location,
            actual_path,
        } = self;
        let Location { file, line, column } = location;
        let expected_package_path = structure::relative_file_for_package(package_name);
        let expected_path_expr = create_path_expr(file, expected_package_path);
        let relative_package_dir = structure::relative_dir_for_package(package_name);
        let actual_path_expr = create_path_expr(file, actual_path);
        gh_write(
            f,
            formatdoc!("
            - Because {relative_package_dir} exists, the attribute `pkgs.{package_name}` must be defined like

                {package_name} = callPackage {expected_path_expr} {{ /* ... */ }};

              However, in this PR, the first `callPackage` argument is the wrong path. See the definition in {file}:{line}:

                {package_name} = callPackage {actual_path_expr} {{ /* ... */ }};
            "),
            Options {
                file: Some(file),
                start_line: Some(*line),
                start_col: Some(*column),
                ..Default::default()
            }
        )
    }
}
