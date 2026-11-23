use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

use crate::structure;

#[derive(Clone, Debug, new)]
pub struct TopLevelPackageMovedOutOfByName {
    #[new(into)]
    by_name_subpath: RelativePathBuf,
    #[new(into)]
    full_attr_path: Vec<String>,
    #[new(into)]
    call_package_path: Option<RelativePathBuf>,
    #[new(into)]
    file: RelativePathBuf,
}

impl fmt::Display for TopLevelPackageMovedOutOfByName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            by_name_subpath,
            full_attr_path,
            call_package_path,
            file,
        } = self;
        let relative_package_file =
            // we should be able to safely unwrap here because the full_attr_path should always contain at least the attribute name
            structure::relative_file_for_package(by_name_subpath, full_attr_path.last().unwrap());
        let call_package_arg = call_package_path
            .as_ref()
            .map_or_else(|| "...".into(), |path| format!("./{}", path));
        writedoc!(
            f,
            "
            - Attribute `{}` was previously defined in {relative_package_file}, but is now manually defined as `callPackage {call_package_arg} {{ /* ... */ }}` in {file}.
              Please move the package back and remove the manual `callPackage`.
            ", full_attr_path.join("."),
        )
    }
}
