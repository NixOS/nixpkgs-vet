use std::fmt;

use indoc::writedoc;
use relative_path::RelativePathBuf;

use crate::location::Location;
use crate::structure;

use super::create_path_expr;

#[derive(Clone)]
pub struct ByNameOverrideContainsWrongCallPackagePath {
    package_name: String,
    actual_path: RelativePathBuf,
    location: Location,
}

impl ByNameOverrideContainsWrongCallPackagePath {
    pub fn new(
        package_name: impl Into<String>,
        actual_path: impl Into<RelativePathBuf>,
        location: impl Into<Location>,
    ) -> Self {
        Self {
            package_name: package_name.into(),
            actual_path: actual_path.into(),
            location: location.into(),
        }
    }
}

impl fmt::Display for ByNameOverrideContainsWrongCallPackagePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            package_name,
            location,
            actual_path,
        } = self;
        let Location { file, line, .. } = location;
        let expected_package_path = structure::relative_file_for_package(package_name);
        let expected_path_expr = create_path_expr(file, expected_package_path);
        let relative_package_dir = structure::relative_dir_for_package(package_name);
        let actual_path_expr = create_path_expr(file, actual_path);
        writedoc!(
            f,
            "
            - Because {relative_package_dir} exists, the attribute `pkgs.{package_name}` must be defined like

                {package_name} = callPackage {expected_path_expr} {{ /* ... */ }};

              However, in this PR, the first `callPackage` argument is the wrong path. See the definition in {file}:{line}:

                {package_name} = callPackage {actual_path_expr} {{ /* ... */ }};
            ",
        )
    }
}
