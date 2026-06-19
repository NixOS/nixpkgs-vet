---
default: minor
---

# Ignore ineffective package attrs in `strictDeps` & `__structuredAttrs` checks

Previously, setting e.g. `passthru.__structuredAttrs` would satisfy our checks, even though this is not effective to actually enable `__structuredAttrs` in the derivation.

Detection of these features now more closely aligns with the actual derivation's use of them.
