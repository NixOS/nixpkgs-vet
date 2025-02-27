use rnix::SyntaxKind::NODE_PATH;
use rowan::ast::AstNode;
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
        let p = nix_file.path.strip_prefix(nixpkgs_path).unwrap();
        let s = nix_file.syntax_root.syntax();
        for d in s.descendants() {
            let line = nix_file.line_index.line(d.text_range().start().into());
            if d.kind() == NODE_PATH {
                if d.text().to_string().starts_with("/") {
                    eprintln!("- [{}#L{}](https://github.com/NixOS/nixpkgs/blob/576f2c930108b9ff47e58623ea77836fa648b137/{}#L{})", p.to_string_lossy(), line, p.to_string_lossy(), line);
                }
            }
        }
        // Noop for now, only boilerplate to make it easier to add future file-based checks
        Ok(Success(ratchet::File {}))
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
        if entry.file_name() == ".git" {
            continue;
        }
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
