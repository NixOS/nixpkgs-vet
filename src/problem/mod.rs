use std::ffi::OsString;
use std::fmt;

use derive_enum_from_into::EnumFrom;
use indoc::writedoc;
use relative_path::RelativePath;
use relative_path::RelativePathBuf;

use crate::structure;
use crate::utils::PACKAGE_NIX_FILENAME;

mod npv_100_by_name_undefined_attribute;

pub use npv_100_by_name_undefined_attribute::ByNameUndefinedAttribute;

/// Any problem that can occur when checking Nixpkgs
/// All paths are relative to Nixpkgs such that the error messages can't be influenced by Nixpkgs absolute
/// location
#[derive(Clone, EnumFrom)]
pub enum Problem {
    /// NPV-100: attribute is not defined but it should be defined automatically
    ByNameUndefinedAttribute(ByNameUndefinedAttribute),

    // By the end of this PR, all these will be gone.
    Shard(ShardError),
    Package(PackageError),
    ByName(ByNameError),
    ByNameOverride(ByNameOverrideError),
    Path(PathError),
    NixFile(NixFileError),
    TopLevelPackage(TopLevelPackageError),
    NixEval(NixEvalError),
}

/// A file structure error involving a shard (e.g. `fo` is the shard in the path `pkgs/by-name/fo/foo/package.nix`)
#[derive(Clone)]
pub struct ShardError {
    pub shard_name: String,
    pub kind: ShardErrorKind,
}

#[derive(Clone)]
pub enum ShardErrorKind {
    ShardNonDir,
    InvalidShardName,
    CaseSensitiveDuplicate { first: OsString, second: OsString },
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

/// An error related to checks involving by-name attributes. For example, attribute `foo` in
/// `pkgs/by-name/fo/foo/package.nix`.
#[derive(Clone)]
pub struct ByNameError {
    pub attribute_name: String,
    pub kind: ByNameErrorKind,
}

#[derive(Clone)]
pub enum ByNameErrorKind {
    UndefinedAttr,
    NonDerivation,
    InternalCallPackageUsed,
    CannotDetermineAttributeLocation,
}

/// An error related to packages in `pkgs/by-name` that are manually overridden, e.g. in
/// all-packages.nix
#[derive(Clone)]
pub struct ByNameOverrideError {
    pub package_name: String,
    pub expected_package_path: RelativePathBuf,
    pub file: RelativePathBuf,
    pub line: usize,
    pub column: usize,
    pub definition: String,
    pub kind: ByNameOverrideErrorKind,
}

#[derive(Clone)]
pub enum ByNameOverrideErrorKind {
    NonSyntacticCallPackage,
    NonToplevelCallPackage,
    WrongCallPackagePath { actual_path: RelativePathBuf },
    EmptyArgument,
    NonPath,
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

/// A Nix evaluation error for some package in `pkgs/by-name`
#[derive(Clone)]
pub struct NixEvalError {
    pub stderr: String,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ByNameUndefinedAttribute(inner) => fmt::Display::fmt(inner, f),

