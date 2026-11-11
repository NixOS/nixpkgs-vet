---
default: minor
---

# Ratchet `*FlagsArray` uses with structured attributes

Top-level packages with `__structuredAttrs = true` must use `*Flags` instead of
introducing `*FlagsArray` variable names in string literals in their defining Nix file
(NPV-175). With structured attributes, `*Flags` is already a Bash array.

The ratchet allows additional uses of variables already present in a package, but
rejects new variable names when structured attributes are enabled.
