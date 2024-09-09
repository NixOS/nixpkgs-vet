use std::fmt;

use derive_enum_from_into::EnumFrom;
use indoc::writedoc;
use relative_path::RelativePath;
use relative_path::RelativePathBuf;

use crate::structure::{self, PACKAGE_NIX_FILENAME};

mod npv_100_by_name_undefined_attribute;
mod npv_101_by_name_non_derivation;
mod npv_102_by_name_internal_call_package_used;
mod npv_103_by_name_cannot_determine_attribute_location;
mod npv_104_by_name_override_of_non_syntactic_call_package;
mod npv_105_by_name_override_of_non_top_level_package;
mod npv_106_by_name_override_contains_wrong_call_package_path;
mod npv_107_by_name_override_contains_empty_argument;
mod npv_108_by_name_override_contains_empty_path;
mod npv_109_by_name_shard_is_not_directory;
mod npv_110_by_name_shard_is_invalid;
mod npv_111_by_name_shard_is_case_sensitive_duplicate;
mod npv_120_nix_eval_error;
mod npv_121_nix_file_path_interpolation_unsupported;
mod npv_122_nix_file_search_path_expression_unsupported;
mod npv_123_nix_file_path_outside_of_directory;

pub use npv_100_by_name_undefined_attribute::ByNameUndefinedAttribute;
pub use npv_101_by_name_non_derivation::ByNameNonDerivation;
pub use npv_102_by_name_internal_call_package_used::ByNameInternalCallPackageUsed;
pub use npv_103_by_name_cannot_determine_attribute_location::ByNameCannotDetermineAttributeLocation;
pub use npv_104_by_name_override_of_non_syntactic_call_package::ByNameOverrideOfNonSyntacticCallPackage;
pub use npv_105_by_name_override_of_non_top_level_package::ByNameOverrideOfNonTopLevelPackage;
pub use npv_106_by_name_override_contains_wrong_call_package_path::ByNameOverrideContainsWrongCallPackagePath;
pub use npv_107_by_name_override_contains_empty_argument::ByNameOverrideContainsEmptyArgument;
pub use npv_108_by_name_override_contains_empty_path::ByNameOverrideContainsEmptyPath;
pub use npv_109_by_name_shard_is_not_directory::ByNameShardIsNotDirectory;
pub use npv_110_by_name_shard_is_invalid::ByNameShardIsInvalid;
pub use npv_111_by_name_shard_is_case_sensitive_duplicate::ByNameShardIsCaseSensitiveDuplicate;
pub use npv_120_nix_eval_error::NixEvalError;
pub use npv_121_nix_file_path_interpolation_unsupported::NixFileContainsPathInterpolation;
pub use npv_122_nix_file_search_path_expression_unsupported::NixFileContainsSearchPath;
pub use npv_123_nix_file_path_outside_of_directory::NixFileContainsPathOutsideDirectory;

/// Any problem that can occur when checking Nixpkgs
/// All paths are relative to Nixpkgs such that the error messages can't be influenced by Nixpkgs absolute
/// location
#[derive(Clone, EnumFrom)]
pub enum Problem {
    /// NPV-100: attribute is not defined but it should be defined automatically
    ByNameUndefinedAttribute(ByNameUndefinedAttribute),

    /// NPV-101: attribute is not a derivation
    ByNameNonDerivation(ByNameNonDerivation),

    /// NPV-102: attribute uses `_internalCallByNamePackageFile`
    ByNameInternalCallPackageUsed(ByNameInternalCallPackageUsed),

    /// NPV-103: attribute name position cannot be determined
    ByNameCannotDetermineAttributeLocation(ByNameCannotDetermineAttributeLocation),

    /// NPV-104: non-syntactic override of by-name package
    ByNameOverrideOfNonSyntacticCallPackage(ByNameOverrideOfNonSyntacticCallPackage),

    /// NPV-105: by-name override of ill-defined callPackage
    ByNameOverrideOfNonTopLevelPackage(ByNameOverrideOfNonTopLevelPackage),

