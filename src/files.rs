use relative_path::RelativePath;
use relative_path::RelativePathBuf;
use rnix::ast;
use rnix::ast::AstToken;
use rowan::ast::AstNode;
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::location;
use crate::nix_file::{NixFile, NixFileStore};
use crate::problem::npv_172::{ExecutableReference, ExecutableReferenceFix};
use crate::problem::{Problem, npv_145, npv_146, npv_170, npv_171};
use crate::validation::ResultIteratorExt;
use crate::validation::Validation::{Failure, Success};
use crate::validation::sequence_;
use crate::{ratchet, structure, validation};

/// Runs check on all Nix files, returning a ratchet result for each
pub fn check_files(
    nixpkgs_path: &Path,
    nix_file_store: &mut NixFileStore,
) -> validation::Result<BTreeMap<RelativePathBuf, ratchet::File>> {
    process_nix_files(nixpkgs_path, nix_file_store, |relative_path, nix_file| {
        let result = sequence_([
            check_executable_iff_shebang(relative_path, &nix_file.path)?,
            check_invalid_escapes(relative_path, nix_file)?,
            check_optional_list_parameters(relative_path, nix_file)?,
        ]);
        Ok(result.map(|()| ratchet::File {
            executable_references: collect_executable_references(relative_path, nix_file),
        }))
    })
}

/// Processes all Nix files in a Nixpkgs directory according to a given function `f`, collecting the
/// results into a mapping from each file to a ratchet value.
fn process_nix_files(
    nixpkgs_path: &Path,
    nix_file_store: &mut NixFileStore,
    f: impl Fn(&RelativePath, &NixFile) -> validation::Result<ratchet::File>,
) -> validation::Result<BTreeMap<RelativePathBuf, ratchet::File>> {
    // Get all Nix files
    let files = {
        let mut files = vec![];
        collect_nix_files(nixpkgs_path, &RelativePathBuf::new(), &mut files)?;
        files
    };

    let results = ResultIteratorExt::collect_vec(files.into_iter().map(|path| {
        // Get the (optionally-cached) parsed Nix file
        let nix_file = nix_file_store.get(&path.to_path(nixpkgs_path))?;
        let result = f(&path, nix_file)?;
        let val = result.map(|ratchet| (path, ratchet));
        Ok::<_, anyhow::Error>(val)
    }))?;

    Ok(validation::sequence(results).map(|entries| {
        // Convert the Vec to a BTreeMap
        entries.into_iter().collect()
    }))
}

/// Check that a Nix file is executable if and only if it has a shebang (`#!`) line.
fn check_executable_iff_shebang(
    relative_path: &RelativePath,
    absolute_path: &Path,
) -> validation::Result<()> {
    let metadata = fs::metadata(absolute_path)?;
    let mode = metadata.permissions().mode();
    let is_executable = mode & 0o111 != 0;

    let mut file = fs::File::open(absolute_path)?;
    let mut buf = [0u8; 2];
    let bytes_read = file.read(&mut buf)?;
    let has_shebang = bytes_read >= 2 && buf == *b"#!";

    match (is_executable, has_shebang) {
        // Executable without shebang: error
        (true, false) => Ok(npv_145::NixFileIsExecutableWithoutShebang::new(relative_path).into()),
        // Shebang without executable: error
        (false, true) => Ok(npv_146::NixFileHasShebangButNotExecutable::new(relative_path).into()),
        // Both or neither: fine
        _ => Ok(Success(())),
    }
}

