use relative_path::RelativePathBuf;
use std::fmt;

#[derive(Clone)]
pub struct TopLevelWithMayShadowVariablesAndBreakStaticChecks {
    file: RelativePathBuf,
    node: String,
}

impl TopLevelWithMayShadowVariablesAndBreakStaticChecks {
    pub fn new(file: RelativePathBuf, node: String) -> Self {
        Self { file, node }
    }
}

impl fmt::Display for TopLevelWithMayShadowVariablesAndBreakStaticChecks {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { file, node: _node } = self;
        write!(f, "- {file}: Top level with is discouraged as it may shadow variables and break static checks.")
    }
}