    /// NPV-106: by-name override contains wrong callPackage path
    ByNameOverrideContainsWrongCallPackagePath(ByNameOverrideContainsWrongCallPackagePath),

    /// NPV-107: by-name override contains empty argument
    ByNameOverrideContainsEmptyArgument(ByNameOverrideContainsEmptyArgument),

    /// NPV-108: by-name override contains empty path
    ByNameOverrideContainsEmptyPath(ByNameOverrideContainsEmptyPath),

    /// NPV-109: by-name shard is not a directory
    ByNameShardIsNotDirectory(ByNameShardIsNotDirectory),

    /// NPV-110: by-name shard is invalid
    ByNameShardIsInvalid(ByNameShardIsInvalid),

    /// NPV-111: by-name shard is case-sensitive duplicate
    ByNameShardIsCaseSensitiveDuplicate(ByNameShardIsCaseSensitiveDuplicate),

    /// NPV-120: Nix evaluation failed
    NixEvalError(NixEvalError),

    /// NPV-121: Nix file contains interpolated path
    NixFileContainsPathInterpolation(NixFileContainsPathInterpolation),

    /// NPV-122: Nix file contains search path
    NixFileContainsSearchPath(NixFileContainsSearchPath),

    /// NPV-123: Nix file contains path expression outside of directory
    NixFileContainsPathOutsideDirectory(NixFileContainsPathOutsideDirectory),

    // By the end of this PR, all these will be gone.
    Package(PackageError),
    Path(PathError),
    NixFile(NixFileError),
    TopLevelPackage(TopLevelPackageError),
}

/// A file structure error involving the package name and/or path.
#[derive(Clone)]
pub struct PackageError {
    pub relative_package_dir: RelativePathBuf,
    pub kind: PackageErrorKind,
}

#[derive(Clone)]
pub enum PackageErrorKind {
    PackageNonDir {
        package_name: String,
    },
    InvalidPackageName {
        invalid_package_name: String,
    },
    IncorrectShard {
        correct_relative_package_dir: RelativePathBuf,
    },
    PackageNixNonExistent,
    PackageNixDir,
}

/// An error that results from checks that verify a specific path does not reference outside the
/// package directory.
#[derive(Clone)]
pub struct PathError {
    pub relative_package_dir: RelativePathBuf,
    pub subpath: RelativePathBuf,
    pub kind: PathErrorKind,
}

#[derive(Clone)]
pub enum PathErrorKind {
    OutsideSymlink,
    UnresolvableSymlink { io_error: String },
}

/// An error that results from checks that verify a nix file that contains a path expression does
/// not reference outside the package.
#[derive(Clone)]
pub struct NixFileError {
    pub relative_package_dir: RelativePathBuf,
    pub subpath: RelativePathBuf,
    pub line: usize,
    pub text: String,
    pub kind: NixFileErrorKind,
}

#[derive(Clone)]
pub enum NixFileErrorKind {
    PathInterpolation,
    SearchPath,
    OutsidePathReference,
    UnresolvablePathReference { io_error: String },
}

