---
default: minor
---

# Ban `dontCargoUseSystemLibs = true`

Top-level packages must never set `dontCargoUseSystemLibs` to `true`. This is for
the security of our users. Setting this attribute to true is too big of a hammer as
it allows any vendored libraries in a Rust package (NPV-176). Rather, if there are specific
libraries that cannot be linked to nixpkgs-provided libraries, they should have more
targeted fixes rather than a blanket allow.
