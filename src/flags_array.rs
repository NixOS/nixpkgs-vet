//! Find uses of shell flag arrays in Nix string literals.

use crate::location::Location;
use crate::nix_file::NixFile;
use relative_path::RelativePath;
use rnix::ast::{self, AstToken};
use rowan::ast::AstNode;
use std::collections::BTreeMap;
use std::sync::LazyLock;

pub fn find_uses(
    file: &NixFile,
    relative_path: &RelativePath,
    syntax: &rnix::SyntaxNode,
) -> BTreeMap<String, Vec<Location>> {
    static FLAGS_ARRAY: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"\b(?:[A-Za-z_][A-Za-z0-9_]*)?FlagsArray\b").expect("valid regex")
    });

    let mut uses = BTreeMap::<String, Vec<Location>>::new();
    for string in syntax.descendants().filter_map(ast::Str::cast) {
        let indented = string.syntax().text().to_string().starts_with("''");
        for part in string.parts() {
            let ast::InterpolPart::Literal(literal) = part else {
                continue;
            };
            let start: usize = literal.syntax().text_range().start().into();
            let (decoded, offsets) = decode_string_literal(literal.syntax().text(), indented);
            for name in FLAGS_ARRAY.find_iter(&decoded) {
                let offset = start + offsets[name.start()];
                uses.entry(name.as_str().to_owned())
                    .or_default()
                    .push(Location::new(
                        relative_path,
                        file.line_index.line(offset),
                        file.line_index.column(offset),
                    ));
            }
        }
    }
    uses
}

// Keep a source offset for each byte of decoded literal text.
fn decode_string_literal(text: &str, indented: bool) -> (String, Vec<usize>) {
    let mut decoded = String::new();
    let mut offsets = Vec::new();
    let mut i = 0;
    while i < text.len() {
        let rest = &text[i..];
        let escape_prefix_len = if indented {
            if rest.starts_with("''\\") {
                3
            } else if rest.starts_with("''$") || rest.starts_with("'''") {
                2
            } else {
                0
            }
        } else if rest.starts_with('\\') {
            1
        } else {
            0
        };
        let source = i;
        i += escape_prefix_len;
        if i == text.len() {
            break;
        }
        let ch = text[i..]
            .chars()
            .next()
            .expect("remaining text is not empty");
        i += ch.len_utf8();
        let ch = if escape_prefix_len > 0 && (!indented || escape_prefix_len == 3) {
            match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            }
        } else {
            ch
        };
        decoded.push(ch);
        offsets.extend(std::iter::repeat_n(source, ch.len_utf8()));
    }
    (decoded, offsets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{nix_file::NixFileStore, tests};
    use indoc::indoc;
    use relative_path::RelativePathBuf;

    #[test]
    fn detects_decoded_flags_arrays() -> anyhow::Result<()> {
        let dir = tests::tempdir()?;
        let path = dir.path().join("file.nix");
        std::fs::write(
            &path,
            r#"{ x = "\ncheckFlagsArray"; y = ''a''\ncheckFlagsArray''; }"#,
        )?;
        let mut store = NixFileStore::default();
        let file = store.get(&path)?;
        let uses = find_uses(
            file,
            RelativePath::new("file.nix"),
            file.syntax_root.syntax(),
        );
        assert_eq!(
            uses.keys().map(String::as_str).collect::<Vec<_>>(),
            vec!["checkFlagsArray"]
        );
        assert_eq!(uses["checkFlagsArray"].len(), 2);
        assert_eq!(uses["checkFlagsArray"][0].column, 10);
        Ok(())
    }

    #[test]
    fn detects_flags_arrays_in_strings() -> anyhow::Result<()> {
        let dir = tests::tempdir()?;
        let path = dir.path().join("file.nix");
        std::fs::write(
            &path,
            indoc! {r#"
            {
              # checkFlagsArray is ignored in Nix comments.
              /* makeFlagsArray is ignored too. */
              checkFlagsArray = null;
              preCheck = ''
                checkFlagsArray+=("one")
                echo "''${checkFlagsArray[@]}"
                customFlagsArray=(two)
              '';
              postBuild = "echo $makeFlagsArray";
              ignored = "checkFlagsArraySuffix 9makeFlagsArray makeFlags";
            }
        "#},
        )?;
        let mut store = NixFileStore::default();
        let file = store.get(&path)?;
        let uses = find_uses(
            file,
            RelativePath::new("file.nix"),
            file.syntax_root.syntax(),
        );
        assert_eq!(
            uses.iter()
                .map(|(name, locations)| (name.as_str(), locations.len()))
                .collect::<Vec<_>>(),
            vec![
                ("checkFlagsArray", 2),
                ("customFlagsArray", 1),
                ("makeFlagsArray", 1)
            ]
        );
        let first = &uses["checkFlagsArray"][0];
        assert_eq!(
            (&first.file, first.line, first.column),
            (&RelativePathBuf::from("file.nix"), 6, 5)
        );
        Ok(())
    }
}
