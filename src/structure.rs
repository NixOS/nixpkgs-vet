use std::fs::DirEntry;
use std::path::Path;

use anyhow::Context;
use itertools::{concat, process_results};
use lazy_static::lazy_static;
use regex::Regex;
use relative_path::RelativePathBuf;

use crate::problem::{
    ByNameShardIsInvalid, ByNameShardIsNotDirectory, PackageError, PackageErrorKind, Problem,
    ShardError, ShardErrorKind,
};
use crate::references;
use crate::validation::{self, ResultIteratorExt, Validation::Success};
use crate::NixFileStore;

pub const BASE_SUBPATH: &str = "pkgs/by-name";
pub const PACKAGE_NIX_FILENAME: &str = "package.nix";

lazy_static! {
    static ref SHARD_NAME_REGEX: Regex = Regex::new(r"^[a-z0-9_-]{1,2}$").unwrap();
    static ref PACKAGE_NAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
}

/// Deterministic file listing so that tests are reproducible.
pub fn read_dir_sorted(base_dir: &Path) -> anyhow::Result<Vec<DirEntry>> {
    let ctx = || format!("Could not list directory {}", base_dir.display());
    let listing = base_dir.read_dir().with_context(ctx)?;

    process_results(listing, |listing| {
        use itertools::Itertools;
        Itertools::collect_vec(listing.sorted_by_key(|entry| entry.file_name()))
    })
    .with_context(ctx)
}

// Some utility functions for the basic structure

pub fn shard_for_package(package_name: &str) -> String {
    package_name.to_lowercase().chars().take(2).collect()
}

pub fn relative_dir_for_shard(shard_name: &str) -> RelativePathBuf {
    RelativePathBuf::from(format!("{BASE_SUBPATH}/{shard_name}"))
}

pub fn relative_dir_for_package(package_name: &str) -> RelativePathBuf {
    relative_dir_for_shard(&shard_for_package(package_name)).join(package_name)
}

pub fn relative_file_for_package(package_name: &str) -> RelativePathBuf {
    relative_dir_for_package(package_name).join(PACKAGE_NIX_FILENAME)
}

/// Check the structure of Nixpkgs, returning the attribute names that are defined in
/// `pkgs/by-name`
pub fn check_structure(
    path: &Path,
    nix_file_store: &mut NixFileStore,
) -> validation::Result<Vec<String>> {
    let base_dir = path.join(BASE_SUBPATH);

    let shard_results = read_dir_sorted(&base_dir)?
        .into_iter()
        .map(|shard_entry| -> validation::Result<_> {
            let shard_path = shard_entry.path();
            let shard_name = shard_entry.file_name().to_string_lossy().into_owned();

            Ok(if shard_name == "README.md" {
                // README.md is allowed to be a file and not checked
                Success(vec![])
            } else if !shard_path.is_dir() {
                // We can't check for any other errors if it's not a directory, since there are no
                // subdirectories to check.
                ByNameShardIsNotDirectory::new(shard_name).into()
            } else {
                let shard_name_valid = SHARD_NAME_REGEX.is_match(&shard_name);
                let result = if !shard_name_valid {
                    ByNameShardIsInvalid::new(shard_name.clone()).into()
                } else {
                    Success(())
                };

                let entries = read_dir_sorted(&shard_path)?;

                let duplicate_results = entries
                    .iter()
                    .zip(entries.iter().skip(1))
                    .filter(|(l, r)| {
                        l.file_name().to_ascii_lowercase() == r.file_name().to_ascii_lowercase()
                    })
                    .map(|(l, r)| {
                        Problem::Shard(ShardError {
                            shard_name: shard_name.clone(),
                            kind: ShardErrorKind::CaseSensitiveDuplicate {
                                first: l.file_name(),
                                second: r.file_name(),
                            },
                        })
                        .into()
                    });

                let result = result.and(validation::sequence_(duplicate_results));

                let package_results = entries
                    .into_iter()
                    .map(|package_entry| {
                        check_package(
                            nix_file_store,
                            path,
                            &shard_name,
                            shard_name_valid,
                            package_entry,
                        )
                    })
                    .collect_vec()?;

                result.and(validation::sequence(package_results))
            })
        })
        .collect_vec()?;

    // Combine the package names contained within each shard into a longer list.
    Ok(validation::sequence(shard_results).map(concat))
}

fn check_package(
    nix_file_store: &mut NixFileStore,
    path: &Path,
    shard_name: &str,
    shard_name_valid: bool,
    package_entry: DirEntry,
) -> validation::Result<String> {
    let package_path = package_entry.path();
    let package_name = package_entry.file_name().to_string_lossy().into_owned();
    let relative_package_dir =
        RelativePathBuf::from(format!("{BASE_SUBPATH}/{shard_name}/{package_name}"));

    let to_validation = |kind| -> validation::Validation<()> {
        Problem::Package(PackageError {
            relative_package_dir: relative_package_dir.clone(),
            kind,
        })
        .into()
    };

    Ok(if !package_path.is_dir() {
        to_validation(PackageErrorKind::PackageNonDir {
            package_name: package_name.clone(),
        })
        .map(|_| package_name)
    } else {
        let package_name_valid = PACKAGE_NAME_REGEX.is_match(&package_name);
        let result = if !package_name_valid {
            to_validation(PackageErrorKind::InvalidPackageName {
                invalid_package_name: package_name.clone(),
            })
        } else {
            Success(())
        };

        let correct_relative_package_dir = relative_dir_for_package(&package_name);
        let result = result.and(if relative_package_dir != correct_relative_package_dir {
            // Only show this error if we have a valid shard and package name. If one of those is
            // wrong, you should fix that first.
            if shard_name_valid && package_name_valid {
                to_validation(PackageErrorKind::IncorrectShard {
                    correct_relative_package_dir: correct_relative_package_dir.clone(),
                })
            } else {
                Success(())
            }
        } else {
            Success(())
        });

        let package_nix_path = package_path.join(PACKAGE_NIX_FILENAME);
        let result = result.and(if !package_nix_path.exists() {
            to_validation(PackageErrorKind::PackageNixNonExistent)
        } else if package_nix_path.is_dir() {
            to_validation(PackageErrorKind::PackageNixDir)
        } else {
            Success(())
        });

        let result = result.and(references::check_references(
            nix_file_store,
            &relative_package_dir,
            &relative_package_dir.to_path(path),
        )?);

        result.map(|_| package_name)
    })
}
