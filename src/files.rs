use relative_path::RelativePath;
use relative_path::RelativePathBuf;
use std::collections::BTreeMap;
use std::path::Path;

use crate::nix_file::NixFileStore;
use crate::validation::ResultIteratorExt;
use crate::validation::Validation::Success;
use crate::{nix_file, ratchet, structure, validation};

/// Runs check on all Nix files, returning a ratchet result for each
pub fn check_files(
    nixpkgs_path: &Path,
    nix_file_store: &mut NixFileStore,
) -> validation::Result<BTreeMap<RelativePathBuf, ratchet::File>> {
    process_nix_files(nixpkgs_path, nix_file_store, |nix_file| {
        // Bogus ratchet check towards enforcing that no files are strings
        let file_is_str = match nix_file.syntax_root.expr() {
            // This happens if the file can't be parsed, in which case we can't really decide
            // whether it's a string or not
            None => ratchet::RatchetState::NonApplicable,
            // The expression is a string, not allowed for new files and for existing files to be
            // changed to a string
            Some(Str(_)) => ratchet::RatchetState::Loose(
                npv_170::FileIsAString::new(
                    RelativePathBuf::from_path(nix_file.path.strip_prefix(nixpkgs_path).unwrap())
                        .unwrap(),
                )
                .into(),
            ),
            // This is good
            Some(_) => ratchet::RatchetState::Tight,
        };
        Ok(Success(ratchet::File { file_is_str }))
    })
}

/// Processes all Nix files in a Nixpkgs directory according to a given function `f`, collecting the
/// results into a mapping from each file to a ratchet value.
fn process_nix_files(
    nixpkgs_path: &Path,
    nix_file_store: &mut NixFileStore,
    f: impl Fn(&nix_file::NixFile) -> validation::Result<ratchet::File>,
) -> validation::Result<BTreeMap<RelativePathBuf, ratchet::File>> {
    // Get all Nix files
    let files = {
        let mut files = vec![];
        collect_nix_files(nixpkgs_path, &RelativePathBuf::new(), &mut files)?;
        files
    };

    let results = files
        .into_iter()
        .map(|path| {
            // Get the (optionally-cached) parsed Nix file
            let nix_file = nix_file_store.get(&path.to_path(nixpkgs_path))?;
            let result = f(nix_file)?;
            let val = result.map(|ratchet| (path, ratchet));
            Ok::<_, anyhow::Error>(val)
        })
        .collect_vec()?;

    Ok(validation::sequence(results).map(|entries| {
        // Convert the Vec to a BTreeMap
        entries.into_iter().collect()
    }))
}

/// Recursively collects all Nix files in the relative `dir` within `base`
/// into the `files` `Vec`.
fn collect_nix_files(
    base: &Path,
    dir: &RelativePath,
    files: &mut Vec<RelativePathBuf>,
) -> anyhow::Result<()> {
    for entry in structure::read_dir_sorted(&dir.to_path(base))? {
        let mut relative_path = dir.to_relative_path_buf();
        relative_path.push(entry.file_name().to_string_lossy().into_owned());

        let absolute_path = entry.path();

        // We'll get to every file based on directory recursion, no need to follow symlinks.
        if absolute_path.is_symlink() {
            continue;
        }
        if absolute_path.is_dir() {
            collect_nix_files(base, &relative_path, files)?
        } else if absolute_path.extension().is_some_and(|x| x == "nix") {
            files.push(relative_path)
        }
    }
    Ok(())
}