/// Check that a Nix file does not contain useless escape sequences.
fn check_invalid_escapes(
    relative_path: &RelativePath,
    nix_file: &NixFile,
) -> validation::Result<()> {
    let mut problems: Vec<Problem> = Vec::new();

    let mut report = |index: usize, prefix: &str, c: char, fixed: Option<String>| {
        problems.push(
            npv_170::NixFileContainsUselessEscape::new(
                location::Location::new(
                    relative_path,
                    nix_file.line_index.line(index),
                    nix_file.line_index.column(index),
                ),
                format!("{prefix}{c}"),
                c.to_string(),
                fixed,
            )
            .into(),
        );
    };

    for str_node in nix_file
        .syntax_root
        .syntax()
        .descendants()
        .filter_map(ast::Str::cast)
    {
        let is_multiline = is_indented_string(&str_node);

        for part in str_node.parts() {
            let ast::InterpolPart::Literal(lit) = part else {
                continue;
            };
            let base: usize = lit.syntax().text_range().start().into();
            let mut chars = lit.syntax().text().char_indices();

            while let Some((_, ch)) = chars.next() {
                if ch == '\\'
                    && !is_multiline
                    && let Some((i, c)) = chars.next()
                    && !matches!(c, '\\' | '$' | '"' | 'r' | 'n' | 't')
                {
                    report(base + i, "\\", c, Some(format!("\\\\{c}")));
                } else if ch == '\''
                    && is_multiline
                    && let Some((_, '\'')) = chars.next()
                {
                    match chars.next() {
                        Some((_, '\'' | '$')) => continue,
                        Some((_, '\\')) => match chars.next() {
                            Some((_, 'n' | 'r' | 't' | '\'')) => continue,
                            Some((i, c)) => report(base + i, "''\\", c, None),
                            None => break,
                        },
                        _ => break,
                    }
                }
            }
        }
    }

    if problems.is_empty() {
        Ok(Success(()))
    } else {
        Ok(Failure(problems))
    }
}

/// Whether a string literal is of the `''...''` kind.
fn is_indented_string(str_node: &ast::Str) -> bool {
    str_node
        .syntax()
        .children_with_tokens()
        .filter_map(rnix::SyntaxElement::into_token)
        .next()
        .is_some_and(|t| t.text() == "''")
}

/// Collect the strings that spell out an executable path by hand, for the ratchet check in
/// `ratchet::File`.
///
/// Indented strings are skipped: they are usually scripts, and the path-to-string coercion may be
/// relied on. Strings in attribute name position are skipped because the fix does not apply there.
fn collect_executable_references(
    relative_path: &RelativePath,
    nix_file: &NixFile,
) -> BTreeMap<String, Vec<ExecutableReference>> {
    let mut references: BTreeMap<String, Vec<ExecutableReference>> = BTreeMap::new();

    for str_node in nix_file
        .syntax_root
        .syntax()
        .descendants()
        .filter_map(ast::Str::cast)
    {
        if is_indented_string(&str_node) {
            continue;
        }
        if str_node
            .syntax()
            .parent()
            .is_some_and(|parent| ast::Attrpath::can_cast(parent.kind()))
        {
            continue;
        }
        let Some(fix) = executable_reference_fix(&str_node) else {
            continue;
        };

        let text = collapse_whitespace(&str_node.syntax().text().to_string());
        let key = text.replace(' ', "");
        let index: usize = str_node.syntax().text_range().start().into();
        references
            .entry(key)
            .or_default()
            .push(ExecutableReference {
                location: location::Location::new(
                    relative_path,
                    nix_file.line_index.line(index),
                    nix_file.line_index.column(index),
                ),
                text,
                fix,
            });
    }

    references
}

/// Classify a `"`-string as one of the executable path shapes, or `None` when it is not one or
/// when the rewrite might not be safe.
fn executable_reference_fix(str_node: &ast::Str) -> Option<ExecutableReferenceFix> {
    use ast::InterpolPart::{Interpolation, Literal};

    match str_node.normalized_parts().as_slice() {
        [Interpolation(call)] => Some(ExecutableReferenceFix::Unwrap {
            call: get_exe_call(&call.expr()?)?,
        }),
        [Interpolation(package), Literal(literal)] => {
            let program = literal.strip_prefix("/bin/")?;
            if !is_program_name(program) {
                return None;
            }
            Some(ExecutableReferenceFix::GetExePrime {
                package: package_expression(&package.expr()?)?,
                program: program.to_owned(),
            })
        }
        [
            Interpolation(package),
            Literal(literal),
            Interpolation(main_program),
        ] if literal.as_str() == "/bin/" => {
            let package = package_expression(&package.expr()?)?;
            if main_program_package(&main_program.expr()?)? != package {
                return None;
            }
            Some(ExecutableReferenceFix::GetExe { package })
        }
        _ => None,
    }
}

