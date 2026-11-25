// Temporarily uncomment to view more pedantic or new nursery lints.
// #![warn(clippy::pedantic)]
// #![allow(clippy::if_not_else)]
// #![allow(clippy::ignored_unit_patterns)]
// #![allow(clippy::module_name_repetitions)]
// #![allow(clippy::uninlined_format_args)]
// #![allow(clippy::unnested_or_patterns)]
// #![warn(clippy::nursery)]
// #![allow(clippy::use_self)]
// #![allow(clippy::missing_const_for_fn)]

mod eval;
mod files;
mod location;
mod nix_file;
mod problem;
mod ratchet;
mod references;
mod status;
mod structure;
mod validation;

use anyhow::Context as _;
use clap::Parser;
use relative_path::RelativePathBuf;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{panic, thread};

use crate::nix_file::NixFileStore;
use crate::problem::npv_180;
use crate::status::{ColoredStatus, Status};
use crate::structure::check_structure;
use crate::validation::Validation::Failure;
use crate::validation::Validation::Success;

/// Program to check the validity of pkgs/by-name
///
/// This CLI interface may be changed over time if the CI workflow making use of it is adjusted to
/// deal with the change appropriately.
///
/// Exit code:
/// - `0`: If the validation is successful
/// - `1`: If the validation is not successful
/// - `2`: If an unexpected I/O error occurs
///
/// Standard error:
/// - Informative messages
/// - Detected problems if validation is not successful
#[derive(Parser, Debug)]
#[command(about, version, verbatim_doc_comment)]
pub struct Args {
    /// Path to the main Nixpkgs to check. For PRs, set this to a checkout of the PR branch.
    nixpkgs: PathBuf,

    /// Path to the base Nixpkgs to run ratchet checks against.
    /// For PRs, set this to a checkout of the PRs base branch.
    #[arg(long)]
    base: PathBuf,
}

// default config
#[cfg(not(test))]
const DEFAULT_NIXPKGS_VET_JSON: &str = include_str!("../default-nixpkgs-vet.json");
// default config for tests
#[cfg(test)]
const DEFAULT_NIXPKGS_VET_JSON: &str = include_str!("../tests/tests-nixpkgs-vet.json");

/// config file format
#[derive(Clone, Deserialize)]
pub struct Config {
    /// List of package sets that is to be checked by nixpkgs-vet
    package_sets: Vec<PkgSet>,
}

/// one checked package set, characterized the file- and attribute path
#[derive(Clone, Deserialize)]
pub struct PkgSet {
    /// The path to the by-name folder
    /// e.g. pkgs/by-name
    by_name_subpath: RelativePathBuf,

    /// The attribute path to the package set
    /// empty for the top-level package set
    attr_path: Vec<String>,
}

impl Config {
    /// reads in the confing file at the given path with serde
    fn read(path: &Path) -> Result<Self, serde_json::Error> {
        let mut config_path: PathBuf = path.into();
        config_path.push("nixpkgs-vet.json");
        let config_content =
            read_to_string(config_path).unwrap_or_else(|_| String::from(DEFAULT_NIXPKGS_VET_JSON));

        serde_json::from_str(&config_content)
    }
}

fn main() -> ExitCode {
    let args = Args::parse();
    let status: ColoredStatus = process(args.base, &args.nixpkgs).into();
    eprintln!("{status}");
    status.into()
}

