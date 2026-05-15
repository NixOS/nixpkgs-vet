use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

use anyhow::Context;
use relative_path::RelativePathBuf;
use serde::Deserialize;

use crate::nix_file::CallPackageArgumentInfo;
use crate::problem::{npv_100, npv_101, npv_120, npv_180, npv_181, npv_182};
use crate::ratchet::RatchetState::{Loose, Tight};
use crate::structure::BASE_SUBPATH;
use crate::validation::ResultIteratorExt as _;
use crate::validation::{self, Validation::Success};
use crate::{NixFileStore, structure};
use crate::{location, ratchet};

const EVAL_NIX: &[u8] = include_bytes!("eval.nix");

/// Attribute set of this structure is returned by `./eval.nix`
#[derive(Deserialize)]
enum Attribute {
    /// An attribute that should be defined via `pkgs/by-name`.
    ByName(ByNameAttribute),
    /// An attribute not defined via `pkgs/by-name`.
    NonByName(NonByNameAttribute),
}

#[derive(Deserialize)]
enum NonByNameAttribute {
    /// The attribute doesn't evaluate.
    EvalFailure,
    EvalSuccess(AttributeInfo),
}

#[derive(Deserialize)]
enum ByNameAttribute {
    /// The attribute doesn't exist at all.
    Missing,
    Existing(AttributeInfo),
}

#[derive(Deserialize)]
struct AttributeInfo {
    /// The location of the attribute as returned by `builtins.unsafeGetAttrPos`.
    location: Option<Location>,
    attribute_variant: AttributeVariant,
}

/// The structure returned by a successful `builtins.unsafeGetAttrPos`.
#[derive(Deserialize, Clone, Debug)]
struct Location {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
}

impl Location {
    /// Returns the location, but relative to the given Nixpkgs root.
    fn relative(self, nixpkgs_path: &Path) -> anyhow::Result<location::Location> {
        let Self { file, line, column } = self;
        let path = file.strip_prefix(nixpkgs_path).with_context(|| {
            format!(
                "The file ({}) is outside Nixpkgs ({})",
                file.display(),
                nixpkgs_path.display()
            )
        })?;
        let relative_file = RelativePathBuf::from_path(path).expect("relative path");
        Ok(location::Location::new(relative_file, line, column))
    }
}

#[derive(Deserialize)]
pub enum AttributeVariant {
    /// The attribute is not an attribute set, so we're limited in the amount of information we can
    /// get from it. Since all derivations are attribute sets, it's obviously not a derivation.
    NonAttributeSet,
    AttributeSet {
        /// Whether the attribute is a derivation (`lib.isDerivation`)
        is_derivation: bool,
        /// Whether the derivation has any maintainers
        has_maintainers: bool,
        /// Attribute that should correspond to whether the package has no maintainers but packages depending on it
        has_no_maintainers_but_dependents: bool,
        /// Whether the attribute evaluates with `strictDeps = true`.
        strict_deps: bool,
        /// Whether the attribute evaluates with `__structuredAttrs = true`.
        structured_attrs: bool,
        /// Whether the attribute was defined via `callPackage` of the same scope
        is_same_scope_call_package: bool,
    },
}

/// Pass through variables needed to make Nix evaluation work inside Nix build. See `initNix`.
/// If these variables don't exist, assume we're not in a Nix sandbox.
fn pass_through_environment_variables_for_nix_eval_in_nix_build(command: &mut process::Command) {
    for variable in [
        "NIX_CONF_DIR",
        "NIX_LOCALSTATE_DIR",
        "NIX_LOG_DIR",
        "NIX_STATE_DIR",
        "NIX_STORE_DIR",
    ] {
        if let Ok(value) = env::var(variable) {
            command.env(variable, value);
        }
    }
}

#[cfg(not(test))]
#[allow(clippy::unnecessary_wraps)]
fn mutate_nix_instatiate_arguments_based_on_cfg(
    _work_dir_path: &Path,
    command: &mut process::Command,
) -> anyhow::Result<()> {
    command.arg("--show-trace");

    Ok(())
}

