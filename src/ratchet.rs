//! This module implements the ratchet checks, see ../README.md#ratchet-checks
//!
//! Each type has a `compare` method that validates the ratchet checks for that item.

use relative_path::RelativePath;
use std::collections::BTreeMap;

use relative_path::RelativePathBuf;

use crate::nix_file::CallPackageArgumentInfo;
use crate::problem::{Problem, npv_160, npv_161, npv_162, npv_163};
use crate::structure::Config;
use crate::validation::{self, Validation, Validation::Success};

/// The ratchet value for the entirety of Nixpkgs.
#[derive(Default, Debug)]
pub struct Nixpkgs {
    /// The ratchet values for all packages
    pub packages: BTreeMap<String, Package>,
    pub files: BTreeMap<RelativePathBuf, File>,
}

impl Nixpkgs {
    /// Validates the ratchet checks for Nixpkgs
    pub fn compare(from: &Self, to: &Self) -> Validation<()> {
        // let mut comparison: Vec<Validation<'a, ()>> = Vec::new();
        // comparison.reserve(to.packages.len());
        // for (name, pkg) in to.packages.iter() {
        //     comparison.push(Package::compare(name, from.packages.get(name), &pkg))
        // }
        validation::sequence_(
            // We only loop over the current attributes,
            // we don't need to check ones that were removed
            // comparison
            to.packages
                .iter()
                .map(|(name, pkg)| {
                    Package::compare(name, from.packages.get(name), pkg)
                }),
        )
        .and_(validation::sequence_(to.files.iter().map(
            |(name, file)| File::compare(name, from.files.get(name), file),
        )))
    }
}

/// The ratchet value for a top-level package
#[derive(Debug, Clone)]
pub struct Package {
    /// The ratchet value for the check for non-auto-called empty arguments
    pub manual_definition: RatchetState<ManualDefinition>,

    /// The ratchet value for the check for new packages using pkgs/by-name
    pub uses_by_name: RatchetState<UsesByName>,
}

impl Package {
    /// Validates the ratchet checks for a top-level package
    pub fn compare(name: &str, optional_from: Option<&Self>, to: &Self) -> Validation<()> {
        validation::sequence_([
            RatchetState::<ManualDefinition>::compare(
                name,
                optional_from.map(|x| &x.manual_definition),
                &to.manual_definition,
            ),
            RatchetState::<UsesByName>::compare(
                name,
                optional_from.map(|x| &x.uses_by_name),
                &to.uses_by_name,
            ),
        ])
    }
}

#[derive(Debug)]
pub struct File {}

impl File {
    /// Validates the ratchet checks for a top-level package
    pub fn compare(
        _name: &RelativePath,
        _optional_from: Option<&Self>,
        _to: &Self,
    ) -> Validation<()> {
        Success(())
    }
}

/// The ratchet state of a generic ratchet check.
#[derive(Debug, Clone)]
pub enum RatchetState<Ratchet: ToProblem> {
    /// The ratchet is loose. It can be tightened more. In other words, this is the legacy state
    /// we're trying to move away from.
    ///
    /// Introducing new instances is not allowed but previous instances will continue to be
    /// allowed. The `Context` is context for error messages in case a new instance of this state
    /// is introduced.
    Loose(Ratchet::ToContext),

    /// The ratchet is tight. It can't be tightened any further. This is either because we already
    /// use the latest state, or because the ratchet isn't relevant.
    Tight,

    /// This ratchet can't be applied. State transitions from/to `NonApplicable` are always allowed.
    NonApplicable,
}

/// A trait that can convert an attribute-specific error context into a Problem.
pub trait ToProblem {
    /// Context relating to the Nixpkgs that is being transitioned _to_.
    type ToContext;

    /// How to convert an attribute-specific error context into a Problem.
    fn to_problem(name: &str, optional_from: Option<()>, to: &Self::ToContext) -> Problem;
}

impl<'a, Context: ToProblem> RatchetState<Context> {
    /// Compare the previous ratchet state of an attribute to the new state.
    /// The previous state may be `None` in case the attribute is new.
    fn compare(name: &str, optional_from: Option<&Self>, to: &'a Self) -> Validation<()> {
        match (optional_from, to) {
            // Loosening a ratchet is not allowed.
            (Some(RatchetState::Tight), RatchetState::Loose(loose_context)) => {
                Context::to_problem(name, Some(()), loose_context).into()
            }

            // Introducing a loose ratchet is also not allowed.
            (None, RatchetState::Loose(loose_context)) => {
                Context::to_problem(name, None, loose_context).into()
            }

            // Everything else is allowed, including:
            // - Loose -> Loose (grandfathering policy for a loose ratchet)
            // - -> Tight (always okay to keep or make the ratchet tight)
            // - Anything involving NotApplicable, where we can't really make any good calls
            _ => Success(()),
        }
    }
}

/// The ratchet to check whether a top-level attribute has/needs a manual definition, e.g. in
/// `pkgs/top-level/all-packages.nix`.
///
/// This ratchet is only tight for attributes that:
///
/// - Are not defined in `pkgs/by-name`, and rely on a manual definition.
///
/// - Are defined in `pkgs/by-name` without any manual definition (no custom argument overrides).
///
/// - Are defined with `pkgs/by-name` with a manual definition that can't be removed
///   because it provides custom argument overrides.
///
/// In comparison, this ratchet is loose for attributes that:
///
/// - Are defined in `pkgs/by-name` with a manual definition that doesn't have any
///   custom argument overrides.
#[derive(Debug, Clone)]
pub enum ManualDefinition {}

impl ToProblem for ManualDefinition {
    type ToContext = Problem;

    fn to_problem(_name: &str, _optional_from: Option<()>, to: &Self::ToContext) -> Problem {
        (*to).clone()
    }
}

/// The ratchet value of an attribute for the check that new packages use `pkgs/by-name`.
///
/// This checks that all new package defined using `callPackage` must be defined via
/// `pkgs/by-name`. It also checks that once a package uses `pkgs/by-name`, it can't switch back
/// to `pkgs/top-level/all-packages.nix`.
#[derive(Debug, Clone)]
pub enum UsesByName {}

impl ToProblem for UsesByName {
    type ToContext = (CallPackageArgumentInfo, RelativePathBuf, Config);

    fn to_problem(
        name: &str,
        optional_from: Option<()>,
        (to, file, config): &Self::ToContext,
    ) -> Problem {
        let is_new = optional_from.is_none();
        let is_empty = to.empty_arg;
        match (is_new, is_empty) {
            (false, true) => npv_160::TopLevelPackageMovedOutOfByName::new(
                name,
                to.relative_path.clone(),
                file,
                config.clone(),
            )
            .into(),
            // This can happen if users mistakenly assume that `pkgs/by-name` can't be used
            // for custom arguments.
            (false, false) => npv_161::TopLevelPackageMovedOutOfByNameWithCustomArguments::new(
                name,
                to.relative_path.clone(),
                file,
                config.clone(),
            )
            .into(),
            (true, true) => npv_162::NewTopLevelPackageShouldBeByName::new(
                name,
                to.relative_path.clone(),
                file,
                config.clone(),
            )
            .into(),
            (true, false) => npv_163::NewTopLevelPackageShouldBeByNameWithCustomArgument::new(
                name,
                to.relative_path.clone(),
                file,
                config.clone(),
            )
            .into(),
        }
    }
}
