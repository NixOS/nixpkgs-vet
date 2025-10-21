---
default: minor
---

# Remove support for by-name overrides
A substantial portion of the codebase was dedicated to handling overrides in
all-packages.nix of packages defined in pkgs/by-name.

These are now disallowed by NixOS/nixpkgs#483820, so this functionality is no
longer needed.