/// Tests need to be able to mock out `<nixpkgs>`; do that for them.
#[cfg(test)]
fn mutate_nix_instatiate_arguments_based_on_cfg(
    work_dir_path: &Path,
    command: &mut process::Command,
) -> anyhow::Result<()> {
    const MOCK_NIXPKGS: &[u8] = include_bytes!("../tests/mock-nixpkgs.nix");
    let mock_nixpkgs_path = work_dir_path.join("mock-nixpkgs.nix");
    fs::write(&mock_nixpkgs_path, MOCK_NIXPKGS)?;

    // Wire it up so that it can be imported as `import <test-nixpkgs> { }`.
    command.arg("-I");
    command.arg(format!("test-nixpkgs={}", mock_nixpkgs_path.display()));

    // Retrieve the path to the real nixpkgs lib, then wire it up to `import <test-nixpkgs/lib>`.
    let nixpkgs_lib = env::var("NIXPKGS_VET_NIXPKGS_LIB")
        .with_context(|| "Could not get environment variable NIXPKGS_VET_NIXPKGS_LIB")?;

    command.arg("-I");
    command.arg(format!("test-nixpkgs/lib={nixpkgs_lib}"));

    Ok(())
}

/// Check that the Nixpkgs attribute values corresponding to the packages in `pkgs/by-name` are of
/// the form `callPackage <package_file> { ... }`. See the `./eval.nix` file for how this is
/// achieved on the Nix side.
///
/// The validation result is a map from package names to a package ratchet state.
pub fn check_values(
    nixpkgs_path: &Path,
    nix_file_store: &mut NixFileStore,
    idents_to_files: &BTreeMap<String, BTreeSet<RelativePathBuf>>,
    package_names: &[String],
) -> validation::Result<BTreeMap<Vec<String>, ratchet::Package>> {
    let work_dir = tempfile::Builder::new()
        .prefix("nixpkgs-vet")
        .tempdir()
        .with_context(|| "Failed to create a working directory")?;

    // Canonicalize the path so that if a symlink were returned, we wouldn't ask Nix to follow it.
    let work_dir_path = work_dir.path().canonicalize()?;

    // Write the list of packages we need to check into a temporary JSON file.
    let package_names_path = work_dir_path.join("package-names.json");
    let package_names_file = fs::File::create(&package_names_path)?;
    serde_json::to_writer(&package_names_file, &package_names).with_context(|| {
        format!(
            "Failed to serialise the package names to the work dir {}",
            work_dir_path.display()
        )
    })?;

    // Write the Nix file into the work directory.
    let eval_nix_path = work_dir_path.join("eval.nix");
    fs::write(&eval_nix_path, EVAL_NIX)?;

    // Pinning Nix in this way makes the tool more reproducible
    let nix_package = env::var("NIXPKGS_VET_NIX_PACKAGE")
        .with_context(|| "Could not get environment variable NIXPKGS_VET_NIX_PACKAGE")?;

    // With restrict-eval, only paths in NIX_PATH can be accessed. We explicitly specify them here.
    let mut command = process::Command::new(format!("{nix_package}/bin/nix-instantiate"));
    command
        // Capture stderr so that it can be printed later in case of failure
        .stderr(process::Stdio::piped())
        // Clear environment so that nothing from the outside influences this `nix-instantiate`.
        .env_clear()
        .args([
            "--eval",
            "--json",
            "--strict",
            "--readonly-mode",
            "--restrict-eval",
        ])
        // Add the work directory to the NIX_PATH so that it can be accessed in restrict-eval mode.
        .arg("-I")
        .arg(&work_dir_path)
        .args(["--arg", "attrsPath"])
        .arg(&package_names_path)
        // Same for the nixpkgs to test, adding it to the NIX_PATH so it can be accessed in
        // restrict-eval mode.
        .args(["--arg", "nixpkgsPath"])
        .arg(nixpkgs_path)
        .arg("-I")
        .arg(nixpkgs_path);

    pass_through_environment_variables_for_nix_eval_in_nix_build(&mut command);
    mutate_nix_instatiate_arguments_based_on_cfg(&work_dir_path, &mut command)?;

    command.arg(eval_nix_path);

    let result = command
        .output()
        .with_context(|| format!("Failed to run command {command:?}"))?;

    if !result.status.success() {
        // Early return in case evaluation fails
        return Ok(npv_120::NixEvalError::new(String::from_utf8_lossy(&result.stderr)).into());
    }

    // Parse the resulting JSON value
    let attributes: Vec<(Vec<String>, Attribute)> = serde_json::from_slice(&result.stdout)
        .with_context(|| {
            format!(
                "Failed to deserialise {}",
                String::from_utf8_lossy(&result.stdout)
            )
        })?;

    let check_result = validation::sequence(
        attributes
            .into_iter()
            .map(|(attribute_path, attribute_value)| {
                let check_result = match attribute_value {
                    Attribute::NonByName(non_by_name_attribute) => handle_non_by_name_attribute(
                        nixpkgs_path,
                        nix_file_store,
                        idents_to_files,
                        &attribute_path,
                        non_by_name_attribute,
                    )?,
                    Attribute::ByName(by_name_attribute) => {
                        assert!(attribute_path.len() == 1);
                        by_name(idents_to_files, &attribute_path[0], by_name_attribute)?
                    }
                };
                Ok::<_, anyhow::Error>(check_result.map(|value| (attribute_path.clone(), value)))
            })
            .collect_vec()?,
    );

    Ok(check_result.map(|elems| elems.into_iter().collect()))
}

