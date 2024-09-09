use derive_enum_from_into::EnumFrom;
use derive_more::Display;
use relative_path::RelativePath;

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
mod npv_124_nix_file_contains_unresolvable_path;
mod npv_125_package_contains_symlink_pointing_outside;
mod npv_126_package_contains_unresolvable_symlink;
mod npv_140_package_directory_is_not_directory;
mod npv_141_invalid_package_directory_name;
mod npv_142_package_in_wrong_shard;
mod npv_143_package_nix_missing;
mod npv_144_package_nix_is_not_a_file;
mod npv_160_top_level_package_moved_back_from_by_name;
mod npv_161_top_level_package_moved_with_custom_arguments;
mod npv_162_new_top_level_package_should_be_by_name;
mod npv_163_new_top_level_package_with_custom_arguments;

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
pub use npv_124_nix_file_contains_unresolvable_path::NixFileContainsUnresolvablePath;
pub use npv_125_package_contains_symlink_pointing_outside::PackageContainsSymlinkPointingOutside;
pub use npv_126_package_contains_unresolvable_symlink::PackageContainsUnresolvableSymlink;
pub use npv_140_package_directory_is_not_directory::PackageDirectoryIsNotDirectory;
pub use npv_141_invalid_package_directory_name::InvalidPackageDirectoryName;
pub use npv_142_package_in_wrong_shard::PackageInWrongShard;
pub use npv_143_package_nix_missing::PackageNixMissing;
pub use npv_144_package_nix_is_not_a_file::PackageNixIsNotFile;
pub use npv_160_top_level_package_moved_back_from_by_name::TopLevelPackageMovedOutOfByName;
pub use npv_161_top_level_package_moved_with_custom_arguments::TopLevelPackageMovedOutOfByNameWithCustomArguments;
pub use npv_162_new_top_level_package_should_be_by_name::NewTopLevelPackageShouldBeByName;
pub use npv_163_new_top_level_package_with_custom_arguments::NewTopLevelPackageShouldBeByNameWithCustomArgument;

/// Any problem that can occur when checking Nixpkgs
/// All paths are relative to Nixpkgs such that the error messages can't be influenced by Nixpkgs absolute
/// location
#[derive(Clone, Display, EnumFrom)]
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

    /// NPV-124: Nix file contains unresolvable path expression
    NixFileContainsUnresolvablePath(NixFileContainsUnresolvablePath),

    /// NPV-125: Package contains symlink pointing outside its directory
    PackageContainsSymlinkPointingOutside(PackageContainsSymlinkPointingOutside),

    /// NPV-126: Package contains unresolvable symlink
    PackageContainsUnresolvableSymlink(PackageContainsUnresolvableSymlink),

    /// NPV-140: Package directory is not directory
    PackageDirectoryIsNotDirectory(PackageDirectoryIsNotDirectory),

    /// NPV-141: Package name is not valid
    InvalidPackageDirectoryName(InvalidPackageDirectoryName),

    /// NPV-142: Package is in the wrong by-name shard
    PackageInWrongShard(PackageInWrongShard),

    /// NPV-143: `package.nix` is missing
    PackageNixMissing(PackageNixMissing),

    /// NPV-144: `package.nix` is not a file
    PackageNixIsNotFile(PackageNixIsNotFile),

    /// NPV-160: top-level package moved out of by-name
    TopLevelPackageMovedOutOfByName(TopLevelPackageMovedOutOfByName),

    /// NPV-161: top-level package moved out of by-name with custom arguments
    TopLevelPackageMovedOutOfByNameWithCustomArguments(
        TopLevelPackageMovedOutOfByNameWithCustomArguments,
    ),

    /// NPV-162: new top-level package should be in by-name
    NewTopLevelPackageShouldBeByName(NewTopLevelPackageShouldBeByName),

    /// NPV-163: new top-level package should be in by-name with a custom argument
    NewTopLevelPackageShouldBeByNameWithCustomArgument(
        NewTopLevelPackageShouldBeByNameWithCustomArgument,
    ),
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