/// Only plain program names are rewritten; anything else may be shell syntax or a nested path.
fn is_program_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '+' | '-'))
}

/// The source text of `X` in `"${X}/bin/..."` when `X` plausibly is a derivation whose `bin`
/// output is wanted. `getBin X` is accepted because `lib.getExe'` is defined through `getBin`.
fn package_expression(expr: &ast::Expr) -> Option<String> {
    if let ast::Expr::Apply(apply) = expr {
        let function = attribute_path(&apply.lambda()?)?;
        if function != ["getBin"] && function != ["lib", "getBin"] {
            return None;
        }
        let argument = apply.argument()?;
        if matches!(argument, ast::Expr::Apply(_)) {
            return None;
        }
        return package_expression(&argument);
    }

    let path = attribute_path(expr)?;
    if looks_like_output_or_path(path.last()?) {
        return None;
    }
    Some(path.join("."))
}

/// Explicit outputs must not be rewritten, since `getBin` may pick a different output.
/// Names ending in `path`, `dir` or `home` are usually strings, which `lib.getExe'` rejects.
fn looks_like_output_or_path(name: &str) -> bool {
    const OUTPUT_NAMES: [&str; 11] = [
        "out", "bin", "dev", "lib", "man", "doc", "info", "devdoc", "debug", "static", "outPath",
    ];
    let lower = name.to_ascii_lowercase();
    OUTPUT_NAMES.contains(&name)
        || ["path", "dir", "home"]
            .iter()
            .any(|suffix| lower.ends_with(suffix))
}

/// `X` as source text when `expr` is `X.meta.mainProgram`.
fn main_program_package(expr: &ast::Expr) -> Option<String> {
    let path = attribute_path(expr)?;
    let (package, selector) = path.split_at(path.len().checked_sub(2)?);
    if package.is_empty() || selector != ["meta", "mainProgram"] {
        return None;
    }
    Some(package.join("."))
}

/// `a.b.c` as `["a", "b", "c"]` for an identifier or a select chain of identifiers without a
/// default. Dynamic or quoted attributes give `None`.
fn attribute_path(expr: &ast::Expr) -> Option<Vec<String>> {
    match expr {
        ast::Expr::Ident(ident) => Some(vec![ident.ident_token()?.text().to_owned()]),
        ast::Expr::Select(select) if select.default_expr().is_none() => {
            let mut path = attribute_path(&select.expr()?)?;
            for attr in select.attrpath()?.attrs() {
                let ast::Attr::Ident(ident) = attr else {
                    return None;
                };
                path.push(ident.ident_token()?.text().to_owned());
            }
            Some(path)
        }
        _ => None,
    }
}

