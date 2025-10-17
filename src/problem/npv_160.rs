use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::{RelativePath, RelativePathBuf};

use crate::structure;

#[derive(Clone, new, Debug)]
pub struct TopLevelPackageMovedOutOfByName {
    #[new(into)]
    package_name: String,
    #[new(into)]
    call_package_path: Option<RelativePathBuf>,
    #[new(into)]
    file: RelativePathBuf,
}

impl fmt::Display for TopLevelPackageMovedOutOfByName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            package_name,
            call_package_path,
            file,
        } = self;
        let relative_package_file =
            structure::relative_file_for_package(package_name, RelativePath::new("pkgs/by-name"));
        let call_package_arg = call_package_path
            .as_ref()
            .map_or_else(|| "...".into(), |path| format!("./{}", path));
        writedoc!(
            f,
            "
            - Attribute `pkgs.{package_name}` was previously defined in {relative_package_file}, but is now manually defined as `callPackage {call_package_arg} {{ /* ... */ }}` in {file}.
              Please move the package back and remove the manual `callPackage`.
            ",
        )
    }
}
