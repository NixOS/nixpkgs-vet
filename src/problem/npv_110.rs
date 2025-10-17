use std::fmt;

use derive_new::new;
use relative_path::RelativePath;

use crate::structure;

#[derive(Clone, new, Debug)]
pub struct ByNameShardIsInvalid {
    #[new(into)]
    shard_name: String,
}

impl fmt::Display for ByNameShardIsInvalid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let shard_name = &self.shard_name;
        let relative_shard_path =
            structure::relative_dir_for_shard(shard_name, RelativePath::new("pkgs/by-name"));
        write!(
            f,
            "- {relative_shard_path}: Invalid directory name \"{shard_name}\", must be at most 2 ASCII characters consisting of a-z, 0-9, \"-\" or \"_\".",
        )
    }
}