/// Does the actual work. This is the abstraction used both by `main` and the tests.
///
/// # Arguments
/// - `base_nixpkgs`: Path to the base Nixpkgs to run ratchet checks against.
/// - `main_nixpkgs`: Path to the main Nixpkgs to check.
fn process(base_nixpkgs: PathBuf, main_nixpkgs: &Path) -> Status {
    // Parse the config files, return the respective Status when parsing fails.
    let (base_config, main_config) = match (Config::read(&base_nixpkgs), Config::read(main_nixpkgs))
    {
        (Err(_), Err(e)) => {
            return Status::BranchStillBroken(vec![
                npv_180::ConfigFileIsBroken::new(format!("{e}")).into(),
            ]);
        }
        (Err(_), Ok(_)) => return Status::BranchHealed,
        (Ok(_), Err(e)) => {
            return Status::ProblemsIntroduced(vec![
                npv_180::ConfigFileIsBroken::new(format!("{e}")).into(),
            ]);
        }
        (Ok(c1), Ok(c2)) => (c1, c2),
    };

    // Remove newly introduced package sets from the main config.
    // Newly added package sets will not get eval checked.
    // Otherwise migrating a large package set would be near impossible.
    // (The ratchet would need to be tight before adding it to nixpkgs-vet).
    let new_package_sets: Vec<PkgSet> = main_config
        .package_sets
        .clone()
        .into_iter()
        .filter(|x1| {
            base_config
                .package_sets
                .iter()
                .all(|x2| x2.attr_path != x1.attr_path)
        })
        .collect();

    // Very easy to parallelise this, since both operations are totally independent of each other.
    let base_thread = thread::spawn(move || check_nixpkgs(&base_nixpkgs, base_config, &[]));
    let main_result = match check_nixpkgs(main_nixpkgs, main_config, &new_package_sets) {
        Ok(result) => result,
        Err(error) => {
            return error.into();
        }
    };

    let base_result = match base_thread.join() {
        Ok(Ok(status)) => status,
        Ok(Err(error)) => {
            return error.into();
        }
        Err(e) => panic::resume_unwind(e),
    };

    match (base_result, main_result) {
        (Failure(..), Failure(errors)) => Status::BranchStillBroken(errors),
        (Success(..), Failure(errors)) => Status::ProblemsIntroduced(errors),
        (Failure(..), Success(..)) => Status::BranchHealed,
        (Success(base), Success(main)) => {
            // Both base and main branch succeed. Check ratchet state between them...
            match ratchet::Nixpkgs::compare(&base, main) {
                Failure(errors) => Status::DiscouragedPatternedIntroduced(errors),
                Success(..) => Status::ValidatedSuccessfully,
            }
        }
    }
}

