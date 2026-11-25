use derive_new::new;
use std::fmt;

#[derive(Clone, Debug, new)]
pub struct ConfigFileIsBroken {
    pub error: String,
}

impl fmt::Display for ConfigFileIsBroken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\n- Config file couldn't be read properly, see error above.",
            self.error
        )
    }
}
