use crate::nix_file;
use relative_path::Component;
use relative_path::RelativePath;
use relative_path::RelativePathBuf;
use rowan::ast::AstNode;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;
use std::str;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Reference {
    pub line: usize,

    // The longest ancestor of the referenced path that can be moved
    // around without breaking the reference
    // E.g. if the reference is `./foo`, then this is `./.`, since we can move the current
    // directory without breaking this reference. It can't be `./foo` because moving `./foo` around
    // would break the reference
    // Another example: If the reference is `../bar`, then movable_ancestor is `..`. It's not `./.`
    // because if we moved the current directory around we could break this reference.
    pub movable_ancestor: RelativePathBuf,

    pub rel_to_root: RelativePathBuf,

    pub text: String,
}

#[derive(Debug, Clone)]
pub struct PathIndex {
    pub references: Vec<Reference>,
    pub referenced_by: Vec<(RelativePathBuf, usize)>,
}

impl PathIndex {
    fn new() -> PathIndex {
        PathIndex {
            references: Vec::new(),
            referenced_by: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlobalIndex {
    // For each Nix file, what paths it references
    pub path_indices: HashMap<RelativePathBuf, PathIndex>,
}

impl GlobalIndex {
    pub fn new(
        nixpkgs_path: impl AsRef<Path>,
        paths: &HashSet<RelativePathBuf>,
        nix_file_store: &mut nix_file::NixFileStore,
    ) -> GlobalIndex {
        // TODO: Remove the unwrap's return Result

        let mut path_indices: HashMap<RelativePathBuf, PathIndex> = paths
            .iter()
            .map(|p| (p.clone(), PathIndex::new()))
            .collect();

        //eprintln!("{:#?}", path_indices);

        paths
            .iter()
            .filter(|p| !p.to_path(nixpkgs_path.as_ref()).is_dir() && p.extension() == Some("nix"))
            .for_each(|subpath| {
                //eprintln!("Processing {subpath}");

                let abs_path = subpath.to_path(nixpkgs_path.as_ref());

                if abs_path.is_dir() || subpath.extension() != Some("nix") {
                    return;
                }

                // TODO: Handle error
                let file = nix_file_store.get(&abs_path).unwrap();

                'nodes: for node in file.syntax_root.syntax().descendants() {
                    let text = node.text().to_string();
                    let line = file.line_index.line(node.text_range().start().into());

                    let Some(ast_path) = rnix::ast::Path::cast(node) else {
                        continue 'nodes;
                    };

                    //eprintln!("Processing reference {text} on line {line}");

                    // TODO: Error reporting
                    let nix_file::ResolvedPath::Within(mut rel_to_root, movable_ancestor) =
                        file.static_resolve_path(&ast_path, nixpkgs_path.as_ref())
                    else {
                        continue 'nodes;
                    };

                    let mut rel_to_source = RelativePathBuf::from(&text);

                    let abs = rel_to_root.to_path(&nixpkgs_path);

                    // FIXME: This should not be necessary, it's something `import` specific
                    if abs.is_dir() && abs.join("default.nix").exists() {
                        rel_to_root = rel_to_root.join("default.nix");
                        rel_to_source = rel_to_source.join("default.nix");
                    }

                    let reference = Reference {
                        line,
                        movable_ancestor,
                        rel_to_root,
                        text: text.clone(),
                    };

                    let path_index = path_indices.get_mut(&*subpath).unwrap();
                    let current_length = path_index.references.len();
                    let pointer = (subpath.clone(), current_length);

                    // Insert the reference
                    path_index.references.push(reference);
                    // We can't move the file that contains the reference itself without breaking the
                    // reference contained in it
                    path_index.referenced_by.push(pointer.clone());

                    let mut focused_dir = subpath.parent().unwrap().to_relative_path_buf();
                    //eprintln!("Focused dir is: {focused_dir}");
                    // The directory of the file is referenced by the file
                    path_indices
                        .get_mut(&focused_dir)
                        .unwrap()
                        .referenced_by
                        .push(pointer.clone());

                    for component in rel_to_source.components() {
                        match component {
                            Component::CurDir => {}
                            Component::ParentDir => {
                                path_indices
                                    .get_mut(&focused_dir)
                                    .unwrap()
                                    .referenced_by
                                    .push(pointer.clone());
                                focused_dir = focused_dir.parent().unwrap().to_relative_path_buf();
                                //eprintln!("Focused dir is: {focused_dir}");
                            }
                            Component::Normal(osstr) => {
                                focused_dir = focused_dir.join(osstr).to_relative_path_buf();
                                //eprintln!("Focused dir is: {focused_dir}");
                                path_indices
                                    .get_mut(&focused_dir)
                                    .unwrap()
                                    .referenced_by
                                    .push(pointer.clone());
                            }
                        }
                    }
                }
            });

        GlobalIndex { path_indices }
    }
}
