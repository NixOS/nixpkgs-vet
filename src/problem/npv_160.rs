use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

use crate::structure::{self, Config};

#[derive(Clone, new, Debug)]
pub struct TopLevelPackageMovedOutOfByName {
    #[new(into)]
    package_name: String,
    #[new(into)]
    call_package_path: Option<RelativePathBuf>,
    #[new(into)]
    file: RelativePathBuf,
    config: Config,
}

impl fmt::Display for TopLevelPackageMovedOutOfByName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            package_name,
            call_package_path,
            file,
            config,
        } = self;
        let by_name_path = structure::expected_by_name_dir_for_package(package_name, config)
            .unwrap()
            .path;
        let relative_package_file =
            structure::relative_file_for_package(package_name, &by_name_path);
        let call_package_arg = call_package_path
            .as_ref()
            .map_or_else(|| "...".into(), |path| format!("./{}", path));
        writedoc!(
            f,
            "
            - Attribute `{package_name}` was previously defined in {relative_package_file}, but is now manually defined as `callPackage {call_package_arg} {{ /* ... */ }}` in {file}.
              Please move the package back and remove the manual `callPackage`.
            ",
        )
    }
}
