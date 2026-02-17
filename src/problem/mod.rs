use derive_enum_from_into::EnumFrom;
use derive_more::Display;

pub mod npv_100;
pub mod npv_101;
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
pub mod npv_162;

#[derive(Clone, Display, EnumFrom)]
pub enum Problem {
    /// NPV-100: attribute is not defined but it should be defined automatically
    ByNameUndefinedAttribute(npv_100::ByNameUndefinedAttribute),

    /// NPV-101: attribute is not a derivation
    ByNameNonDerivation(npv_101::ByNameNonDerivation),

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

    /// NPV-162: new top-level package should be in by-name
    NewTopLevelPackageShouldBeByName(npv_162::NewTopLevelPackageShouldBeByName),
}