/// Handle the evaluation result for an attribute in `pkgs/by-name`, making it a validation result.
fn by_name(
    idents_to_files: &BTreeMap<String, BTreeSet<RelativePathBuf>>,
    attribute_name: &str,
    by_name_attribute: ByNameAttribute,
) -> validation::Result<ratchet::Package> {
    // At this point we know that `pkgs/by-name/fo/foo/package.nix` has to exist. This match
    // decides whether the attribute `foo` is defined accordingly.
    let result = match by_name_attribute {
        // The attribute is missing.
        ByNameAttribute::Missing => {
            // This indicates a bug in the `pkgs/by-name` overlay, because it's supposed to
            // automatically defined attributes in `pkgs/by-name`
            npv_100::ByNameUndefinedAttribute::new(attribute_name).into()
        }
        // The attribute exists
        ByNameAttribute::Existing(AttributeInfo {
            // But it's not an attribute set, which limits the amount of information we can get
            // about this attribute (see ./eval.nix)
            attribute_variant: AttributeVariant::NonAttributeSet,
            location: _location,
        }) => {
            // The only thing we know is that it's definitely not a derivation, since those are
            // always attribute sets.
            //
            // We can't know whether the attribute is automatically or manually defined for sure,
            // and while we could check the location, the error seems clear enough as is.
            npv_101::ByNameNonDerivation::new(attribute_name).into()
        }
        // The attribute exists
        ByNameAttribute::Existing(AttributeInfo {
            // And it's an attribute set, which allows us to get more information about it
            attribute_variant:
                AttributeVariant::AttributeSet {
                    is_derivation,
                    strict_deps,
                    structured_attrs,
                    is_same_scope_call_package: _,
                    has_maintainers,
                    has_no_maintainers_but_dependents,
                },
            location: _,
        }) => {
            // Only derivations are allowed in `pkgs/by-name`.
            let is_derivation_result = if is_derivation {
                Success(())
            } else {
                npv_101::ByNameNonDerivation::new(attribute_name).into()
            };

            let leaf_result = leaf_result(
                has_maintainers,
                has_no_maintainers_but_dependents,
                &vec![attribute_name.to_string()],
                idents_to_files,
                &structure::relative_dir_for_package(attribute_name),
                &structure::relative_file_for_package(attribute_name),
            );

            // Independently report problems about whether it's a derivation and its leaf status.
            leaf_result
                .and_(is_derivation_result)
                .map(|()| ratchet::Package {
                    uses_by_name: Tight,
                    strict_deps: enabled_attribute_ratchet(
                        strict_deps,
                        structure::relative_file_for_package(attribute_name),
                    ),
                    structured_attrs: enabled_attribute_ratchet(
                        structured_attrs,
                        structure::relative_file_for_package(attribute_name),
                    ),
                })
        }
    };
    Ok(result)
}