/// Checks whether the by-name structure at the given path in Nixpkgs is valid.
///
/// This does not include ratchet checks, see ../README.md#ratchet-checks
/// Instead a `ratchet::Nixpkgs` value is returned, whose `compare` method allows performing the
/// ratchet check against another result.
fn check_nixpkgs(
    nixpkgs_path: &Path,
    config: Config,
    new_package_sets: &[PkgSet],
) -> validation::Result<ratchet::Nixpkgs> {
    let nixpkgs_path = nixpkgs_path.canonicalize().with_context(|| {
        format!(
            "Nixpkgs path {} could not be resolved",
            nixpkgs_path.display()
        )
    })?;

    let mut nix_file_store = NixFileStore::default();

    let mut package_result = Success(BTreeMap::new());

    for pkgset in config.package_sets {
        let mut other_package_result =
            if !nixpkgs_path.join(pkgset.by_name_subpath.as_str()).exists() {
                // No directory at the given location (e.g. pkgs/by-name), always valid
                Success(BTreeMap::new())
            } else {
                let structure =
                    check_structure(&nixpkgs_path, &mut nix_file_store, &pkgset.by_name_subpath)?;

                // Only if we could successfully parse the structure, we do the evaluation checks
                structure.result_map(|package_names| {
                    eval::check_values(
                        &nixpkgs_path,
                        &pkgset.by_name_subpath,
                        &pkgset.attr_path,
                        &mut nix_file_store,
                        package_names.as_slice(),
                    )
                })?
            };

        // ignore certain problems for new package sets
        if new_package_sets
            .iter()
            .any(|x| x.attr_path == pkgset.attr_path)
        {
            other_package_result = other_package_result.map(|package_result_inner| {
                package_result_inner
                    .into_iter()
                    .filter(|(_full_attr_path, package)| {
                        // skip checks for packages that don't use by-name
                        matches!(package.uses_by_name, ratchet::RatchetState::Tight)
                    })
                    .collect()
            });
        }

        package_result = package_result.and(other_package_result, |mut self_, mut other| {
            self_.append(&mut other);
            self_
        });
    }

    let file_result = files::check_files(&nixpkgs_path, &mut nix_file_store)?;

    Ok(
        package_result.and(file_result, |packages, files| ratchet::Nixpkgs {
            packages,
            files,
        }),
    )
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use anyhow::Context;
    use pretty_assertions::StrComparison;
    use tempfile::{TempDir, tempdir_in};

    use super::process;

    // Manually repeat this for each subdir under tests/ in order to disambiguate
    #[fixtures::fixtures(["tests/integration/*"])]
    #[allow(non_snake_case)] // the test directories don't use snake case, and the macro expansion causes a warning.
    #[test]
    fn test_integration(path: &Path) -> anyhow::Result<()> {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if !path.is_dir() {
            return Ok(());
        }

        let expected_errors = fs::read_to_string(path.join("expected"))
            .with_context(|| format!("No expected file for test {name}"))?;

        test_nixpkgs(&name, path, &expected_errors);
        Ok(())
    }

    // tempfile::tempdir needs to be wrapped in temp_env lock
    // because it accesses TMPDIR environment variable.
    pub fn tempdir() -> anyhow::Result<TempDir> {
        let empty_list: [(&str, Option<&str>); 0] = [];
        Ok(temp_env::with_vars(empty_list, tempfile::tempdir)?)
    }

    // We cannot check case-conflicting files into Nixpkgs (the channel would fail to build),
    // so we generate the case-conflicting file instead.
    #[test]
    fn test_case_sensitive() -> anyhow::Result<()> {
        let temp_nixpkgs = tempdir()?;
        let path = temp_nixpkgs.path();

        if is_case_insensitive_fs(path)? {
            eprintln!("We're on a case-insensitive filesystem, skipping case-sensitivity test");
            return Ok(());
        }

        let base = path.join("main/pkgs/by-name");
        fs::create_dir_all(path.join("main"))?;
        fs::write(
            path.join("main/nixpkgs-vet.json"),
            include_str!("../tests/tests-nixpkgs-vet.json"),
        )?;

        fs::create_dir_all(base.join("fo/foo"))?;
        fs::write(base.join("fo/foo/package.nix"), "{ someDrv }: someDrv")?;

        fs::create_dir_all(base.join("fo/foO"))?;
        fs::write(base.join("fo/foO/package.nix"), "{ someDrv }: someDrv")?;

        test_nixpkgs(
            "case_sensitive",
            path,
            "- pkgs/by-name/fo: Duplicate case-sensitive package directories \"foO\" and \"foo\". (https://github.com/NixOS/nixpkgs-vet/wiki/NPV-111)\n\
            This PR introduces the problems listed above. Please fix them before merging, \
            otherwise the base branch would break.\n",
        );
        Ok(())
    }

    /// Tests symlinked temporary directories.
    ///
    /// This is needed because on Darwin, `/tmp` is a symlink to `/private/tmp`, and Nix's
    /// restrict-eval doesn't also allow access to the canonical path when you allow the
    /// non-canonical one.
    ///
    /// The error if we didn't do this would look like this:
    /// error: access to canonical path
    /// '/private/var/folders/[...]/.tmpFbcNO0' is forbidden in restricted mode
    #[test]
    fn test_symlinked_tmpdir() -> anyhow::Result<()> {
        // Create a directory with two entries:
        // - actual (dir)
        // - symlinked -> actual (symlink)
        let temp_root = tempdir()?;
        fs::create_dir(temp_root.path().join("actual"))?;
        std::os::unix::fs::symlink("actual", temp_root.path().join("symlinked"))?;
        let tmpdir = temp_root.path().join("symlinked");

        temp_env::with_var("TMPDIR", Some(&tmpdir), || {
            test_nixpkgs(
                "symlinked_tmpdir",
                Path::new("tests/integration/success"),
                "Validated successfully\n",
            );
        });
        Ok(())
    }

    fn test_nixpkgs(name: &str, path: &Path, expected_errors: &str) {
        // Match the expected errors almost verbatim -- `@REDACTED@` turns into `.*`.
        let pattern = format!(
            "^{}$",
            regex::escape(expected_errors).replace("@REDACTED@", ".*")
        );

        let expected_errors_regex = regex::RegexBuilder::new(&pattern)
            .dot_matches_new_line(true)
            .build()
            .expect("valid regex");

        let main_path = path.join("main");
        let base_path = path.join("base");
        let base_nixpkgs = if base_path.exists() {
            base_path
        } else {
            Path::new("tests/integration/empty-base").to_owned()
        };

        // Empty dir, needed so that no warnings are printed when testing older Nix versions
        // that don't recognise certain newer keys in nix.conf
        let nix_conf_dir = tempdir().expect("directory");
        let nix_conf_dir = nix_conf_dir.path().as_os_str();

        let status = temp_env::with_var("NIX_CONF_DIR", Some(nix_conf_dir), || {
            process(base_nixpkgs, &main_path)
        });

        let actual_errors = format!("{status}\n");

        assert!(
            expected_errors_regex.is_match(&actual_errors),
            "Failed test case {name}: {}",
            StrComparison::new(expected_errors, &actual_errors)
        );
    }

    /// Check whether a path is in a case-insensitive filesystem
    fn is_case_insensitive_fs(path: &Path) -> anyhow::Result<bool> {
        let dir = tempdir_in(path)?;
        let base = dir.path();
        fs::write(base.join("aaa"), "")?;
        Ok(base.join("AAA").exists())
    }
}
