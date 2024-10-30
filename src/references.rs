use std::ffi::OsStr;
use std::path::Path;

use anyhow::Context;
use relative_path::RelativePath;
use rowan::ast::AstNode;

use crate::problem::{npv_121, npv_122, npv_123, npv_124, npv_125, npv_126};
use crate::structure::read_dir_sorted;
use crate::validation::{self, ResultIteratorExt, Validation::Success};
use crate::NixFileStore;

/// Check that every package directory in pkgs/by-name doesn't link to outside that directory.
/// Both symlinks and Nix path expressions are checked.
pub fn check_references(
    nix_file_store: &mut NixFileStore,
    relative_package_dir: &RelativePath,
    absolute_package_dir: &Path,
) -> validation::Result<()> {
    // The first subpath to check is the package directory itself, which we can represent as an
    // empty path, since the absolute package directory gets prepended to this.
    // We don't use `./.` to keep the error messages cleaner, since there's no canonicalisation
    // going on underneath.
    let subpath = RelativePath::new("");
    check_path(
        nix_file_store,
        relative_package_dir,
        absolute_package_dir,
        subpath,
    )
    .with_context(|| {
        format!(
            "While checking the references in package directory {relative_package_dir}"
        )
    })
}

/// Checks for a specific path to not have references outside.
///
/// The subpath is the relative path within the package directory we're currently checking.
/// A relative path so that the error messages don't get absolute paths (which are messy in CI).
/// The absolute package directory gets prepended before doing anything with it though.
fn check_path(
    nix_file_store: &mut NixFileStore,
    relative_package_dir: &RelativePath,
    absolute_package_dir: &Path,
    subpath: &RelativePath,
) -> validation::Result<()> {
    let path = subpath.to_path(absolute_package_dir);

    Ok(if path.is_symlink() {
        // Check whether the symlink resolves to outside the package directory.
        match path.canonicalize() {
            Ok(target) => {
                // No need to handle the case of it being inside the directory,
                // since we scan through the entire directory recursively in any case.
                if let Err(_prefix_error) = target.strip_prefix(absolute_package_dir) {
                    npv_125::PackageContainsSymlinkPointingOutside::new(
                        relative_package_dir,
                        subpath,
                    )
                    .into()
                } else {
                    Success(())
                }
            }
            Err(err) => {
                npv_126::PackageContainsUnresolvableSymlink::new(relative_package_dir, subpath, err)
                    .into()
            }
        }
    } else if path.is_dir() {
        // Recursively check each entry
        validation::sequence_(
            read_dir_sorted(&path)?
                .into_iter()
                .map(|entry| {
                    check_path(
                        nix_file_store,
                        relative_package_dir,
                        absolute_package_dir,
                        // TODO: The relative_path crate doesn't seem to support OsStr
                        &subpath.join(entry.file_name().to_string_lossy().to_string()),
                    )
                })
                .collect_vec()
                .with_context(|| format!("Error while recursing into {subpath}"))?,
        )
    } else if path.is_file() {
        // Only check Nix files
        if let Some(ext) = path.extension() {
            if ext == OsStr::new("nix") {
                check_nix_file(
                    nix_file_store,
                    relative_package_dir,
                    absolute_package_dir,
                    subpath,
                )
                .with_context(|| format!("Error while checking Nix file {subpath}"))?
            } else {
                Success(())
            }
        } else {
            Success(())
        }
    } else {
        // This should never happen, git doesn't support other file types
        anyhow::bail!("Unsupported file type for path {}", subpath);
    })
}

/// Check whether a Nix file contains path expression references pointing outside the package
/// directory.
fn check_nix_file(
    nix_file_store: &mut NixFileStore,
    relative_package_dir: &RelativePath,
    absolute_package_dir: &Path,
    subpath: &RelativePath,
) -> validation::Result<()> {
    let path = subpath.to_path(absolute_package_dir);

    let nix_file = nix_file_store.get(&path)?;

    Ok(validation::sequence_(
        nix_file.syntax_root.syntax().descendants().map(|node| {
            let line = nix_file.line_index.line(node.text_range().start().into());
            let text = node.text().to_string();

            // We're only interested in Path expressions
            let Some(path) = rnix::ast::Path::cast(node) else {
                return Success(());
            };

            use crate::nix_file::ResolvedPath;

            match nix_file.static_resolve_path(&path, absolute_package_dir) {
                ResolvedPath::Interpolated => npv_121::NixFileContainsPathInterpolation::new(
                    relative_package_dir,
                    subpath,
                    line,
                    text,
                )
                .into(),
                ResolvedPath::SearchPath => npv_122::NixFileContainsSearchPath::new(
                    relative_package_dir,
                    subpath,
                    line,
                    text,
                )
                .into(),
                ResolvedPath::Outside => npv_123::NixFileContainsPathOutsideDirectory::new(
                    relative_package_dir,
                    subpath,
                    line,
                    text,
                )
                .into(),
                ResolvedPath::Unresolvable(err) => npv_124::NixFileContainsUnresolvablePath::new(
                    relative_package_dir,
                    subpath,
                    line,
                    text,
                    err,
                )
                .into(),
                ResolvedPath::Within(..) => {
                    // No need to handle the case of it being inside the directory, since we scan
                    // through the entire directory recursively in any case.
                    Success(())
                }
            }
        }),
    ))
}
