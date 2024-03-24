use crate::structure;
use crate::utils::PACKAGE_NIX_FILENAME;
use indoc::writedoc;
use relative_path::RelativePath;
use relative_path::RelativePathBuf;
use std::ffi::OsString;
use std::fmt;

/// Any problem that can occur when checking Nixpkgs
/// All paths are relative to Nixpkgs such that the error messages can't be influenced by Nixpkgs absolute
/// location
#[derive(Clone)]
pub enum NixpkgsProblem {
    ShardProblem(ShardError),
    PackageNonDir {
        package_name: String,
    },
    InvalidPackageName {
        relative_package_dir: RelativePathBuf,
        invalid_package_name: String,
    },
    IncorrectShard {
        relative_package_dir: RelativePathBuf,
        correct_relative_package_dir: RelativePathBuf,
    },
    PackageNixNonExistent {
        relative_package_dir: RelativePathBuf,
    },
    PackageNixDir {
        relative_package_dir: RelativePathBuf,
    },
    UndefinedAttr {
        package_name: String,
    },
    ByNameOverrideProblem(ByNameOverrideError),
    NonDerivation {
        package_name: String,
    },
    OutsideSymlink {
        relative_package_dir: RelativePathBuf,
        subpath: RelativePathBuf,
    },
    UnresolvableSymlink {
        relative_package_dir: RelativePathBuf,
        subpath: RelativePathBuf,
        io_error: String,
    },
    NixFileProblem(NixFileError),
    RatchetProblem(RatchetError),
    InternalCallPackageUsed {
        attr_name: String,
    },
    CannotDetermineAttributeLocation {
        attr_name: String,
    },
}

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

#[derive(Clone)]
pub struct ByNameOverrideError {
    pub package_name: String,
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
    WrongCallPackagePath {
        actual_path: RelativePathBuf,
        expected_path: RelativePathBuf,
    },
    EmptyArgument,
    NonPath,
}

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

#[derive(Clone)]
pub struct RatchetError {
    pub package_name: String,
    pub call_package_path: Option<RelativePathBuf>,
    pub file: RelativePathBuf,
    pub kind: RatchetErrorKind,
}

#[derive(Clone)]
pub enum RatchetErrorKind {
    MovedOutOfByName,
    MovedOutOfByNameEmptyArg,
    NewPackageNotUsingByName,
    NewPackageNotUsingByNameEmptyArg,
}

