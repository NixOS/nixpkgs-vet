use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::{RelativePath, RelativePathBuf};

use crate::location::Location;
use crate::structure;

use super::create_path_expr;

#[derive(Clone, new, Debug)]
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
        let Location { file, line, .. } = location;
        let expected_package_path =
            structure::relative_file_for_package(package_name, RelativePath::new("pkgs/by-name"));
        let expected_path_expr = create_path_expr(file, expected_package_path);
        let relative_package_dir =
            structure::relative_dir_for_package(package_name, RelativePath::new("pkgs/by-name"));
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
