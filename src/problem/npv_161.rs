use std::fmt;

use crate::gh_write::{Options, gh_write};
use derive_new::new;
use indoc::formatdoc;
use relative_path::RelativePathBuf;

use crate::structure;

#[derive(Clone, new)]
pub struct TopLevelPackageMovedOutOfByNameWithCustomArguments {
    #[new(into)]
    package_name: String,
    #[new(into)]
    call_package_path: Option<RelativePathBuf>,
    #[new(into)]
    file: RelativePathBuf,
}

impl fmt::Display for TopLevelPackageMovedOutOfByNameWithCustomArguments {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            package_name,
            call_package_path,
            file,
        } = self;
        let relative_package_file = structure::relative_file_for_package(package_name);
        let call_package_arg = call_package_path
            .as_ref()
            .map_or_else(|| "...".into(), |path| format!("./{}", path));
        gh_write(
            f,
            formatdoc!("
            - Attribute `pkgs.{package_name}` was previously defined in {relative_package_file}, but is now manually defined as `callPackage {call_package_arg} {{ ... }}` in {file}.
              While the manual `callPackage` is still needed, it's not necessary to move the package files.
            "),
            Options {
                file: Some(file),
                ..Default::default()
            },
        )
    }
}