/// The collapsed source text of `expr` when it is a saturated call to `getExe` or `getExe'`,
/// optionally through `lib` or `lib.meta`.
fn get_exe_call(expr: &ast::Expr) -> Option<String> {
    let mut head = expr.clone();
    let mut arguments = 0;
    while let ast::Expr::Apply(apply) = head {
        head = apply.lambda()?;
        arguments += 1;
    }

    let path = attribute_path(&head)?;
    let path: Vec<&str> = path.iter().map(String::as_str).collect();
    let function = match path.as_slice() {
        [function] | ["lib", function] | ["lib", "meta", function] => *function,
        _ => return None,
    };
    let expected_arguments = match function {
        "getExe" => 1,
        "getExe'" => 2,
        _ => return None,
    };
    if arguments != expected_arguments {
        return None;
    }

    Some(collapse_whitespace(&expr.syntax().text().to_string()))
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Check that `optional` is not called with a list literal.
///
/// `optional condition [ value ]` produces either `[]` or `[[ value ]]`.
/// The latter is almost always unintended, and in derivation attributes
/// can interact particularly poorly with `__structuredAttrs`.
fn check_optional_list_parameters(
    relative_path: &RelativePath,
    nix_file: &NixFile,
) -> validation::Result<()> {
    let mut problems: Vec<Problem> = Vec::new();

    for apply in nix_file
        .syntax_root
        .syntax()
        .descendants()
        .filter_map(ast::Apply::cast)
    {
        let Some(ast::Expr::List(_)) = apply.argument() else {
            continue;
        };
        let Some(ast::Expr::Apply(optional_apply)) = apply.lambda() else {
            continue;
        };
        let Some(function) = optional_apply.lambda() else {
            continue;
        };
        let Some(optional_function) = OptionalFunction::from_expr(&function) else {
            continue;
        };

        let index: usize = function.syntax().text_range().start().into();
        problems.push(
            npv_171::NixFileContainsOptionalList::new(
                location::Location::new(
                    relative_path,
                    nix_file.line_index.line(index),
                    nix_file.line_index.column(index),
                ),
                optional_function.singular(),
                optional_function.plural(),
            )
            .into(),
        );
    }

    if problems.is_empty() {
        Ok(Success(()))
    } else {
        Ok(Failure(problems))
    }
}

/// The syntactic forms of `optional` covered by the upstream check.
enum OptionalFunction {
    Bare,
    Lib,
    LibLists,
}

impl OptionalFunction {
    fn from_expr(expr: &ast::Expr) -> Option<Self> {
        match expr {
            ast::Expr::Ident(ident) if ident.syntax().text() == "optional" => Some(Self::Bare),
            ast::Expr::Select(select) if select.default_expr().is_none() => {
                let ast::Expr::Ident(root) = select.expr()? else {
                    return None;
                };
                if root.syntax().text() != "lib" {
                    return None;
                }

                let attrpath = select.attrpath()?;
                let attrs: Vec<_> = attrpath.attrs().map(|attr| attr.syntax().text()).collect();

                match attrs.as_slice() {
                    [optional] if *optional == "optional" => Some(Self::Lib),
                    [lists, optional] if *lists == "lists" && *optional == "optional" => {
                        Some(Self::LibLists)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    const fn singular(&self) -> &'static str {
        match self {
            Self::Bare => "optional",
            Self::Lib => "lib.optional",
            Self::LibLists => "lib.lists.optional",
        }
    }

    const fn plural(&self) -> &'static str {
        match self {
            Self::Bare => "optionals",
            Self::Lib => "lib.optionals",
            Self::LibLists => "lib.lists.optionals",
        }
    }
}

/// Recursively collects all Nix files in the relative `dir` within `base`
/// into the `files` `Vec`.
fn collect_nix_files(
    base: &Path,
    dir: &RelativePath,
    files: &mut Vec<RelativePathBuf>,
) -> anyhow::Result<()> {
    for entry in structure::read_dir_sorted(&dir.to_path(base))? {
        let mut relative_path = dir.to_relative_path_buf();
        relative_path.push(entry.file_name().to_string_lossy().into_owned());

        let absolute_path = entry.path();

        // We'll get to every file based on directory recursion, no need to follow symlinks.
        if absolute_path.is_symlink() {
            continue;
        }
        if absolute_path.is_dir() {
            collect_nix_files(base, &relative_path, files)?
        } else if absolute_path.extension().is_some_and(|x| x == "nix") {
            files.push(relative_path)
        }
    }
    Ok(())
}