/// An error related to the introduction/move of a top-level package not using `pkgs/by-name`, but
/// it should.
#[derive(Clone)]
pub struct TopLevelPackageError {
    pub package_name: String,
    pub call_package_path: Option<RelativePathBuf>,
    pub file: RelativePathBuf,
    pub is_new: bool,
    pub is_empty: bool,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ByNameUndefinedAttribute(inner) => fmt::Display::fmt(inner, f),
            Self::ByNameNonDerivation(inner) => fmt::Display::fmt(inner, f),
            Self::ByNameInternalCallPackageUsed(inner) => fmt::Display::fmt(inner, f),
            Self::ByNameCannotDetermineAttributeLocation(inner) => fmt::Display::fmt(inner, f),
            Self::ByNameOverrideOfNonSyntacticCallPackage(inner) => fmt::Display::fmt(inner, f),
            Self::ByNameOverrideOfNonTopLevelPackage(inner) => fmt::Display::fmt(inner, f),
            Self::ByNameOverrideContainsWrongCallPackagePath(inner) => fmt::Display::fmt(inner, f),
            Self::ByNameOverrideContainsEmptyArgument(inner) => fmt::Display::fmt(inner, f),
            Self::ByNameOverrideContainsEmptyPath(inner) => fmt::Display::fmt(inner, f),
            Self::ByNameShardIsNotDirectory(inner) => fmt::Display::fmt(inner, f),
            Self::ByNameShardIsInvalid(inner) => fmt::Display::fmt(inner, f),
            Self::ByNameShardIsCaseSensitiveDuplicate(inner) => fmt::Display::fmt(inner, f),
            Self::NixEvalError(inner) => fmt::Display::fmt(inner, f),
            Self::NixFileContainsPathInterpolation(inner) => fmt::Display::fmt(inner, f),
            Self::NixFileContainsSearchPath(inner) => fmt::Display::fmt(inner, f),
            Self::NixFileContainsPathOutsideDirectory(inner) => fmt::Display::fmt(inner, f),

