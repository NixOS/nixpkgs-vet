use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

use crate::structure;

#[derive(Clone, Debug, new)]
pub struct NewTopLevelPackageShouldBeByName {
    #[new(into)]
    by_name_subpath: RelativePathBuf,
    #[new(into)]
    full_attr_path: Vec<String>,
    #[new(into)]
    call_package_path: Option<RelativePathBuf>,
}

impl fmt::Display for NewTopLevelPackageShouldBeByName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            by_name_subpath,
            full_attr_path,
            call_package_path,
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
            - Attribute `{}` is a new top-level package using `callPackage {call_package_arg} {{ /* ... */ }}`.
              Please define it in {relative_package_file} instead.
              See `pkgs/by-name/README.md` for more details.
            ", full_attr_path.join("."),
        )
    }
}
