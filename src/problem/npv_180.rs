use itertools::Itertools;
use relative_path::RelativePathBuf;
use std::collections::BTreeSet;
use std::fmt;

use derive_new::new;
use indoc::writedoc;

#[derive(Clone, Debug, new)]
pub struct DependentsAttrsShouldBeSet {
    #[new(into)]
    package_path: Vec<String>,
    #[new(into)]
    package_file: RelativePathBuf,
    #[new(into)]
    pname: Option<String>,
    #[new(into)]
    name: Option<String>,
    #[new(into)]
    referenced_by_files: BTreeSet<RelativePathBuf>,
}

impl fmt::Display for DependentsAttrsShouldBeSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match (&self.pname, &self.name) {
            (Some(pname), _) => format!("pname: {pname}, "),
            (None, Some(name)) => format!("name: {name}, "),
            (None, None) => String::new(),
        };
        writedoc!(
            f,
            "
            - {} (defined by {}, {}{} pot. dependents): {}
            ",
            self.package_path.join("."),
            self.package_file,
            name,
            self.referenced_by_files.len(),
            self.referenced_by_files.iter().take(5).join(", "),
        )
    }
}
