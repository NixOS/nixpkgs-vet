use std::ffi::OsString;
use std::fmt;

use derive_new::new;

use crate::structure::{self, ByNameDir};

#[derive(Clone, new, Debug)]
pub struct ByNameShardIsCaseSensitiveDuplicate {
    #[new(into)]
    shard_name: String,
    first: OsString,
    second: OsString,
    by_name_dir: ByNameDir,
}

impl fmt::Display for ByNameShardIsCaseSensitiveDuplicate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let relative_shard_path =
            structure::relative_dir_for_shard(&self.shard_name, &self.by_name_dir.path);
        let first = self.first.to_string_lossy();
        let second = self.second.to_string_lossy();
        write!(
            f,
            "- {relative_shard_path}: Duplicate case-sensitive package directories \"{first}\" and \"{second}\"."
        )
    }
}
