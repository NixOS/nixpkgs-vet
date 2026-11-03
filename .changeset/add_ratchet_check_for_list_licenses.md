---
default: minor
---

# Add ratchet check for list-valued `meta.license`

Nixpkgs is moving from `meta.license = [ ... ]` to compound license expressions built with `lib.licenses.OR` and `lib.licenses.AND`.

New top-level packages must no longer use a list for `meta.license` (NPV-168), and packages that don't use a list can't regress to using one (NPV-169).
