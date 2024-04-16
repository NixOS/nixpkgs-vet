# Contributor documentation

[@infinisil](https://github.com/infinisil) is the admin and main developer of this repository,
while everybody in [@NixOS/nixpkgs-check-by-name](https://github.com/orgs/NixOS/teams/nixpkgs-check-by-name) has write access.

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
- Linting and formatting:
  ```bash
  cargo clippy --all-targets
  treefmt
  ```
- Running the [main CI checks](./.github/workflows/main.yml) locally:
  ```bash
  nix-build -A ci
  ```

Note that pinned dependencies are [regularly updated automatically](./.github/workflows/update.yml).

### Integration tests

Integration tests are declared in [`./tests`](./tests) as subdirectories imitating Nixpkgs with these files:
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
  Contains another subdirectory imitating Nixpkgs with potentially any of the above structures.
  This is used to test [ratchet checks](./README.md#ratchet-checks).

- `expected` (optional):
  A file containing the expected standard output.
  The default is expecting an empty standard output.

## Releases and changelogs

The following pipeline is used to ensure a smooth releases process with automated changelogs.

### Pull requests

The default [PR template](./.github/pull_request_template.md) adds this line to the description:

> - [x] This change is user-facing

Unless this field is explicitly unchecked, the PR [is checked to](./.github/workflows/check-changelog.yml)
add a [changelog entry](#changelog-entries) to describe the user-facing change.

This ensures that all user-facing changes have a changelog entry.

### Changelog entries

In order to avoid conflicts between different PRs,
a changelog entry is a Markdown file under a directory in
[`changes/unreleased`](./changes/unreleased).
Depending on the effort (see [EffVer](https://jacobtomlinson.dev/effver/))
required for users to update to this change,
a different directory should be used:

- [`changes/unreleased/major`](./changes/unreleased/major):
  A large effort. This will cause a version bump from e.g. 0.1.2 to 1.0.0
- [`changes/unreleased/medium`](./changes/unreleased/medium):
  Some effort. This will cause a version bump from e.g. 0.1.2 to 1.2.0
- [`changes/unreleased/minor`](./changes/unreleased/minor):
  Little/no effort. This will cause a version bump from e.g. 0.1.2 to 0.1.3

The Markdown file must have the `.md` file ending, and be of the form

```markdown
# Some descriptive title of the change

Optionally more information
```

### Release branch

After every push to the main branch, the [infinixbot:release
branch](https://github.com/infinixbot/nixpkgs-check-by-name/tree/release) is rebased such that it
contains one commit on top of master, which:
- Increments the version in `Cargo.toml` according to the unreleased changelog entries.
- Collects all changelog entries in [`./changes/unreleased`](./changes/unreleased)
  and combines them into a new `./changes/released/<version>.md` file.

Regularly a PR is [opened automatically](./.github/workflows/regular-release.yml)
to merge the release branch into the main branch.
When this PR is merged, a GitHub release is [automatically created](./.github/workflows/release.yml).
