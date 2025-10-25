use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

use crate::structure::{self, Config};

#[derive(Clone, new, Debug)]
pub struct NewTopLevelPackageShouldBeByName {
    #[new(into)]
    attr_path: String,
    #[new(into)]
    call_package_path: Option<RelativePathBuf>,
    #[new(into)]
    file: RelativePathBuf,
    config: Config,
}

impl fmt::Display for NewTopLevelPackageShouldBeByName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            attr_path,
            call_package_path,
            file,
            config,
        } = self;
        let by_name_path =
            structure::expected_by_name_dir_for_package(attr_path, config).unwrap().path;
        let relative_package_file =
            structure::relative_file_for_package(attr_path, &by_name_path);
        println!("{}:{}: attr_path {attr_path}, by_name_path {by_name_path}, relative_package_file {relative_package_file}", file!(), line!());
        let call_package_arg = call_package_path
            .as_ref()
            .map_or_else(|| "...".into(), |path| format!("./{}", path));
        writedoc!(
            f,
            "
            - Attribute `{attr_path}` is a new top-level package using `callPackage {call_package_arg} {{ /* ... */ }}`.
              Please define it in {relative_package_file} instead.
              See `pkgs/by-name/README.md` for more details.
              Since the second `callPackage` argument is `{{ }}`, no manual `callPackage` in {file} is needed anymore.
            ",
        )
    }
}
