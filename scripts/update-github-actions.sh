#!/usr/bin/env bash

# This script calls the dependabot CLI (https://github.com/dependabot/cli)
# to determine updates to GitHub Action dependencies in the local repository.
# It then also applies the updates and outputs the results to standard output.

set -euo pipefail

REPO_ROOT=$1

echo -e "<details><summary>GitHub Action updates</summary>\n\n"

# CI sets the GH_TOKEN env var, which `gh auth token` defaults to if set
githubToken=$(gh auth token)

# Each dependabot update call tries to update all dependencies,
# but it outputs individual results for each dependency,
# with the intention of creating a PR for each.
#
# We want to have all changes together though,
# so we'd need to merge updates of the same files together,
# which could cause merge conflicts, no good.
#
# Instead, we run dependabot repeatedly,
# each time only taking the first dependency update and updating the files with it,
# such that the next iteration takes into account the previous updates.
# We do this until there's no more dependencies to be updated,
# at which point --exit-status will make jq return with a non-zero exit code.
#
# This does mean that dependabot internally needs to perform O(n^2) updates,
# but this isn't a problem in practice, since we run these updates regularly,
# so n is low.
while
  # Unused argument would be the remote GitHub repo, which is not used if we pass --local
  create_pull_request=$(LOCAL_GITHUB_ACCESS_TOKEN="$githubToken" \
    dependabot update github_actions this-argument-is-unused --local "$REPO_ROOT" \
    | jq --exit-status --compact-output --slurp 'map(select(.type == "create_pull_request")) | .[0].data')
do
  title=$(jq --exit-status --raw-output '."pr-title"' <<< "$create_pull_request")
  echo "<details><summary>$title</summary>"

  # Needed because GitHub's rendering of the first body line breaks down otherwise
  echo ""

  jq --exit-status --raw-output '."pr-body"' <<< "$create_pull_request"
  echo '</details>'

  jq --compact-output '."updated-dependency-files"[]' <<< "$create_pull_request" \
    | while read -r fileUpdate; do
      file=$(jq --exit-status --raw-output '.name' <<< "$fileUpdate")
      # --join-output makes sure to not output a trailing newline
      jq --exit-status --raw-output --join-output '.content' <<< "$fileUpdate" > "$REPO_ROOT/$file"
    done
done

echo -e "</details>"
