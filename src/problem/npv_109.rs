use std::fmt;

use crate::gh_write::{Options, gh_write};
use derive_new::new;

use crate::structure;

#[derive(Clone, new)]
pub struct ByNameShardIsNotDirectory {
    #[new(into)]
    shard_name: String,
}

impl fmt::Display for ByNameShardIsNotDirectory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let relative_shard_path = structure::relative_dir_for_shard(&self.shard_name);
        gh_write(
            f,
            format!("- {relative_shard_path}: This is a file, but it should be a directory."),
            Options {
                ..Default::default()
            },
        )
    }
}
