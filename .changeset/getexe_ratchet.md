---
default: minor
---

#### Ratchet against hand-written executable paths (NPV-172)

New or edited Nix files may no longer introduce `"${pkg}/bin/${pkg.meta.mainProgram}"`,
`"${pkg}/bin/name"` or `"${lib.getExe pkg}"`. Use `lib.getExe pkg`, `lib.getExe' pkg "name"`
or the bare `lib.getExe pkg` call instead. Existing instances are grandfathered per file.
This replaces the treewide rewrite attempted in
[NixOS/nixpkgs#318290](https://github.com/NixOS/nixpkgs/pull/318290).
