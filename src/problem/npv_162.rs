use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

use crate::structure;

#[derive(Clone, new)]
pub struct NewTopLevelPackageShouldBeByName {
    #[new(into)]
    package_name: String,
    #[new(into)]
    call_package_path: Option<RelativePathBuf>,
    #[new(into)]
    file: RelativePathBuf,
}

impl fmt::Display for NewTopLevelPackageShouldBeByName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            package_name,
            call_package_path,
            file,
        } = self;
        let relative_package_file = structure::relative_file_for_package(package_name);
        let call_package_arg = if let Some(path) = call_package_path {
            format!("./{}", path)
        } else {
            "...".into()
        };
        writedoc!(
            f,
            "
            - Attribute `pkgs.{package_name}` is a new top-level package using `pkgs.callPackage {call_package_arg} {{ /* ... */ }}`.
              Please define it in {relative_package_file} instead.
              See `pkgs/by-name/README.md` for more details.
              Since the second `callPackage` argument is `{{ }}`, no manual `callPackage` in {file} is needed anymore.
            ",
        )
    }
}
