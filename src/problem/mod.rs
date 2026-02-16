use derive_enum_from_into::EnumFrom;
use derive_more::Display;
use relative_path::RelativePath;

pub mod npv_100;
pub mod npv_101;
pub mod npv_102;
pub mod npv_103;
pub mod npv_104;
pub mod npv_105;
pub mod npv_106;
pub mod npv_107;
pub mod npv_108;
pub mod npv_109;
pub mod npv_110;
pub mod npv_111;

pub mod npv_120;
pub mod npv_121;
pub mod npv_122;
pub mod npv_123;
pub mod npv_124;
pub mod npv_125;
pub mod npv_126;
pub mod npv_127;
pub mod npv_128;

pub mod npv_140;
pub mod npv_141;
pub mod npv_142;
pub mod npv_143;
pub mod npv_144;

pub mod npv_160;
pub mod npv_161;
pub mod npv_162;
pub mod npv_163;

#[derive(Clone, Display, EnumFrom)]
pub enum Problem {
    /// NPV-100: attribute is not defined but it should be defined automatically
    ByNameUndefinedAttribute(npv_100::ByNameUndefinedAttribute),

    /// NPV-101: attribute is not a derivation
    ByNameNonDerivation(npv_101::ByNameNonDerivation),

    /// NPV-102: attribute uses `_internalCallByNamePackageFile`
    ByNameInternalCallPackageUsed(npv_102::ByNameInternalCallPackageUsed),

    /// NPV-103: attribute name position cannot be determined
    ByNameCannotDetermineAttributeLocation(npv_103::ByNameCannotDetermineAttributeLocation),

    /// NPV-104: non-syntactic override of by-name package
    ByNameOverrideOfNonSyntacticCallPackage(npv_104::ByNameOverrideOfNonSyntacticCallPackage),

    /// NPV-105: by-name override of ill-defined callPackage
    ByNameOverrideOfNonTopLevelPackage(npv_105::ByNameOverrideOfNonTopLevelPackage),

    /// NPV-106: by-name override contains wrong callPackage path
    ByNameOverrideContainsWrongCallPackagePath(npv_106::ByNameOverrideContainsWrongCallPackagePath),

    /// NPV-107: by-name override contains empty argument
    ByNameOverrideContainsEmptyArgument(npv_107::ByNameOverrideContainsEmptyArgument),

    /// NPV-108: by-name override contains empty path
    ByNameOverrideContainsEmptyPath(npv_108::ByNameOverrideContainsEmptyPath),

    /// NPV-109: by-name shard is not a directory
    ByNameShardIsNotDirectory(npv_109::ByNameShardIsNotDirectory),

    /// NPV-110: by-name shard is invalid
    ByNameShardIsInvalid(npv_110::ByNameShardIsInvalid),

    /// NPV-111: by-name shard is case-sensitive duplicate
    ByNameShardIsCaseSensitiveDuplicate(npv_111::ByNameShardIsCaseSensitiveDuplicate),

    /// NPV-120: Nix evaluation failed
    NixEvalError(npv_120::NixEvalError),

    /// NPV-121: Nix file contains interpolated path
    NixFileContainsPathInterpolation(npv_121::NixFileContainsPathInterpolation),

    /// NPV-122: Nix file contains search path
    NixFileContainsSearchPath(npv_122::NixFileContainsSearchPath),

    /// NPV-123: Nix file contains path expression outside of directory
    NixFileContainsPathOutsideDirectory(npv_123::NixFileContainsPathOutsideDirectory),

    /// NPV-124: Nix file contains unresolvable path expression
    NixFileContainsUnresolvablePath(npv_124::NixFileContainsUnresolvablePath),

    /// NPV-125: Package contains symlink pointing outside its directory
    PackageContainsSymlinkPointingOutside(npv_125::PackageContainsSymlinkPointingOutside),

    /// NPV-126: Package contains unresolvable symlink
    PackageContainsUnresolvableSymlink(npv_126::PackageContainsUnresolvableSymlink),

    /// NPV-127: Nix file contains absolute path expression
    NixFileContainsAbsolutePath(npv_127::NixFileContainsAbsolutePath),

    /// NPV-128: Nix file contains home-relative path expression
    NixFileContainsHomeRelativePath(npv_128::NixFileContainsHomeRelativePath),

    /// NPV-140: Package directory is not directory
    PackageDirectoryIsNotDirectory(npv_140::PackageDirectoryIsNotDirectory),

    /// NPV-141: Package name is not valid
    InvalidPackageDirectoryName(npv_141::InvalidPackageDirectoryName),

    /// NPV-142: Package is in the wrong by-name shard
    PackageInWrongShard(npv_142::PackageInWrongShard),

    /// NPV-143: `package.nix` is missing
    PackageNixMissing(npv_143::PackageNixMissing),

    /// NPV-144: `package.nix` is not a file
    PackageNixIsNotFile(npv_144::PackageNixIsNotFile),

    /// NPV-160: top-level package moved out of by-name
    TopLevelPackageMovedOutOfByName(npv_160::TopLevelPackageMovedOutOfByName),

    /// NPV-161: top-level package moved out of by-name with custom arguments
    TopLevelPackageMovedOutOfByNameWithCustomArguments(
        npv_161::TopLevelPackageMovedOutOfByNameWithCustomArguments,
    ),

    /// NPV-162: new top-level package should be in by-name
    NewTopLevelPackageShouldBeByName(npv_162::NewTopLevelPackageShouldBeByName),

    /// NPV-163: new top-level package should be in by-name with a custom argument
    NewTopLevelPackageShouldBeByNameWithCustomArgument(
        npv_163::NewTopLevelPackageShouldBeByNameWithCustomArgument,
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
