use std::ffi::OsString;
use std::fmt;

use derive_new::new;

use crate::structure;

#[derive(Clone, new)]
pub struct ByNameShardIsCaseSensitiveDuplicate {
    #[new(into)]
    by_name_subpath: String,
    #[new(into)]
    shard_name: String,
    first: OsString,
    second: OsString,
}

impl fmt::Display for ByNameShardIsCaseSensitiveDuplicate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            by_name_subpath,
            shard_name,
            first,
            second,
        } = self;
        let relative_shard_path = structure::relative_dir_for_shard(by_name_subpath, shard_name);
        let first = first.to_string_lossy();
        let second = second.to_string_lossy();
        write!(
            f,
            "- {relative_shard_path}: Duplicate case-sensitive package directories \"{first}\" and \"{second}\"."
        )
    }
}
