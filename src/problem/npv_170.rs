use std::fmt;

use derive_new::new;
use indoc::writedoc;
use relative_path::RelativePathBuf;

#[derive(Clone, new)]
pub struct FileIsAString {
    #[new(into)]
    file: RelativePathBuf,
}

impl fmt::Display for FileIsAString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { file } = self;
        writedoc!(
            f,
            "
            - File {file} is a string, which is not allowed anymore
            ",
        )
    }
}
