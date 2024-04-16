#!/usr/bin/env bash

set -euo pipefail
shopt -s nullglob

root=$1
prNumber=$2

# The PR template has this, selected by default
userFacingString="- [x] This change is user-facing"
nonUserFacingString="- [ ] This change is user-facing"

# Run this first to validate files
for file in "$root"/changes/unreleased/*/*; do
  if [[ "$(basename "$file")" == ".gitkeep" ]]; then
    continue
  fi
  if [[ ! "$file" == *.md ]]; then
    echo "File $file: Must be a markdown file with file ending .md"
    exit 1
  fi
  if [[ "$(sed -n '/^#/=' "$file")" != "1" ]]; then
    echo "File $file: The first line must start with #, while all others must not start with #"
    exit 1
  fi
done

body=$(gh api \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  /repos/NixOS/nixpkgs-check-by-name/pulls/"$prNumber" \
  | jq -r '.body')

if grep -F -- "$userFacingString" <<< "$body" >/dev/null ; then
  echo "User-facing change, changelog necessary"
elif grep -F -- "$nonUserFacingString" <<< "$body" >/dev/null; then
  echo "Not a user-facing change, no changelog necessary"
  exit 0
else
  echo "Depending on whether this PR has a user-facing change, add one of these lines to the PR description:"
  printf "%s\n" "$userFacingString"
  printf "%s\n" "$nonUserFacingString"
  exit 1
fi

# This checks whether the most recent commit changed any files in changes/unreleased
# This works well for PR's CI because there it runs on the merge commit,
# where HEAD^ is the first parent commit, which is the base branch.
if [[ -z "$(git -C "$root" log HEAD^..HEAD --name-only "$root"/changes/unreleased)" ]]; then
  echo "If this PR contains a user-facing change, add a changelog in ./changes/unreleased"
  echo "Otherwise, check the checkbox:"
  printf "%s\n" "$nonUserFacingString"
  exit 1
else
  echo "A changelog exists"
fi
