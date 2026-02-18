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

/// The maximum number of non-trivia tokens allowed under a single `with` expression.
const WITH_MAX_TOKENS: usize = 125;

/// The maximum fraction of a file's non-trivia tokens that a single `with` expression may cover.
const WITH_MAX_FILE_FRACTION: f64 = 0.25;

/// Files with fewer than this many non-trivia tokens are exempt from the `with` check entirely.
/// Small files don't benefit much from restricting `with` scope.
const WITH_FILE_MIN_TOKENS: usize = 50;

/// Counts the non-trivia (non-whitespace, non-comment) tokens under a syntax node.
fn count_non_trivia_tokens(node: &rnix::SyntaxNode) -> usize {
    node.descendants_with_tokens()
        .filter(|element| element.as_token().is_some_and(|t| !t.kind().is_trivia()))
        .count()
}

/// Finds the first `with` expression in the syntax tree that is overly broad, meaning it either:
///
/// - Contains more than [`WITH_MAX_TOKENS`] non-trivia tokens, or
/// - Covers more than [`WITH_MAX_FILE_FRACTION`] of the file's total non-trivia tokens.
///
/// Files with fewer than [`WITH_FILE_MIN_TOKENS`] non-trivia tokens are exempt.
///
/// Large `with` scopes shadow variables across a wide region, making static analysis unreliable
/// and code harder to understand. Small, tightly-scoped uses (e.g. `with lib.maintainers; [...]`)
/// are fine.
///
/// Returns `Some(node)` for the first offending `with` node, or `None` if no such node exists.
fn find_overly_broad_with(syntax: &rnix::SyntaxNode) -> Option<rnix::SyntaxNode> {
    let file_tokens = count_non_trivia_tokens(syntax);

    if file_tokens < WITH_FILE_MIN_TOKENS {
        return None;
    }

    syntax
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::NODE_WITH)
        .find(|node| {
            let with_tokens = count_non_trivia_tokens(node);
            with_tokens > WITH_MAX_TOKENS
                || with_tokens as f64 > WITH_MAX_FILE_FRACTION * file_tokens as f64
        })
}

/// Runs ratchet checks on all Nix files in the Nixpkgs tree, returning a ratchet result for each.
pub fn check_files(
    nixpkgs_path: &Path,
    nix_file_store: &mut NixFileStore,
) -> validation::Result<BTreeMap<RelativePathBuf, ratchet::File>> {
    process_nix_files(nixpkgs_path, nix_file_store, |relative_path, nix_file| {
        Ok(Success(ratchet::File {
            top_level_with: check_top_level_with(relative_path, nix_file),
        }))
    })
}

/// Checks a single Nix file for overly broad `with` expressions. Returns [`RatchetState::Loose`]
/// with a problem if such a `with` is found, or [`RatchetState::Tight`] if the file is clean.
fn check_top_level_with(
    relative_path: &RelativePath,
    nix_file: &nix_file::NixFile,
) -> RatchetState<ratchet::DoesNotIntroduceToplevelWiths> {
    if find_overly_broad_with(nix_file.syntax_root.syntax()).is_some() {
        RatchetState::Loose(
            npv_169::OverlyBroadWith::new(relative_path.to_relative_path_buf()).into(),
        )
    } else {
        RatchetState::Tight
    }
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

        // We reach every file via directory recursion, no need to follow symlinks.
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

/// Processes all Nix files in a Nixpkgs directory according to a given function `f`, collecting the
/// results into a mapping from each file to a ratchet value.
fn process_nix_files(
    nixpkgs_path: &Path,
    nix_file_store: &mut NixFileStore,
    f: impl Fn(&RelativePath, &nix_file::NixFile) -> validation::Result<ratchet::File>,
) -> validation::Result<BTreeMap<RelativePathBuf, ratchet::File>> {
    // Get all Nix files
    let files = {
        let mut files = vec![];
        collect_nix_files(nixpkgs_path, &RelativePathBuf::new(), &mut files)?;
        files
    };

    let file_results: Vec<validation::Validation<(RelativePathBuf, ratchet::File)>> = files
        .into_iter()
        .map(|path| {
            // Get the (optionally-cached) parsed Nix file
            let nix_file = nix_file_store.get(&path.to_path(nixpkgs_path))?;
            let val = f(&path, nix_file)?.map(|file| (path, file));
            Ok::<_, anyhow::Error>(val)
        })
        .collect_vec()?;

    Ok(validation::sequence(file_results).map(|entries| entries.into_iter().collect()))
}