impl fmt::Display for NixpkgsProblem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NixpkgsProblem::ShardProblem(ShardError {
                shard_name,
                kind,
            }) => {
                let relative_shard_path = structure::relative_dir_for_shard(shard_name);
                match kind {
                    ShardErrorKind::ShardNonDir =>
                        write!(
                            f,
                            "{relative_shard_path}: This is a file, but it should be a directory.",
                        ),
                    ShardErrorKind::InvalidShardName =>
                        write!(
                            f,
                            "{relative_shard_path}: Invalid directory name \"{shard_name}\", must be at most 2 ASCII characters consisting of a-z, 0-9, \"-\" or \"_\".",
                        ),
                    ShardErrorKind::CaseSensitiveDuplicate { first, second } =>
                        write!(
                            f,
                            "{relative_shard_path}: Duplicate case-sensitive package directories {first:?} and {second:?}.",
                        ),
                }
            }
            NixpkgsProblem::PackageNonDir { package_name } => {
                let relative_package_dir = structure::relative_dir_for_package(package_name);
                write!(
                    f,
                    "{relative_package_dir}: This path is a file, but it should be a directory.",
                )
            }
            NixpkgsProblem::InvalidPackageName { relative_package_dir, invalid_package_name } =>
                write!(
                    f,
                    "{relative_package_dir}: Invalid package directory name \"{invalid_package_name}\", must be ASCII characters consisting of a-z, A-Z, 0-9, \"-\" or \"_\".",
                ),
            NixpkgsProblem::IncorrectShard { relative_package_dir, correct_relative_package_dir } =>
                write!(
                    f,
                    "{relative_package_dir}: Incorrect directory location, should be {correct_relative_package_dir} instead.",
                ),
            NixpkgsProblem::PackageNixNonExistent { relative_package_dir } =>
                write!(
                    f,
                    "{relative_package_dir}: Missing required \"{PACKAGE_NIX_FILENAME}\" file.",
                ),
            NixpkgsProblem::PackageNixDir { relative_package_dir } =>
                write!(
                    f,
                    "{relative_package_dir}: \"{PACKAGE_NIX_FILENAME}\" must be a file.",
                ),
            NixpkgsProblem::UndefinedAttr {  package_name } => {
                let relative_package_file = structure::relative_file_for_package(package_name);
                write!(
                    f,
                    "pkgs.{package_name}: This attribute is not defined but it should be defined automatically as {relative_package_file}",
                )
            }
            NixpkgsProblem::ByNameOverrideProblem(ByNameOverrideError {
                package_name,
                file,
                line,
                column,
                definition,
                kind,
            }) => {
                let relative_package_dir = structure::relative_dir_for_package(package_name);
                let relative_package_file = structure::relative_file_for_package(package_name);
                let indented_definition = indent_definition(*column, definition.clone());

                match kind {
                    ByNameOverrideErrorKind::NonSyntacticCallPackage =>
                        writedoc!(
                            f,
                            "
                            - Because {relative_package_dir} exists, the attribute `pkgs.{package_name}` must be defined like

                                {package_name} = callPackage ./{relative_package_file} {{ /* ... */ }};

                              However, in this PR, it isn't defined that way. See the definition in {file}:{line}

                            {indented_definition}
                            ",
                        ),
                    ByNameOverrideErrorKind::NonToplevelCallPackage =>
                        writedoc!(
                            f,
                            "
                            - Because {relative_package_dir} exists, the attribute `pkgs.{package_name}` must be defined like

                                {package_name} = callPackage ./{relative_package_file} {{ /* ... */ }};

                              However, in this PR, a different `callPackage` is used. See the definition in {file}:{line}:

                            {indented_definition}
                            ",
                        ),
                    ByNameOverrideErrorKind::WrongCallPackagePath { actual_path, expected_path } => {
                        let expected_path_expr = create_path_expr(file, expected_path);
                        let actual_path_expr = create_path_expr(file, actual_path);
                        writedoc! {
                            f,
                            "
                            - Because {relative_package_dir} exists, the attribute `pkgs.{package_name}` must be defined like

                                {package_name} = callPackage {expected_path_expr} {{ /* ... */ }};

                              However, in this PR, the first `callPackage` argument is the wrong path. See the definition in {file}:{line}:

                                {package_name} = callPackage {actual_path_expr} {{ /* ... */ }};
                            ",
                        }
                    }
                    ByNameOverrideErrorKind::EmptyArgument =>
                        writedoc!(
                            f,
                            "
                            - Because {relative_package_dir} exists, the attribute `pkgs.{package_name}` must be defined like

                                {package_name} = callPackage ./{relative_package_file} {{ /* ... */ }};

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

                                {package_name} = callPackage ./{relative_package_file} {{ /* ... */ }};

                              However, in this PR, the first `callPackage` argument is not a path. See the definition in {file}:{line}:

                            {indented_definition}
                            ",
                        ),
                }
            },
            NixpkgsProblem::NonDerivation { package_name } => {
                let relative_package_file = structure::relative_file_for_package(package_name);
                write!(
                    f,
                    "pkgs.{package_name}: This attribute defined by {relative_package_file} is not a derivation",
                )
            }
            NixpkgsProblem::OutsideSymlink { relative_package_dir, subpath } =>
                write!(
                    f,
                    "{relative_package_dir}: Path {subpath} is a symlink pointing to a path outside the directory of that package.",
                ),
            NixpkgsProblem::UnresolvableSymlink { relative_package_dir, subpath, io_error } =>
                write!(
                    f,
                    "{relative_package_dir}: Path {subpath} is a symlink which cannot be resolved: {io_error}.",
                ),
            NixpkgsProblem::NixFileProblem(NixFileError {
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
                            "{relative_package_dir}: File {subpath} at line {line} contains the path expression \"{text}\", which is not yet supported and may point outside the directory of that package.",
                        ),
                    NixFileErrorKind::SearchPath =>
                        write!(
                            f,
                            "{relative_package_dir}: File {subpath} at line {line} contains the nix search path expression \"{text}\" which may point outside the directory of that package.",
                        ),
                    NixFileErrorKind::OutsidePathReference =>
                        write!(
                            f,
                            "{relative_package_dir}: File {subpath} at line {line} contains the path expression \"{text}\" which may point outside the directory of that package.",
                        ),
                    NixFileErrorKind::UnresolvablePathReference { io_error } =>
                        write!(
                            f,
                            "{relative_package_dir}: File {subpath} at line {line} contains the path expression \"{text}\" which cannot be resolved: {io_error}.",
                        ),
                }
            },
            NixpkgsProblem::RatchetProblem(RatchetError {
                package_name,
                call_package_path,
                file,
                kind,
            }) => {
                let call_package_arg =
                    if let Some(path) = &call_package_path {
                        format!("./{}", path)
                    } else {
                        "...".into()
                    };
                let relative_package_file = structure::relative_file_for_package(package_name);

                match kind {
                    RatchetErrorKind::MovedOutOfByNameEmptyArg =>
                        writedoc!(
                            f,
                            "
                            - Attribute `pkgs.{package_name}` was previously defined in {relative_package_file}, but is now manually defined as `callPackage {call_package_arg} {{ /* ... */ }}` in {file}.
                              Please move the package back and remove the manual `callPackage`.
                            ",
                        ),
                    RatchetErrorKind::MovedOutOfByName =>
                        // This can happen if users mistakenly assume that for custom arguments,
                        // pkgs/by-name can't be used.
                        writedoc!(
                            f,
                            "
                            - Attribute `pkgs.{package_name}` was previously defined in {relative_package_file}, but is now manually defined as `callPackage {call_package_arg} {{ ... }}` in {file}.
                              While the manual `callPackage` is still needed, it's not necessary to move the package files.
                            ",
                        ),
                    RatchetErrorKind::NewPackageNotUsingByNameEmptyArg =>
                        writedoc!(
                            f,
                            "
                            - Attribute `pkgs.{package_name}` is a new top-level package using `pkgs.callPackage {call_package_arg} {{ /* ... */ }}`.
                              Please define it in {relative_package_file} instead.
                              See `pkgs/by-name/README.md` for more details.
                              Since the second `callPackage` argument is `{{ }}`, no manual `callPackage` in {file} is needed anymore.
                            ",
                        ),
                    RatchetErrorKind::NewPackageNotUsingByName =>
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
            NixpkgsProblem::InternalCallPackageUsed { attr_name } =>
                write!(
                    f,
                    "pkgs.{attr_name}: This attribute is defined using `_internalCallByNamePackageFile`, which is an internal function not intended for manual use.",
                ),
            NixpkgsProblem::CannotDetermineAttributeLocation { attr_name } =>
                write!(
                    f,
                    "pkgs.{attr_name}: Cannot determine the location of this attribute using `builtins.unsafeGetAttrPos`.",
                ),
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
