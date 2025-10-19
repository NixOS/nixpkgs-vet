use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

use anyhow::Context;
use relative_path::RelativePathBuf;
use serde::Deserialize;

use crate::nix_file::CallPackageArgumentInfo;
use crate::problem::{
    npv_100, npv_101, npv_102, npv_103, npv_104, npv_105, npv_106, npv_107, npv_108, npv_120,
};
use crate::ratchet::RatchetState::{Loose, Tight};
use crate::structure::{self, ByNameDir, Config};
use crate::validation::ResultIteratorExt as _;
use crate::validation::{self, Validation::Success};
use crate::NixFileStore;
use crate::{location, ratchet};

const EVAL_NIX: &[u8] = include_bytes!("eval.nix");

/// Attribute set of this structure is returned by `./eval.nix`
#[derive(Deserialize, Debug)]
enum Attribute {
    /// An attribute that should be defined via a `by-name` directory.
    ByName(ByNameAttribute),
    /// An attribute not defined via a `by-name` directory.
    NonByName(NonByNameAttribute),
}

#[derive(Deserialize, Debug)]
enum NonByNameAttribute {
    /// The attribute doesn't evaluate.
    EvalFailure,
    EvalSuccess(AttributeInfo),
}

#[derive(Deserialize, Debug)]
enum ByNameAttribute {
    /// The attribute doesn't exist at all.
    Missing,
    Existing(AttributeInfo),
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub enum AttributeVariant {
    /// The attribute is not an attribute set, so we're limited in the amount of information we can
    /// get from it. Since all derivations are attribute sets, it's obviously not a derivation.
    NonAttributeSet,
    AttributeSet {
        /// Whether the attribute is a derivation (`lib.isDerivation`)
        is_derivation: bool,
        /// The type of `callPackage` used.
        definition_variant: DefinitionVariant,
    },
}

#[derive(Deserialize, Debug)]
pub enum DefinitionVariant {
    /// An automatic definition by the `by-name` overlay, though it's detected using the
    /// internal `_internalCallByNamePackageFile` attribute, which can in theory also be used by
    /// other code.
    AutoDefinition,
    /// A manual definition of the attribute, typically in `all-packages.nix`.
    ManualDefinition {
        /// Whether the attribute is defined as `pkgs.callPackage ...` or something else.
        is_semantic_call_package: bool,
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
    _config_file_path: &PathBuf,
) -> anyhow::Result<()> {
    command.arg("--show-trace");

    Ok(())
}

/// Tests need to be able to mock out `<nixpkgs>`; do that for them.
#[cfg(test)]
fn mutate_nix_instatiate_arguments_based_on_cfg(
    work_dir_path: &Path,
    command: &mut process::Command,
    config_file_path: &PathBuf,
) -> anyhow::Result<()> {
    println!("{}:{}: work dir path: {work_dir_path:?}",file!(), line!());
    const MOCK_NIXPKGS: &[u8] = include_bytes!("../tests/mock-nixpkgs.nix");
    let mock_nixpkgs_path = work_dir_path.join("mock-nixpkgs.nix");
    fs::write(&mock_nixpkgs_path, MOCK_NIXPKGS)?;

    let test_config_file_path = work_dir_path.join("by-name-config.json");
    fs::write(&test_config_file_path, fs::read(&config_file_path)?)?;

    // Wire it up so that it can be imported as `import <test-nixpkgs> { }`.
    command.arg("-I");
    command.arg(format!("test-nixpkgs={}", mock_nixpkgs_path.display()));

    // Retrieve the path to the real nixpkgs lib, then wire it up to `import <test-nixpkgs/lib>`.
    let nixpkgs_lib = env::var("NIXPKGS_VET_NIXPKGS_LIB")
        .with_context(|| "Could not get environment variable NIXPKGS_VET_NIXPKGS_LIB")?;

    command.arg("-I");
    command.arg(format!("test-nixpkgs/lib={nixpkgs_lib}"));

    command.arg("--argstr");
    command.arg("configPath");
    command.arg(test_config_file_path);

    command.arg("--show-trace");
    command.arg("--verbose");
    command.arg("--debug");

    Ok(())
}

/// Check that the Nixpkgs attribute values corresponding to the packages in a `by-name` directory are of
/// the form `callPackage <package_file> { ... }`. See the `./eval.nix` file for how this is
/// achieved on the Nix side.
///
/// The validation result is a map from package names to a package ratchet state.
pub fn check_values(
    nixpkgs_path: &Path,
    nix_file_store: &mut NixFileStore,
    packages: &[(String, String)],
    by_name_dir: &ByNameDir,
    config: &Config,
) -> validation::Result<BTreeMap<String, ratchet::Package>> {
    let work_dir = tempfile::Builder::new()
        .prefix("nixpkgs-vet")
        .tempdir()
        .with_context(|| "Failed to create a working directory")?;

    // Canonicalize the path so that if a symlink were returned, we wouldn't ask Nix to follow it.
    let work_dir_path = work_dir.path().canonicalize()?;
    let tmp = nixpkgs_path.join(by_name_dir.path.as_str());
    let full_path = tmp.to_string_lossy();

    println!(
        "{}:{}: package_names for {full_path}: {}",
        file!(), line!(),
        packages
            .iter()
            .map(|x| x.0.to_owned())
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Write the list of packages we need to check into a temporary JSON file.
    let package_names_path = work_dir_path.join("package-names.json");
    let package_names_file = fs::File::create(&package_names_path)?;
    serde_json::to_writer(
        &package_names_file,
        &packages.iter().map(|x| x.1.to_owned()).collect::<Vec<_>>(),
    )
    .with_context(|| {
        format!(
            "Failed to serialise the package names to the work dir {}",
            work_dir_path.display()
        )
    })?;

    let config_file_path = work_dir_path.join("by-name-config.json");
    let config_file = fs::File::create(&config_file_path)?;
    serde_json::to_writer(&config_file, &config).with_context(|| {
        format!(
            "Failed to serialise the config file to the work dir {}",
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
    mutate_nix_instatiate_arguments_based_on_cfg(&work_dir_path, &mut command, &config_file_path)?;

    command.arg(eval_nix_path);

    let result = command
        .output()
        .with_context(|| format!("Failed to run command {command:?}"))?;
        
    println!(
        "{}:{}: result for {full_path} (stdout): {}",
        file!(), line!(),
        String::from_utf8(result.stdout.clone()).unwrap(),
    );
    println!(
        "{}:{}: result for {full_path} (stderr): {}",
        file!(), line!(),
        String::from_utf8(result.stderr.clone()).unwrap(),
    );

    if !result.status.success() {
        println!("{}:{}: : eval failed for {full_path}" , file!(), line!());
        // Early return in case evaluation fails
        let foo =  Ok(npv_120::NixEvalError::new(
            String::from_utf8_lossy(&result.stderr),
            by_name_dir.clone(),
        )
        .into());
        return foo
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
            .map(|(attribute_name, attribute_value)| {
                println!("{}:{}: : attribute_name: {attribute_name:?}; attribute_value: {attribute_value:?}", file!(), line!());
                let check_result = match attribute_value {
                    Attribute::NonByName(non_by_name_attribute) => handle_non_by_name_attribute(
                        nixpkgs_path,
                        nix_file_store,
                        &attribute_name.join("."),
                        non_by_name_attribute,
                        config,
                        by_name_dir,
                    )?,
                    Attribute::ByName(by_name_attribute) => by_name(
                        nix_file_store,
                        nixpkgs_path,
                        &attribute_name.join("."),
                        by_name_attribute,
                        config,
                        by_name_dir,
                    )?,
                };
                Ok::<_, anyhow::Error>(check_result.map(|value| (attribute_name.clone(), value)))
            })
            .collect_vec()?,
    );

    Ok(check_result.map(|elems| {
        elems
            .into_iter()
            .map(|(attribute_name, package)| (attribute_name.join("."), package))
            .into_iter()
            .collect()
    }))
}

/// Handle the evaluation result for an attribute in a `by-name` directory, making it a validation result.
fn by_name(
    nix_file_store: &mut NixFileStore,
    nixpkgs_path: &Path,
    attribute_name: &str,
    by_name_attribute: ByNameAttribute,
    config: &Config,
    by_name_dir: &ByNameDir,
) -> validation::Result<ratchet::Package> {
    println!("{}:{}:  attribute_name: {attribute_name}; by_name_attribute: {by_name_attribute:?}", file!(), line!());
    // At this point we know that `pkgs/by-name/fo/foo/package.nix` has to exist.  This match
    // decides whether the attribute `foo` is defined accordingly and whether a legacy manual
    // definition could be removed.
    let manual_definition_result = match by_name_attribute {
        // The attribute is missing.
        ByNameAttribute::Missing => {
            // This indicates a bug in the `pkgs/by-name` overlay, because it's supposed to
            // automatically define attributes in a `by-name` directory
            npv_100::ByNameUndefinedAttribute::new(attribute_name, by_name_dir.clone()).into()
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
            npv_101::ByNameNonDerivation::new(attribute_name, by_name_dir.clone()).into()
        }
        // The attribute exists
        ByNameAttribute::Existing(AttributeInfo {
            // And it's an attribute set, which allows us to get more information about it
            attribute_variant:
                AttributeVariant::AttributeSet {
                    is_derivation,
                    definition_variant,
                },
            location,
        }) => {
            // Only derivations are allowed in a `by-name` directory
            let is_derivation_result = if is_derivation {
                Success(())
            } else {
                npv_101::ByNameNonDerivation::new(attribute_name, by_name_dir.clone()).into()
            };

            // If the definition looks correct
            let variant_result = match definition_variant {
                // An automatic `callPackage` by the `pkgs/by-name` overlay.
                // Though this gets detected by checking whether the internal
                // `_internalCallByNamePackageFile` was used
                DefinitionVariant::AutoDefinition => location.map_or_else(
                    || Success(Tight),
                    // Such an automatic definition should definitely not have a location.
                    // Having one indicates that somebody is using
                    // `_internalCallByNamePackageFile`,
                    |_location| npv_102::ByNameInternalCallPackageUsed::new(attribute_name).into(),
                ),
                // The attribute is manually defined, e.g. in `all-packages.nix`.
                // This means we need to enforce it to look like this:
                //   callPackage ../pkgs/by-name/fo/foo/package.nix { ... }
                DefinitionVariant::ManualDefinition {
                    is_semantic_call_package,
                } => {
                    // We should expect manual definitions to have a location, otherwise we can't
                    // enforce the expected format
                    if let Some(location) = location {
                        // Parse the Nix file in the location
                        let nix_file = nix_file_store.get(&location.file)?;

                        // The relative path of the Nix file, for error messages
                        let location = location.relative(nixpkgs_path).with_context(|| {
                            format!(
                                "Failed to resolve the file where attribute {attribute_name} is defined"
                            )
                        })?;

                        // Figure out whether it's an attribute definition of the form
                        // `= callPackage <arg1> <arg2>`, returning the arguments if so.
                        let (optional_syntactic_call_package, definition) = nix_file
                            .call_package_argument_info_at(
                                location.line,
                                location.column,
                                nixpkgs_path,
                            )
                            .with_context(|| {
                                format!(
                                    "Failed to get the definition info for attribute {}",
                                    attribute_name
                                )
                            })?;

                        println!("{}:{}: : attribute_name: {attribute_name}; is_semantic_call_package: {is_semantic_call_package}; optional_syntactic_call_package: {optional_syntactic_call_package:?}; definition: {definition}; location: {location:?}", file!(), line!());

                        by_name_override(
                            attribute_name,
                            is_semantic_call_package,
                            optional_syntactic_call_package,
                            definition,
                            location,
                            config,
                            by_name_dir,
                        )
                    } else {
                        // If manual definitions don't have a location, it's likely `mapAttrs`'d
                        // over, e.g. if it's defined in aliases.nix.
                        // We can't verify whether its of the expected `callPackage`, so error out.
                        npv_103::ByNameCannotDetermineAttributeLocation::new(attribute_name).into()
                    }
                }
            };

            println!("{}:{}: attribute_name: {attribute_name}; is_derivation_result: {is_derivation_result:?}; variant_result: {variant_result:?}", file!(), line!());
            // Independently report problems about whether it's a derivation and the callPackage
            // variant.
            is_derivation_result.and_(variant_result)
        }
    };
    let result = Ok(
        // Packages being checked in this function are _always_ already defined in a `by-name` directory,
        // so instead of repeating ourselves all the time to define `uses_by_name`, just set it
        // once at the end with a map.
        manual_definition_result.map(|manual_definition| ratchet::Package {
            manual_definition,
            uses_by_name: Tight,
        }),
    );
    println!("{}:{}: : result: {result:?}", file!(), line!());
    result
}

/// Handles the case for packages in a `by-name` directory that are manually overridden,
/// e.g. in `pkgs/top-level/all-packages.nix`.
fn by_name_override(
    attribute_name: &str,
    is_semantic_call_package: bool,
    optional_syntactic_call_package: Option<CallPackageArgumentInfo>,
    definition: String,
    location: location::Location,
    config: &Config,
    by_name_dir: &ByNameDir,
) -> validation::Validation<ratchet::RatchetState<ratchet::ManualDefinition>> {
    let Some(syntactic_call_package) = optional_syntactic_call_package else {
        // Something like `<attr> = foo`
        return npv_104::ByNameOverrideOfNonSyntacticCallPackage::new(
            attribute_name,
            location,
            definition,
            by_name_dir.clone(),
        )
        .into();
    };

    if !is_semantic_call_package {
        // Something like `<attr> = pythonPackages.callPackage ...`
        return npv_105::ByNameOverrideOfNonTopLevelPackage::new(
            attribute_name,
            location,
            definition,
            by_name_dir.clone(),
        )
        .into();
    }

    let Some(actual_package_path) = syntactic_call_package.relative_path else {
        return npv_108::ByNameOverrideContainsEmptyPath::new(
            attribute_name,
            location,
            definition,
            by_name_dir.clone(),
        )
        .into();
    };

    let expected_by_name_dir = structure::expected_by_name_dir_for_package(attribute_name, config);

    let expected_package_path =
        structure::relative_file_for_package(attribute_name, &expected_by_name_dir);
    if actual_package_path != expected_package_path {
        return npv_106::ByNameOverrideContainsWrongCallPackagePath::new(
            attribute_name,
            actual_package_path,
            location,
            by_name_dir.clone(),
        )
        .into();
    }

    // Manual definitions with empty arguments are not allowed anymore, but existing ones should
    // continue to be allowed. This is the state to migrate away from.
    if syntactic_call_package.empty_arg {
        println!("{}:{}: : here; attribute_name: {attribute_name}", file!(), line!());
        Success(Loose(
            npv_107::ByNameOverrideContainsEmptyArgument::new(
                attribute_name,
                location,
                definition,
                by_name_dir.clone(),
            )
            .into(),
        ))
    } else {
        // This is the state to migrate to.
        Success(Tight)
    }
}

/// Handles the evaluation result for an attribute _not_ in a `by-name`
/// directory, turning it into a validation result.
fn handle_non_by_name_attribute(
    nixpkgs_path: &Path,
    nix_file_store: &mut NixFileStore,
    attribute_name: &str,
    non_by_name_attribute: NonByNameAttribute,
    config: &Config,
    by_name_dir: &ByNameDir,
) -> validation::Result<ratchet::Package> {
    use ratchet::RatchetState::{Loose, NonApplicable, Tight};
    use NonByNameAttribute::EvalSuccess;
    println!("{}:{}: : attribute_name: {attribute_name}", file!(), line!());

    // The ratchet state whether this attribute uses a `by-name` directory
    //
    // This is never `Tight`, because we only either:
    // - Know that the attribute _could_ be migrated to a `by-name` directory, which is `Loose`
    // - Or we're unsure, in which case we use `NonApplicable`
    let uses_by_name =
        // This is a big ol' match on various properties of the attribute
        //
        // First, it needs to succeed evaluation. We can't know whether an attribute could be
        // migrated to a `by-name` directory if it doesn't evaluate, since we need to check that it's a
        // derivation.
        //
        // This only has the minor negative effect that if a PR that breaks evaluation gets merged,
        // fixing those failures won't force anything into a `by-name` directory.
        //
        // For now this isn't our problem, but in the future we might have another check to enforce
        // that evaluation must not be broken.
        //
        // The alternative of assuming that failing attributes would have been fit for
        // a `by-name` directory has the problem that if a package evaluation gets broken temporarily,
        // fixing it requires a move to a `by-name` directory, which could happen more often and isn't
        // really justified.
        if let EvalSuccess(AttributeInfo {
            // We're only interested in attributes that are attribute sets, which all derivations
            // are. Anything else can't be in a `by-name` directory.
            attribute_variant: AttributeVariant::AttributeSet {
                // As of today, non-derivation attribute sets can't be in a `by-name` directory.
                is_derivation: true,
                // Of the two definition variants, really only the manual one makes sense here.
                //
                // Special cases are:
                //
                // - Manual aliases to auto-called packages are not treated as manual definitions,
                //   due to limitations in the semantic `callPackage` detection.
                //   So those should be ignored.
                //
                // - Manual definitions using the internal `_internalCallByNamePackageFile` are
                //   not treated as manual definitions, since `_internalCallByNamePackageFile` is
                //   used to detect automatic ones. We can't distinguish from the above case, so we
                //   just need to ignore this one too, even if that internal attribute should never
                //   be called manually.
                definition_variant: DefinitionVariant::ManualDefinition {
                    is_semantic_call_package
                }
            },
            // We need the location of the manual definition, because otherwise we can't figure out
            // whether it's a syntactic `callPackage`.
            location: Some(location),
        }) = non_by_name_attribute {

        // Parse the Nix file in the location
        let nix_file = nix_file_store.get(&location.file)?;

        // The relative location of the Nix file, for error messages
        let location = location.relative(nixpkgs_path).with_context(|| {
            format!("Failed to resolve the file where attribute {attribute_name} is defined")
        })?;

        // Figure out whether it's an attribute definition of the form
        // `= callPackage <arg1> <arg2>`, returning the arguments if so.
        let (optional_syntactic_call_package, _definition) = nix_file
            .call_package_argument_info_at(
                location.line,
                location.column,
                // Passing the Nixpkgs path here both checks that the <arg1> is within Nixpkgs,
                // and strips the absolute Nixpkgs path from it, such that
                // syntactic_call_package.relative_path is relative to Nixpkgs
                nixpkgs_path
            )
            .with_context(|| {
                format!("Failed to get the definition info for attribute {}", attribute_name)
            })?;

        println!("{}:{}:  attribute_name: {attribute_name}; is_semantic_call_package: {is_semantic_call_package}; optional_syntactic_call_package: {optional_syntactic_call_package:?}", file!(), line!());
        // At this point, we completed two different checks for whether it's a `callPackage`.
        match (is_semantic_call_package, optional_syntactic_call_package) {
            // Something like `<attr> = { }`
            (false, None)
            // Something like `<attr> = pythonPackages.callPackage ...`
            | (false, Some(_))
            // Something like `<attr> = bar` where `bar = pkgs.callPackage ...`
            | (true, None) => {
                // In all of these cases, it's not possible to migrate the package to
                // a `by-name` directory
                NonApplicable
            }

            // Something like `<attr> = pkgs.callPackage ...`
            (true, Some(syntactic_call_package)) => {
                // It's only possible to migrate such a definitions if..
                match syntactic_call_package.relative_path {
                    Some(ref rel_path) if config.by_name_dirs.iter().any(|base| rel_path.starts_with(base.path.as_str())) => {
                        // ..the path is not already within a `by-name` directory like
                        //
                        //   foo-variant = callPackage ../by-name/fo/foo/package.nix {
                        //     someFlag = true;
                        //   }
                        //
                        // While such definitions could be moved to a `by-name` directory by using
                        // `.override { someFlag = true; }` instead, this changes the semantics in
                        // relation with overlays, so migration is generally not possible.
                        //
                        // See also "package variants" in RFC 140:
                        // https://github.com/NixOS/rfcs/blob/master/rfcs/0140-simple-package-paths.md#package-variants
                        NonApplicable
                    }
                    _ => {
                        // Otherwise, the path is outside a `by-name` directory, which means it can be
                        // migrated.
                        Loose((syntactic_call_package, location.file, by_name_dir.to_owned()))
                    }
                }
            }
        }
    } else {
        // This catches all the cases not matched by the above `if let`, falling back to not being
        // able to migrate such attributes.
        NonApplicable
    };
    Ok(Success(ratchet::Package {
        // Packages being checked in this function _always_ need a manual definition, because
        // they're not using a `by-name` directory which would allow avoiding it. So instead of repeating
        // ourselves all the time to define `manual_definition`, just set it once at the end here.
        manual_definition: Tight,
        uses_by_name,
    }))
}
