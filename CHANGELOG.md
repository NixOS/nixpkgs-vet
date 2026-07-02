## 0.3.4 (2026-07-01)

### Features

#### Remove support for by-name overrides (#189)

A substantial portion of the codebase was dedicated to handling overrides in
all-packages.nix of packages defined in pkgs/by-name.

These are now disallowed by NixOS/nixpkgs#483820, so this functionality is no
longer needed.

By @mdaniels5757

## 0.3.3 (2026-06-19)

### Features

#### Ignore ineffective package attrs in `strictDeps` & `__structuredAttrs` checks (#265)

Previously, setting e.g. `passthru.__structuredAttrs` would satisfy our checks, even though this is not effective to actually enable `__structuredAttrs` in the derivation.

Detection of these features now more closely aligns with the actual derivation's use of them.

By @RossSmyth

## 0.3.2 (2026-06-05)

Fixed release of v0.3.1.

## 0.3.1 (2026-05-26)

### Features

* **nixpkgs-vet --version** by Tom van Dijk (e20b465)

### Fixes

* **remove OSC 8 hyperlinks (#241)** by Michael Daniels (d846031)

## 0.3.0 (2026-04-05)

### Changes
* On .nix files, disallow executable bit without shebang (NPV-145) and shebang without executable bit (NPV-146) by @philiptaron in https://github.com/NixOS/nixpkgs-vet/pull/199
* Add ratchet checks for `__structuredAttrs` and `strictDeps` by @tobim in https://github.com/NixOS/nixpkgs-vet/pull/203
* disallow useless escapes by @mdaniels5757 in https://github.com/NixOS/nixpkgs-vet/pull/211

## 0.2.0 (2026-03-22)

### Changes
* README.md: update for nixpkgs-vet by @philiptaron in https://github.com/NixOS/nixpkgs-vet/pull/106
* Number every problem reported by `nixpkgs-vet` by @philiptaron in https://github.com/NixOS/nixpkgs-vet/pull/109
* Fix new clippy lints by @willbush in https://github.com/NixOS/nixpkgs-vet/pull/119
* .envrc: watch npins folder by @willbush in https://github.com/NixOS/nixpkgs-vet/pull/118
* Clear some clippy::pedantic pings by @Ben-PH in https://github.com/NixOS/nixpkgs-vet/pull/120
* Lint cleanup by @willbush in https://github.com/NixOS/nixpkgs-vet/pull/127
* Replace lazy_static with std::sync::LazyLock by @willbush in https://github.com/NixOS/nixpkgs-vet/pull/123
* Ensure dependabot doesn't break over time by @infinisil in https://github.com/NixOS/nixpkgs-vet/pull/126
* readme: fix link to `builtins.currentSystem` by @willbush in https://github.com/NixOS/nixpkgs-vet/pull/132
* Fix cargo upgrade to not break by using `--incompatible` by @willbush in https://github.com/NixOS/nixpkgs-vet/pull/131
* workflows: fix xrefcheck detection on invalid symlinks in tests dir by @willbush in https://github.com/NixOS/nixpkgs-vet/pull/140
* Use BTreeMap to get a sorted map instead of Vec + HashMap by @infinisil in https://github.com/NixOS/nixpkgs-vet/pull/143
* Update nixpkgs-check-by-name reference in CODEOWNERS by @infinisil in https://github.com/NixOS/nixpkgs-vet/pull/147
* File ratchet checks by @infinisil in https://github.com/NixOS/nixpkgs-vet/pull/144
* .github: run CI with `max-jobs` set to 1 in order to avoid OOM by @philiptaron in https://github.com/NixOS/nixpkgs-vet/pull/160
* feat: nixpkgs-vet --version by @dtomvan in https://github.com/NixOS/nixpkgs-vet/pull/158
* Update permissions by @infinisil in https://github.com/NixOS/nixpkgs-vet/pull/176
* Drop nix minimum to fix CI by @willbush in https://github.com/NixOS/nixpkgs-vet/pull/177
* fix autoPrUpdate scripts/CI by @mdaniels5757 in https://github.com/NixOS/nixpkgs-vet/pull/183
* Update rust edition to 2024 by @willbush in https://github.com/NixOS/nixpkgs-vet/pull/179
* update npins by @mdaniels5757 in https://github.com/NixOS/nixpkgs-vet/pull/190
* CODEOWNERS: replace nixpkgs-vet with nixpkgs-ci team by @mdaniels5757 in https://github.com/NixOS/nixpkgs-vet/pull/192
* npins: switch nixpkgs from master branch back to nixpkgs-unstable channel by @mdaniels5757 in https://github.com/NixOS/nixpkgs-vet/pull/193
* Update rnix 0.12.0 to 0.14.0 with new path lints by @philiptaron in https://github.com/NixOS/nixpkgs-vet/pull/197
* Don't allow packages that start with dash or a digit by @Ben-PH in https://github.com/NixOS/nixpkgs-vet/pull/122
* Do not encourage merging with CI failure by @philiptaron in https://github.com/NixOS/nixpkgs-vet/pull/198
* Include wiki links in error output by @philiptaron in https://github.com/NixOS/nixpkgs-vet/pull/200
