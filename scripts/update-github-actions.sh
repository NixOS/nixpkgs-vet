#!/usr/bin/env bash

# This script calls the dependabot CLI (https://github.com/dependabot/cli)
# to determine updates to GitHub Action dependencies in the local repository.
# It then also applies the updates and outputs the results to standard output.
# Hopefully there's built-in support for this in the future, see
# https://github.com/dependabot/cli/issues/301

set -euo pipefail

REPO_ROOT=$1

echo "<details><summary>GitHub Action updates</summary>"
# Needed because GitHub's rendering of the first body line breaks down otherwise
echo ""

# CI sets the GH_TOKEN env var, which `gh auth token` defaults to if set
githubToken=$(gh auth token)

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' exit

# Use dependency groups to update all dependencies together
# Note that repo is not used because we pass `--local` on the CLI
cat <<EOF > "$tmp/input.yml"
job:
  package-manager: "github_actions"
  allowed-updates:
    - update-type: all
  source:
    directory: "/"
    provider: github
    repo: not/used
  dependency-groups:
    - name: actions
      rules:
        patterns:
          - "*"
EOF

if ! create_pull_request=$(LOCAL_GITHUB_ACCESS_TOKEN="$githubToken" \
  dependabot update --file "$tmp/input.yml" --local "$REPO_ROOT" \
  | jq --exit-status 'select(.type == "create_pull_request").data'); then
  # Nothing to update
  exit 0
fi

jq --exit-status --raw-output '."pr-body"' <<< "$create_pull_request"

echo '</details>'

jq --compact-output '."updated-dependency-files"[]' <<< "$create_pull_request" \
  | while read -r fileUpdate; do
    file=$(jq --exit-status --raw-output '.name' <<< "$fileUpdate")
    # --join-output makes sure to not output a trailing newline
    jq --exit-status --raw-output --join-output '.content' <<< "$fileUpdate" > "$REPO_ROOT/$file"
  done