            // By the end of this PR, all these cases will vanish.
            Problem::Shard(ShardError {
                shard_name,
                kind,
            }) => {
                let relative_shard_path = structure::relative_dir_for_shard(shard_name);
                match kind {
                    ShardErrorKind::ShardNonDir =>
                        write!(
                            f,
                            "- {relative_shard_path}: This is a file, but it should be a directory.",
                        ),
                    ShardErrorKind::InvalidShardName =>
                        write!(
                            f,
                            "- {relative_shard_path}: Invalid directory name \"{shard_name}\", must be at most 2 ASCII characters consisting of a-z, 0-9, \"-\" or \"_\".",
                        ),
                    ShardErrorKind::CaseSensitiveDuplicate { first, second } =>
                        write!(
                            f,
                            "- {relative_shard_path}: Duplicate case-sensitive package directories {first:?} and {second:?}.",
                        ),
                }
            }
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
            Problem::ByName(ByNameError {
                attribute_name,
                kind,
            }) => {
                match kind {
                    ByNameErrorKind::UndefinedAttr => {
                        let relative_package_file = structure::relative_file_for_package(attribute_name);
                        write!(
                            f,
                            "- pkgs.{attribute_name}: This attribute is not defined but it should be defined automatically as {relative_package_file}",
                        )
                    }
                    ByNameErrorKind::NonDerivation => {
                        let relative_package_file = structure::relative_file_for_package(attribute_name);
                        write!(
                            f,
                            "- pkgs.{attribute_name}: This attribute defined by {relative_package_file} is not a derivation",
                        )
                    }
                    ByNameErrorKind::InternalCallPackageUsed =>
                        write!(
                            f,
                            "- pkgs.{attribute_name}: This attribute is defined using `_internalCallByNamePackageFile`, which is an internal function not intended for manual use.",
                        ),
                    ByNameErrorKind::CannotDetermineAttributeLocation =>
                        write!(
                            f,
                            "- pkgs.{attribute_name}: Cannot determine the location of this attribute using `builtins.unsafeGetAttrPos`.",
                        ),
                }
            }
            Problem::ByNameOverride(ByNameOverrideError {
                package_name,
                expected_package_path,
                file,
                line,
                column,
                definition,
                kind,
            }) => {
                let relative_package_dir = structure::relative_dir_for_package(package_name);
                let expected_path_expr = create_path_expr(file, expected_package_path);
                let indented_definition = indent_definition(*column, definition.clone());

                match kind {
                    ByNameOverrideErrorKind::NonSyntacticCallPackage => {

                        writedoc!(
                            f,
                            "
                            - Because {relative_package_dir} exists, the attribute `pkgs.{package_name}` must be defined like

                                {package_name} = callPackage {expected_path_expr} {{ /* ... */ }};

                              However, in this PR, it isn't defined that way. See the definition in {file}:{line}

                            {indented_definition}
                            ",
                        )
                    }
                    ByNameOverrideErrorKind::NonToplevelCallPackage =>
                        writedoc!(
                            f,
                            "
                            - Because {relative_package_dir} exists, the attribute `pkgs.{package_name}` must be defined like

                                {package_name} = callPackage {expected_path_expr} {{ /* ... */ }};

                              However, in this PR, a different `callPackage` is used. See the definition in {file}:{line}:

                            {indented_definition}
                            ",
                        ),
                    ByNameOverrideErrorKind::WrongCallPackagePath { actual_path } => {
                        let actual_path_expr = create_path_expr(file, actual_path);
                        writedoc!(
                            f,
                            "
                            - Because {relative_package_dir} exists, the attribute `pkgs.{package_name}` must be defined like

                                {package_name} = callPackage {expected_path_expr} {{ /* ... */ }};

                              However, in this PR, the first `callPackage` argument is the wrong path. See the definition in {file}:{line}:

                                {package_name} = callPackage {actual_path_expr} {{ /* ... */ }};
                            ",
                        )
                    }
                    ByNameOverrideErrorKind::EmptyArgument =>
                        writedoc!(
                            f,
                            "
                            - Because {relative_package_dir} exists, the attribute `pkgs.{package_name}` must be defined like

                                {package_name} = callPackage {expected_path_expr} {{ /* ... */ }};

                              However, in this PR, the second argument is empty. See the definition in {file}:{line}:

                            {indented_definition}

                              Such a definition is provided automatically and therefore not necessary. Please remove it.
                            ",
                        ),
                    ByNameOverrideErrorKind::NonPath =>
                        writedoc!(
                            f,
                            "
                            - Because {relative_package_dir} exists, the attribute `pkgs.{package_name}` must be defined like

                                {package_name} = callPackage {expected_path_expr} {{ /* ... */ }};

                              However, in this PR, the first `callPackage` argument is not a path. See the definition in {file}:{line}:

                            {indented_definition}
                            ",
                        ),
                }
            },
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
            Problem::NixEval(NixEvalError { stderr }) => {
                f.write_str(stderr)?;
                write!(f, "- Nix evaluation failed for some package in `pkgs/by-name`, see error above")
            },
       }
    }
}

fn indent_definition(column: usize, definition: String) -> String {
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
