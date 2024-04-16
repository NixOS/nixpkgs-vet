#!/usr/bin/env bash

set -euo pipefail
shopt -s nullglob

root=$1
currentPrNumber=${2:-}

[[ "$(toml get --raw "$root"/Cargo.toml package.version)" =~ ([0-9]+)\.([0-9]+)\.([0-9]+) ]]
splitVersion=("${BASH_REMATCH[@]:1}")

majorChanges=("$root"/changes/unreleased/major/*.md)
mediumChanges=("$root"/changes/unreleased/medium/*.md)
minorChanges=("$root"/changes/unreleased/minor/*.md)

if ((${#majorChanges[@]} > 0)); then
  # If we didn't have `|| true` this would exit the program due to `set -e`,
  # because (( ... )) returns the incremental value, which is treated as the exit code..
  ((splitVersion[0]++)) || true
  splitVersion[1]=0
  splitVersion[2]=0
elif ((${#mediumChanges[@]} > 0)); then
  ((splitVersion[1]++)) || true
  splitVersion[2]=0
elif ((${#minorChanges[@]} > 0)); then
  ((splitVersion[2]++)) || true
else
  echo >&2 "No changes"
  exit 0
fi

next=${splitVersion[0]}.${splitVersion[1]}.${splitVersion[2]}
releaseFile=$root/changes/released/${next}.md
mkdir -p "$(dirname "$releaseFile")"

echo "# Version $next ($(date --iso-8601 --utc))" > "$releaseFile"
echo "" >> "$releaseFile"

# shellcheck disable=SC2016
for file in "${majorChanges[@]}" "${mediumChanges[@]}" "${minorChanges[@]}"; do
  commit=$(git -C "$root" log -1 --format=%H -- "$file")
  if ! gh api graphql \
    -f sha="$commit" \
    -f query='
    query ($sha: String) {
      repository(owner: "NixOS", name: "nixpkgs-check-by-name") {
        commit: object(expression: $sha) {
          ... on Commit {
            associatedPullRequests(first: 100) {
              nodes {
                merged
                baseRefName
                baseRepository { nameWithOwner }
                number
                author { login }
              }
            }
          }
        }
      }
    }' |
    jq --exit-status -r ${currentPrNumber:+--argjson currentPrNumber "$currentPrNumber"} --arg file "$file" '
    .data.repository.commit.associatedPullRequests?.nodes?[]?
      | select(
        # We need to make sure to get the right PR, there can be many
        (.merged or .number == $ARGS.named.currentPrNumber) and
        .baseRepository.nameWithOwner == "NixOS/nixpkgs-check-by-name" and
        .baseRefName == "main")
      | "\(.number) \(.author.login) \($ARGS.named.file)"'; then
    echo >&2 "Couldn't get PR for file $file"
    exit 1
  fi
done |
  sort -n |
  while read -r number author file; do
    # Replace the first line `# <title>` by `- <title> by @author in #number`
    # All other non-empty lines are indented with 2 spaces to make the markdown formatting work
    sed "$file" \
      -e "1s|#[[:space:]]\(.*\)|- \1 by [@$author](https://github.com/$author) in [#$number](https://github.com/NixOS/nixpkgs-check-by-name/pull/$number)|" \
      -e '2,$s/^\(.\)/  \1/' >> "$releaseFile"

    rm "$file"
  done

cargo set-version --manifest-path "$root"/Cargo.toml "$next"
echo "$next"
