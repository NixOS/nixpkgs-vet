use std::fmt;

use derive_new::new;

use crate::structure::{self, ByNameDir};

#[derive(Clone, new, Debug)]
pub struct ByNameShardIsNotDirectory {
    #[new(into)]
    shard_name: String,
    by_name_dir: ByNameDir,
}

impl fmt::Display for ByNameShardIsNotDirectory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let relative_shard_path =
            structure::relative_dir_for_shard(&self.shard_name, &self.by_name_dir.path);
        write!(
            f,
            "- {relative_shard_path}: This is a file, but it should be a directory.",
        )
    }
}
