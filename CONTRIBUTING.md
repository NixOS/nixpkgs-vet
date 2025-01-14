# Contributor documentation

[@infinisil](https://github.com/infinisil) is the admin and main developer of this repository,
while everybody in [@NixOS/nixpkgs-vet](https://github.com/orgs/NixOS/teams/nixpkgs-vet) has write access.

## Development

Enter the development environment in this directory either automatically with
[`direnv`](https://github.com/direnv/direnv) or manually with
```bash
nix-shell
```

The most important tools and commands in this environment are:

- [rust-analyzer](https://rust-analyzer.github.io/) to have an IDE-like experience for your own editor.

- Running tests:

  ```bash
  cargo test
  ```

- Formatting:

  ```bash
  treefmt
  ```

- Linting:

  ```bash
  cargo clippy --all-targets
  ```

  Optionally, check out extra lints or uncomment them at the top of `main.rs`.

  ```bash
  cargo clippy --all-targets -- -W clippy::nursery -W clippy::pedantic
  ```

- Running the [main CI checks](./.github/workflows/main.yml) locally:
  ```bash
  nix-build -A ci
  ```

### Integration tests

Integration tests are declared in [`./tests`](./tests) as subdirectories imitating Nixpkgs with these files:
- `main`: A Nixpkgs root directory with:
  - `default.nix`:
    Always contains
    ```nix
    import <test-nixpkgs> { root = ./.; }
    ```
    which makes
    ```
    nix-instantiate <subdir> --eval -A <attr> --arg overlays <overlays>
    ```
    work very similarly to the real Nixpkgs, just enough for the program to be able to test it.
  - `pkgs/by-name`:
    The `pkgs/by-name` directory to check.

  - `pkgs/top-level/all-packages.nix` (optional):
    Contains an overlay of the form
    ```nix
    self: super: {
      # ...
    }
    ```
    allowing the simulation of package overrides to the real [`pkgs/top-level/all-packages.nix`](https://github.com/NixOS/nixpkgs/blob/master/pkgs/top-level/all-packages.nix).
    The default is an empty overlay.

- `base` (optional):
  Contains another Nixpkgs root directory with potentially any of the above structures.
  This is used to test [ratchet checks](./README.md#ratchet-checks).

- `expected` (optional):
  A file containing the expected standard output.
  The default is expecting an empty standard output.

  This file is matched against the error almost literally,
  with the only exception being that the `@REDACTED@` string can match anything,
  which is useful for non-deterministic errors.

## Automation

Pinned dependencies are [regularly updated automatically](./.github/workflows/update.yml).

Releases are [automatically](./.github/workflows/main.yml) [created](./scripts/release.sh) when the `version` field in [`Cargo.toml`](./Cargo.toml)
is updated from a push to the main branch.