fn leaf_result(
    has_maintainers: bool,
    has_no_maintainers_but_dependents: bool,
    attribute_path: &Vec<String>,
    idents_to_files: &BTreeMap<String, BTreeSet<RelativePathBuf>>,
    tree_to_ignore: &RelativePathBuf,
    package_file: &RelativePathBuf,
) -> validation::Validation<()> {
    if has_maintainers {
        if has_no_maintainers_but_dependents {
            npv_181::DependentsAttrsSetWithMaintainers::new(attribute_path.clone()).into()
        } else {
            Success(())
        }
    } else {
        let mut dependent_files: BTreeMap<String, BTreeSet<RelativePathBuf>> = BTreeMap::new();

        for attr in attribute_path {
            let files = if let Some(referenced_by_files) = idents_to_files.get(attr) {
                referenced_by_files
                    .clone()
                    .into_iter()
                    .filter(|file| !file.starts_with(tree_to_ignore))
                    .collect()
            } else {
                BTreeSet::new()
            };
            dependent_files.insert(attr.clone(), files);
        }

        // All attributes need a reference
        if dependent_files.values().all(|files| !files.is_empty()) {
            // Has potential dependents
            if has_no_maintainers_but_dependents {
                Success(())
            } else {
                npv_180::DependentsAttrsShouldBeSet::new(
                    attribute_path.clone(),
                    package_file,
                    dependent_files,
                )
                .into()
            }
        } else {
            // Has almost definitely no dependents
            if has_no_maintainers_but_dependents {
                npv_182::DependentsAttrsSetWithoutDependents::new(attribute_path.clone()).into()
            } else {
                Success(())
            }
        }
    }
}

fn enabled_attribute_ratchet<R>(enabled: bool, file: RelativePathBuf) -> ratchet::RatchetState<R>
where
    R: ratchet::ToProblem<ToContext = RelativePathBuf>,
{
    if enabled { Tight } else { Loose(file) }
}

