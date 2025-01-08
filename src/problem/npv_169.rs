use derive_new::new;
use relative_path::RelativePathBuf;
use std::fmt;

#[derive(Clone, new)]
pub struct TopLevelWithMayShadowVariablesAndBreakStaticChecks {
    #[new(into)]
    file: RelativePathBuf,
}

impl fmt::Display for TopLevelWithMayShadowVariablesAndBreakStaticChecks {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { file } = self;
        write!(f, "- {file}: Top level with is discouraged as it may shadow variables and break static checks.")
    }
}