            // By the end of this PR, all these cases will vanish.
            Problem::Package(PackageError {
                relative_package_dir,
                kind,
            }) => {
                match kind {
                    PackageErrorKind::PackageNonDir { package_name } => {
                        let relative_package_dir = structure::relative_dir_for_package(package_name);
                        write!(
                            f,
                            "- {relative_package_dir}: This path is a file, but it should be a directory.",
                        )
                    }
                    PackageErrorKind::InvalidPackageName { invalid_package_name } =>
                        write!(
                            f,
                            "- {relative_package_dir}: Invalid package directory name \"{invalid_package_name}\", must be ASCII characters consisting of a-z, A-Z, 0-9, \"-\" or \"_\".",
                        ),
                    PackageErrorKind::IncorrectShard { correct_relative_package_dir } =>
                        write!(
                            f,
                            "- {relative_package_dir}: Incorrect directory location, should be {correct_relative_package_dir} instead.",
                        ),
                    PackageErrorKind::PackageNixNonExistent =>
                        write!(
                            f,
                            "- {relative_package_dir}: Missing required \"{PACKAGE_NIX_FILENAME}\" file.",
                        ),
                    PackageErrorKind::PackageNixDir =>
                        write!(
                            f,
                            "- {relative_package_dir}: \"{PACKAGE_NIX_FILENAME}\" must be a file.",
                        ),
                }
            }
            Problem::Path(PathError {
                relative_package_dir,
                subpath,
                kind,
            }) => {
                match kind {
                    PathErrorKind::OutsideSymlink =>
                        write!(
                            f,
                            "- {relative_package_dir}: Path {subpath} is a symlink pointing to a path outside the directory of that package.",
                        ),
                    PathErrorKind::UnresolvableSymlink { io_error } =>
                        write!(
                            f,
                            "- {relative_package_dir}: Path {subpath} is a symlink which cannot be resolved: {io_error}.",
                        ),
                }
            },
            Problem::NixFile(NixFileError {
                relative_package_dir,
                subpath,
                line,
                text,
                kind
            }) => {
                match kind {
                    NixFileErrorKind::PathInterpolation =>
                        write!(
                            f,
                            "- {relative_package_dir}: File {subpath} at line {line} contains the path expression \"{text}\", which is not yet supported and may point outside the directory of that package.",
                        ),
                    NixFileErrorKind::SearchPath =>
                        write!(
                            f,
                            "- {relative_package_dir}: File {subpath} at line {line} contains the nix search path expression \"{text}\" which may point outside the directory of that package.",
                        ),
                    NixFileErrorKind::OutsidePathReference =>
                        writedoc!(
                            f,
                            "
                            - {relative_package_dir}: File {subpath} at line {line} contains the path expression \"{text}\" which may point outside the directory of that package.
                              This is undesirable because it creates dependencies between internal paths, making it harder to reorganise Nixpkgs in the future.
                              Alternatives include:
                              - If you are creating a new version of a package with a common file between versions, consider following the recommendation in https://github.com/NixOS/nixpkgs/tree/master/pkgs/by-name#recommendation-for-new-packages-with-multiple-versions.
                              - If the path being referenced could be considered a stable interface with multiple uses, consider exposing it via a `pkgs` attribute, then taking it as a attribute argument in {PACKAGE_NIX_FILENAME}.
                              - If the path being referenced is internal and has multiple uses, consider passing the file as an explicit `callPackage` argument in `pkgs/top-level/all-packages.nix`.
                              - If the path being referenced is internal and will need to be modified independently of the original, consider copying it into the {relative_package_dir} directory.
                            "
                        ),
                    NixFileErrorKind::UnresolvablePathReference { io_error } =>
                        write!(
                            f,
                            "- {relative_package_dir}: File {subpath} at line {line} contains the path expression \"{text}\" which cannot be resolved: {io_error}.",
                        ),
                }
            },
            Problem::TopLevelPackage(TopLevelPackageError {
                package_name,
                call_package_path,
                file,
                is_new,
                is_empty,
            }) => {
                let call_package_arg =
                    if let Some(path) = &call_package_path {
                        format!("./{}", path)
                    } else {
                        "...".into()
                    };
                let relative_package_file = structure::relative_file_for_package(package_name);

                match (is_new, is_empty) {
                    (false, true) =>
                        writedoc!(
                            f,
                            "
                            - Attribute `pkgs.{package_name}` was previously defined in {relative_package_file}, but is now manually defined as `callPackage {call_package_arg} {{ /* ... */ }}` in {file}.
                              Please move the package back and remove the manual `callPackage`.
                            ",
                        ),
                    (false, false) =>
                        // This can happen if users mistakenly assume that for custom arguments,
                        // pkgs/by-name can't be used.
                        writedoc!(
                            f,
                            "
                            - Attribute `pkgs.{package_name}` was previously defined in {relative_package_file}, but is now manually defined as `callPackage {call_package_arg} {{ ... }}` in {file}.
                              While the manual `callPackage` is still needed, it's not necessary to move the package files.
                            ",
                        ),
                    (true, true) =>
                        writedoc!(
                            f,
                            "
                            - Attribute `pkgs.{package_name}` is a new top-level package using `pkgs.callPackage {call_package_arg} {{ /* ... */ }}`.
                              Please define it in {relative_package_file} instead.
                              See `pkgs/by-name/README.md` for more details.
                              Since the second `callPackage` argument is `{{ }}`, no manual `callPackage` in {file} is needed anymore.
                            ",
                        ),
                    (true, false) =>
                        writedoc!(
                            f,
                            "
                            - Attribute `pkgs.{package_name}` is a new top-level package using `pkgs.callPackage {call_package_arg} {{ /* ... */ }}`.
                              Please define it in {relative_package_file} instead.
                              See `pkgs/by-name/README.md` for more details.
                              Since the second `callPackage` argument is not `{{ }}`, the manual `callPackage` in {file} is still needed.
                            ",
                        ),
                }
            },
       }
    }
}

fn indent_definition(column: usize, definition: &str) -> String {
    // The entire code should be indented 4 spaces
    textwrap::indent(
        // But first we want to strip the code's natural indentation
        &textwrap::dedent(
            // The definition _doesn't_ include the leading spaces, but we can
            // recover those from the column
            &format!("{}{definition}", " ".repeat(column - 1)),
        ),
        "    ",
    )
}

/// Creates a Nix path expression that when put into Nix file `from_file`, would point to the `to_file`.
fn create_path_expr(
    from_file: impl AsRef<RelativePath>,
    to_file: impl AsRef<RelativePath>,
) -> String {
    // This `expect` calls should never trigger because we only call this function with files.
    // That's why we `expect` them!
    let from = from_file.as_ref().parent().expect("a parent for this path");
    let rel = from.relative(to_file);
    format!("./{rel}")
}
