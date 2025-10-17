use std::fs::{DirEntry, read};
use std::path::Path;
use std::sync::LazyLock;

use anyhow::Context;
use itertools::{concat, process_results};
use regex::Regex;
use relative_path::{RelativePath, RelativePathBuf};
use serde::Deserialize;

use crate::NixFileStore;
use crate::problem::{npv_109, npv_110, npv_111, npv_140, npv_141, npv_142, npv_143, npv_144};
use crate::references;
use crate::validation::{self, ResultIteratorExt, Validation::Success};

pub const PACKAGE_NIX_FILENAME: &str = "package.nix";

static SHARD_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9_-]{1,2}$").unwrap());
static PACKAGE_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap());

#[derive(Deserialize, Debug)]
struct DeserializedByNameDir {
    path: String,
    pub attr_path_regex: String,
}

#[derive(Deserialize, Debug)]
struct DeserializedConfig {
    by_name_dirs: Vec<DeserializedByNameDir>,
}

pub struct ByNameDir {
    pub path: RelativePathBuf,
    pub attr_path_regex: Regex,
}

pub struct Config {
    pub by_name_dirs: Vec<ByNameDir>,
}

pub fn read_config(config_file: &Path) -> Config {
    let config_file_contents = read(config_file);
    println!("config file path {}", config_file.to_string_lossy());
    let config: DeserializedConfig = serde_json::from_slice(config_file_contents.with_context(|| format!("Config file {}", config_file.display())).unwrap().as_slice()).unwrap();
    let by_name_dirs = config
        .by_name_dirs
        .iter()
        .map(|x| {
            let regex_str = x.attr_path_regex.as_str();
            ByNameDir {
                path: RelativePathBuf::from(x.path.as_str()),
                attr_path_regex: regex::Regex::new(regex_str).unwrap(),
            }
        })
        .collect();

    Config { by_name_dirs }
}

/// Deterministic file listing so that tests are reproducible.
pub fn read_dir_sorted(base_dir: &Path) -> anyhow::Result<Vec<DirEntry>> {
    let ctx = || format!("Could not list directory {}", base_dir.display());
    let listing = base_dir.read_dir().with_context(ctx)?;

    process_results(listing, |listing| {
        use itertools::Itertools;
        Itertools::collect_vec(listing.sorted_by_key(DirEntry::file_name))
    })
    .with_context(ctx)
}

// Some utility functions for the basic structure

pub fn shard_for_package(package_name: &str) -> String {
    package_name.to_lowercase().chars().take(2).collect()
}

pub fn relative_dir_for_shard(shard_name: &str, byname_basedir: &RelativePath) -> RelativePathBuf {
    byname_basedir.join(shard_name)
}

pub fn relative_dir_for_package(
    package_name: &str,
    byname_basedir: &RelativePath,
) -> RelativePathBuf {
    relative_dir_for_shard(&shard_for_package(package_name), byname_basedir).join(package_name)
}

pub fn relative_file_for_package(
    package_name: &str,
    byname_basedir: &RelativePath,
) -> RelativePathBuf {
    relative_dir_for_package(package_name, byname_basedir).join(PACKAGE_NIX_FILENAME)
}

pub fn expected_by_name_dir_for_package(attr_name: &str, config: &Config) -> RelativePathBuf {
    let matching_dirs: Vec<&ByNameDir> = config
        .by_name_dirs
        .iter()
        .filter(|x| x.attr_path_regex.is_match(attr_name))
        .collect();
    match matching_dirs.len() {
        1 => matching_dirs[0].path.clone(),
        2 => {
            let dir1 = matching_dirs[0];
            let dir2 = matching_dirs[1];
            if dir1.attr_path_regex.as_str() == ".*" {
                dir1.path.clone()
            } else {
                dir2.path.clone()
            }
        }
        0 => panic!("There should be exactly one wildcard directory."),
        _ => panic!("Multiple wildcard regexes, or overlapping regexes, detected."),
    }
}

/// Check the structure of Nixpkgs, returning the attribute names that are defined in
/// `pkgs/by-name`
pub fn check_structure(
    path: &Path,
    nix_file_store: &mut NixFileStore,
    by_name_subpath: &RelativePath,
) -> validation::Result<Vec<String>> {
    let base_dir = path.join(by_name_subpath.as_str());

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
                npv_109::ByNameShardIsNotDirectory::new(shard_name).into()
            } else {
                let shard_name_valid = SHARD_NAME_REGEX.is_match(&shard_name);
                let result = if !shard_name_valid {
                    npv_110::ByNameShardIsInvalid::new(shard_name.clone()).into()
                } else {
                    Success(())
                };

                let entries = read_dir_sorted(&shard_path)?;

                let duplicate_results = entries
                    .iter()
                    .zip(entries.iter().skip(1))
                    .filter(|(l, r)| l.file_name().eq_ignore_ascii_case(r.file_name()))
                    .map(|(l, r)| {
                        npv_111::ByNameShardIsCaseSensitiveDuplicate::new(
                            shard_name.clone(),
                            l.file_name(),
                            r.file_name(),
                        )
                        .into()
                    });

                let result = result.and_(validation::sequence_(duplicate_results));

                let package_results = entries
                    .into_iter()
                    .map(|package_entry| {
                        check_package(
                            nix_file_store,
                            path,
                            &shard_name,
                            shard_name_valid,
                            &package_entry,
                            by_name_subpath,
                        )
                    })
                    .collect_vec()?;

                result.and_(validation::sequence(package_results))
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
    package_entry: &DirEntry,
    by_name_subpath: &RelativePath,
) -> validation::Result<String> {
    let package_path = package_entry.path();
    let package_name = package_entry.file_name().to_string_lossy().into_owned();
    let relative_package_dir = by_name_subpath.join(shard_name).join(&package_name);
    // RelativePathBuf::from(format!("{BASE_SUBPATH}/{shard_name}/{package_name}"));

    Ok(if !package_path.is_dir() {
        npv_140::PackageDirectoryIsNotDirectory::new(&package_name).into()
    } else {
        let package_name_valid = PACKAGE_NAME_REGEX.is_match(&package_name);
        let result = if !package_name_valid {
            npv_141::InvalidPackageDirectoryName::new(
                package_name.clone(),
                relative_package_dir.clone(),
            )
            .into()
        } else {
            Success(())
        };

        let correct_relative_package_dir = relative_dir_for_package(&package_name, by_name_subpath);
        let result = result.and_(if relative_package_dir != correct_relative_package_dir {
            // Only show this error if we have a valid shard and package name.
            // If one of those is wrong, you should fix that first.
            if shard_name_valid && package_name_valid {
                npv_142::PackageInWrongShard::new(
                    package_name.clone(),
                    relative_package_dir.clone(),
                )
                .into()
            } else {
                Success(())
            }
        } else {
            Success(())
        });

        let package_nix_path = package_path.join(PACKAGE_NIX_FILENAME);
        let result = result.and_(if !package_nix_path.exists() {
            npv_143::PackageNixMissing::new(package_name.clone()).into()
        } else if !package_nix_path.is_file() {
            npv_144::PackageNixIsNotFile::new(package_name.clone()).into()
        } else {
            Success(())
        });

        let result = result.and_(references::check_references(
            nix_file_store,
            &relative_package_dir,
            &relative_package_dir.to_path(path),
        )?);

        result.map(|_| package_name)
    })
}
