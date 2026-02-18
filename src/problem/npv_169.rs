use std::fmt;

use indoc::writedoc;
use relative_path::RelativePathBuf;

#[derive(Clone)]
pub struct OverlyBroadWith {
    file: RelativePathBuf,
}

impl OverlyBroadWith {
    pub fn new(file: RelativePathBuf) -> Self {
        Self { file }
    }
}

impl fmt::Display for OverlyBroadWith {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { file } = self;
        writedoc!(
            f,
            "
            - {file}: `with` expression covers too much of the file.
              Large-scoped `with` expressions shadow bindings and make static analysis unreliable.
              Prefer one of these alternatives:
              - Use fully qualified names, e.g. `lib.mkOption` instead of `with lib; mkOption`
              - Use `inherit (lib) mkOption mkIf;` in a `let` block
              - Limit `with` to small scopes, e.g. `maintainers = with lib.maintainers; [ ... ]`
            ",
        )
    }
}
