use crate::problem::npv_169;
use crate::ratchet::RatchetState;
use relative_path::RelativePath;
use relative_path::RelativePathBuf;

use rnix::SyntaxKind;
use rowan::ast::AstNode;
use std::collections::BTreeMap;
use std::path::Path;

use crate::nix_file::NixFileStore;
use crate::validation::ResultIteratorExt;
use crate::validation::Validation::Success;
use crate::{nix_file, ratchet, structure, validation};

fn find_invalid_withs(syntax: &rnix::SyntaxNode) -> Option<rnix::SyntaxNode> {
    syntax
        .descendants()
        .filter(|node| node.kind() == rnix::SyntaxKind::NODE_WITH)
        .filter(|node| {
            node.descendants()
                .map(|child| {
                    if child == *node {
                        return None;
                    }
                    let node_if_invalid = match child.kind() {
                        SyntaxKind::NODE_WITH => Some(node),
                        SyntaxKind::NODE_LET_IN => Some(node),
                        SyntaxKind::NODE_ATTR_SET => Some(node),
                        _ => None,
                    };
                    println!(
                        "validate with={:?} subexpr={:?} invalid={:?}",
                        node.to_string(),
                        child.to_string(),
                        node_if_invalid
                    );
                    node_if_invalid
                })
                .any(|cond| cond != None)
        })
        .take(1)
        .last()
}

pub fn check_files(
    nixpkgs_path: &Path,
    nix_file_store: &mut NixFileStore,
) -> validation::Result<BTreeMap<RelativePathBuf, ratchet::File>> {
    process_nix_files(nixpkgs_path, nix_file_store, |nix_file| {
        if let Some(_open_scope_with_lib) = find_invalid_withs(nix_file.syntax_root.syntax()) {
            let path = RelativePathBuf::from_path(
                nix_file.path.clone().strip_prefix(nixpkgs_path).unwrap(),
            )
            .unwrap();
            return Ok(Success(ratchet::File {
                file_is_str: Some(
                    RatchetState::Loose(
                        npv_169::TopLevelWithMayShadowVariablesAndBreakStaticChecks::new(path)
                            .into(),
                    )
                    .into(),
                ),
            }));
        }
        Ok(Success(ratchet::File { file_is_str: None }))
    })
}

fn collect_nix_files(
    base: &Path,
    dir: &RelativePath,
    files: &mut Vec<RelativePathBuf>,
) -> anyhow::Result<()> {
    for entry in structure::read_dir_sorted(&dir.to_path(base))? {
        let mut relative_path = dir.to_relative_path_buf();
        relative_path.push(entry.file_name().to_string_lossy().into_owned());

        let absolute_path = entry.path();

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

fn process_nix_files<F: Fn(&nix_file::NixFile) -> validation::Result<ratchet::File>>(
    nixpkgs_path: &Path,
    nix_file_store: &mut NixFileStore,
    f: F,
) -> validation::Result<BTreeMap<RelativePathBuf, ratchet::File>> {
    let files = {
        let mut files = vec![];
        collect_nix_files(nixpkgs_path, &RelativePathBuf::new(), &mut files)?;
        files
    };

    let file_results: Vec<validation::Validation<(RelativePathBuf, ratchet::File)>> = files
        .into_iter()
        .map(|path| {
            let nix_file = nix_file_store.get(&path.to_path(nixpkgs_path))?;
            let val = f(nix_file)?.map(|file| (path, file));
            Ok::<_, anyhow::Error>(val)
        })
        .collect_vec()?;

    Ok(validation::sequence(file_results).map(|entries| entries.into_iter().collect()))
}
