use relative_path::RelativePathBuf;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::path::Path;

use crate::nix_file::NixFileStore;
use crate::validation::ResultIteratorExt;
use crate::validation::Validation::Success;
use crate::{nix_file, ratchet, validation};

/// Runs check on all Nix files, returning a ratchet result for each
pub fn check_files(
    nixpkgs_path: &Path,
    paths: &HashSet<RelativePathBuf>,
    nix_file_store: &mut NixFileStore,
) -> validation::Result<BTreeMap<RelativePathBuf, ratchet::File>> {
    process_nix_files(nixpkgs_path, paths, nix_file_store, |_nix_file| {
        // Noop for now, only boilerplate to make it easier to add future file-based checks
        Ok(Success(ratchet::File {}))
    })
}

/// Processes all Nix files in a Nixpkgs directory according to a given function `f`, collecting the
/// results into a mapping from each file to a ratchet value.
fn process_nix_files(
    nixpkgs_path: &Path,
    paths: &HashSet<RelativePathBuf>,
    nix_file_store: &mut NixFileStore,
    f: impl Fn(&nix_file::NixFile) -> validation::Result<ratchet::File>,
) -> validation::Result<BTreeMap<RelativePathBuf, ratchet::File>> {
    let results = paths
        .iter()
        .filter(|path| path.extension() == Some(".nix") && path.to_path(nixpkgs_path).is_file())
        .map(|path| {
            // Get the (optionally-cached) parsed Nix file
            let nix_file = nix_file_store.get(&path.to_path(nixpkgs_path))?;
            let result = f(nix_file)?;
            let val = result.map(|ratchet| (path.clone(), ratchet));
            Ok::<_, anyhow::Error>(val)
        })
        .collect_vec()?;

    Ok(validation::sequence(results).map(|entries| {
        // Convert the Vec to a BTreeMap
        entries.into_iter().collect()
    }))
}