/// Handles the evaluation result for an attribute _not_ in `pkgs/by-name`, turning it into a
/// validation result.
fn handle_non_by_name_attribute(
    nixpkgs_path: &Path,
    nix_file_store: &mut NixFileStore,
    idents_to_files: &BTreeMap<String, BTreeSet<RelativePathBuf>>,
    attribute_path: &Vec<String>,
    non_by_name_attribute: NonByNameAttribute,
) -> validation::Result<ratchet::Package> {
    use NonByNameAttribute::EvalSuccess;
    use ratchet::RatchetState::{Loose, NonApplicable, Tight};

    // The ratchet state for this package.
    let package_result = match non_by_name_attribute {
        // This is a big ol' match on various properties of the attribute
        //
        // First, it needs to succeed evaluation. We can't know whether an attribute could be
        // migrated to `pkgs/by-name` if it doesn't evaluate, since we need to check that it's a
        // derivation.
        //
        // This only has the minor negative effect that if a PR that breaks evaluation gets merged,
        // fixing those failures won't force anything into `pkgs/by-name`.
        //
        // For now this isn't our problem, but in the future we might have another check to enforce
        // that evaluation must not be broken.
        //
        // The alternative of assuming that failing attributes would have been fit for
        // `pkgs/by-name` has the problem that if a package evaluation gets broken temporarily,
        // fixing it requires a move to pkgs/by-name, which could happen more often and isn't
        // really justified.
        EvalSuccess(AttributeInfo {
            // We're only interested in attributes that are attribute sets, which all derivations
            // are. Anything else can't be in `pkgs/by-name`.
            attribute_variant:
                AttributeVariant::AttributeSet {
                    // As of today, non-derivation attribute sets can't be in `pkgs/by-name`.
                    is_derivation: true,
                    strict_deps,
                    structured_attrs,
                    is_same_scope_call_package,
                    has_maintainers,
                    has_no_maintainers_but_dependents,
                },
            location: Some(location),
        }) => {
            // Parse the Nix file in the location
            let nix_file = nix_file_store.get(&location.file)?;

            // The relative location of the Nix file, for error messages
            let location = location.relative(nixpkgs_path).with_context(|| {
                format!(
                    "Failed to resolve the file where attribute {} is defined",
                    attribute_path.join("."),
                )
            })?;

            // Figure out whether it's an attribute definition of the form
            // `= callPackage <arg1> <arg2>`, returning the arguments if so.
            let (optional_syntactic_call_package, _definition) = nix_file
                .call_package_argument_info_at(location.line, location.column, nixpkgs_path)
                .with_context(|| {
                    format!(
                        "Failed to get the definition info for attribute {}",
                        attribute_path.join("."),
                    )
                })?;

            // This is never `Tight`, because we only either:
            // - Know that the attribute _could_ be migrated to `pkgs/by-name`, which is `Loose`
            // - Or we're unsure, in which case we use `NonApplicable`
            let uses_by_name_result =
                // At this point, we completed two different checks for whether it's a
                // `callPackage`.
                match (is_same_scope_call_package, &optional_syntactic_call_package) {
                        // Something like `<attr> = somethingThatEvaluatesToAPackage`
                        // TODO: We still need to run package checks on this one
                        (false, None)
                        // Something like `<attr> = pythonPackages.callPackage ...`
                        // TODO: We still need to run package checks on this one
                        // TODO: Also need to run leaf checks on this one
                        | (false, Some(_))
                        // Something like `<attr> = bar` where `bar = pkgs.callPackage ...`
                        // TODO: We still need to run package checks on this one
                        | (true, None) => Success(NonApplicable),
                    // Something like `<attr> = pkgs.callPackage ...`
                    // TODO: We still need to run package checks on this one
                    // TODO: Also need to run leaf checks on this one
                    (true, Some(syntactic_call_package)) => {
                        // It's only possible to migrate such definitions if..
                        match syntactic_call_package.relative_path {
                            Some(ref rel_path) if rel_path.starts_with(BASE_SUBPATH) => {
                                // ..the path is not already within `pkgs/by-name` like
                                //
                                //   foo-variant = callPackage ../by-name/fo/foo/package.nix {
                                //     someFlag = true;
                                //   }
                                //
                                // While such definitions could be moved to `pkgs/by-name` by
                                // using `.override { someFlag = true; }` instead, this changes
                                // the semantics in relation with overlays, so migration is
                                // generally not possible.
                                //
                                // See also "package variants" in RFC 140:
                                // https://github.com/NixOS/rfcs/blob/master/rfcs/0140-simple-package-paths.md#package-variants
                                Success(NonApplicable)
                            }
                            _ => Success(Loose((syntactic_call_package.clone(), location.file.clone()))),
                        }
                    }
                };

            let leaf_result = if let Some(CallPackageArgumentInfo {
                relative_path: Some(rel_path),
                ..
            }) = optional_syntactic_call_package.as_ref()
            {
                let file = if rel_path.to_logical_path(nixpkgs_path).is_dir() {
                    &rel_path.join("default.nix")
                } else {
                    rel_path
                };
                leaf_result(
                    has_maintainers,
                    has_no_maintainers_but_dependents,
                    attribute_path,
                    idents_to_files,
                    file,
                    file,
                )
            } else {
                Success(())
            };

            // For evaluated boolean ratchets, point at the package file when we can resolve one.
            // Otherwise fall back to the file that defines the top-level attribute.
            let evaluated_attribute_file = optional_syntactic_call_package
                .as_ref()
                .and_then(|call_package| call_package.relative_path.clone())
                .unwrap_or_else(|| location.file.clone());

            let strict_deps = match (strict_deps, evaluated_attribute_file.clone()) {
                (true, _) => Tight,
                (false, file) => Loose(file),
            };

            let structured_attrs = match (structured_attrs, evaluated_attribute_file) {
                (true, _) => Tight,
                (false, file) => Loose(file),
            };

            leaf_result
                .and_(uses_by_name_result)
                .map(|uses_by_name| ratchet::Package {
                    uses_by_name,
                    strict_deps,
                    structured_attrs,
                })
        }
        // This catches all the cases not matched by the above `EvalSuccess`, falling back to not
        // being able to make any good calls about the ratchet state.
        _ => Success(ratchet::Package {
            uses_by_name: NonApplicable,
            strict_deps: NonApplicable,
            structured_attrs: NonApplicable,
        }),
    };
    Ok(package_result)
}
