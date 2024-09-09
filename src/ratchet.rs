//! This module implements the ratchet checks, see ../README.md#ratchet-checks
//!
//! Each type has a `compare` method that validates the ratchet checks for that item.

use std::collections::HashMap;

use relative_path::RelativePathBuf;

use crate::nix_file::CallPackageArgumentInfo;
use crate::problem::{Problem, TopLevelPackageError, TopLevelPackageMovedOutOfByName};
use crate::validation::{self, Validation, Validation::Success};

/// The ratchet value for the entirety of Nixpkgs.
#[derive(Default)]
pub struct Nixpkgs {
    /// Sorted list of packages in package_map
    pub package_names: Vec<String>,
    /// The ratchet values for all packages
    pub package_map: HashMap<String, Package>,
}

impl Nixpkgs {
    /// Validates the ratchet checks for Nixpkgs
    pub fn compare(from: Self, to: Self) -> Validation<()> {
        validation::sequence_(
            // We only loop over the current attributes,
            // we don't need to check ones that were removed
            to.package_names.into_iter().map(|name| {
                Package::compare(&name, from.package_map.get(&name), &to.package_map[&name])
            }),
        )
    }
}

/// The ratchet value for a top-level package
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

/// The ratchet state of a generic ratchet check.
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

    /// This ratchet can't be applied. State transitions from/to NonApplicable are always allowed.
    NonApplicable,
}

/// A trait that can convert an attribute-specific error context into a Problem.
pub trait ToProblem {
    /// Context relating to the Nixpkgs that is being transitioned _to_.
    type ToContext;

    /// How to convert an attribute-specific error context into a Problem.
    fn to_problem(name: &str, optional_from: Option<()>, to: &Self::ToContext) -> Problem;
}

impl<Context: ToProblem> RatchetState<Context> {
    /// Compare the previous ratchet state of an attribute to the new state.
    /// The previous state may be `None` in case the attribute is new.
    fn compare(name: &str, optional_from: Option<&Self>, to: &Self) -> Validation<()> {
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
pub enum UsesByName {}

impl ToProblem for UsesByName {
    type ToContext = (CallPackageArgumentInfo, RelativePathBuf);

    fn to_problem(name: &str, optional_from: Option<()>, (to, file): &Self::ToContext) -> Problem {
        let is_new = optional_from.is_none();
        let is_empty = to.empty_arg;
        match (is_new, is_empty) {
            (false, true) => {
                TopLevelPackageMovedOutOfByName::new(name, to.relative_path.clone(), file).into()
            }
            _ => Problem::TopLevelPackage(TopLevelPackageError {
                package_name: name.to_owned(),
                call_package_path: to.relative_path.clone(),
                file: file.to_owned(),
                is_new: optional_from.is_none(),
                is_empty: to.empty_arg,
            }),
        }
    }
}
