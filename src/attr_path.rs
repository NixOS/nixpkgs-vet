//! Attribute paths like `python3Packages.requests`, stored as their individual attribute names.
//!
//! [`AttrPathBuf`] owns its names and [`AttrPath`] borrows them, the same way `PathBuf` and `Path`
//! relate in the standard library. Any `&[String]` can be viewed as an `&AttrPath` without copying.

use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;

use ref_cast::RefCast;
use serde::Deserialize;

/// An owned attribute path, e.g. `["python3Packages", "requests"]` for
/// `pkgs.python3Packages.requests`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(transparent)]
pub struct AttrPathBuf(Vec<String>);

/// A borrowed attribute path. Each element is one attribute name, starting at the top level of
/// `pkgs`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, RefCast)]
#[repr(transparent)]
pub struct AttrPath([String]);

impl AttrPath {
    pub fn new(names: &[String]) -> &Self {
        Self::ref_cast(names)
    }

    pub fn names(&self) -> &[String] {
        &self.0
    }

    /// The name of the attribute itself, without the attribute sets containing it.
    pub fn last(&self) -> &str {
        self.0.last().expect("attribute paths are never empty")
    }

    /// The attribute name if this is a top-level attribute like `pkgs.hello`.
    pub fn top_level(&self) -> Option<&str> {
        match &self.0 {
            [name] => Some(name),
            _ => None,
        }
    }
}

impl fmt::Display for AttrPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, name) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(".")?;
            }
            f.write_str(name)?;
        }
        Ok(())
    }
}

impl fmt::Display for AttrPathBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl Deref for AttrPathBuf {
    type Target = AttrPath;

    fn deref(&self) -> &AttrPath {
        AttrPath::new(&self.0)
    }
}

impl Borrow<AttrPath> for AttrPathBuf {
    fn borrow(&self) -> &AttrPath {
        self
    }
}

impl ToOwned for AttrPath {
    type Owned = AttrPathBuf;

    fn to_owned(&self) -> AttrPathBuf {
        AttrPathBuf(self.0.to_vec())
    }
}

impl From<&AttrPath> for AttrPathBuf {
    fn from(path: &AttrPath) -> Self {
        path.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn path(names: &[&str]) -> AttrPathBuf {
        AttrPathBuf(names.iter().map(|name| name.to_string()).collect())
    }

    #[test]
    fn top_level() {
        assert_eq!(path(&["hello"]).top_level(), Some("hello"));
        assert_eq!(path(&["python3Packages", "requests"]).top_level(), None);
    }

    #[test]
    fn display_joins_names() {
        assert_eq!(path(&["hello"]).to_string(), "hello");
        assert_eq!(
            path(&["python3Packages", "requests"]).to_string(),
            "python3Packages.requests"
        );
    }

    #[test]
    fn map_lookup_with_borrowed_path() {
        let map = BTreeMap::from([(path(&["a", "b"]), 1), (path(&["a"]), 2)]);
        let names = ["a".to_string(), "b".to_string()];
        assert_eq!(map.get(AttrPath::new(&names)), Some(&1));
    }
}
