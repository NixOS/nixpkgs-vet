#!/usr/bin/env bash

set -euo pipefail

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' exit

nixeval() {
  # Since there's no stable `nix eval --raw`, also see https://github.com/NixOS/nix/pull/9361
  nix-instantiate --eval --json "$@" | jq -r .
}

# The system to pre-build the release for and distribute artifacts for
system=x86_64-linux
root=$(git rev-parse --show-toplevel)
repository=${GITHUB_REPOSITORY:-NixOS/nixpkgs-check-by-name}

# Get the version from the Cargo.toml file
version=$(nixeval "$root" -A version)
echo "Current version is $version"

if existingRelease=$(gh api \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  /repos/"$repository"/releases/tags/"$version"); then
  echo "Release $version already exists, no new release necessary"
  exit 0
else
  echo "$existingRelease"
  echo "Release $version doesn't exist yet, creating it"
fi

echo "Building release artifact for system $system"

nix-build "$root" -A build -o "$tmp/build" > /dev/null
readarray -t closure < <(nix-store -qR "$tmp/build")
nix-store --export "${closure[@]}" > "$tmp/$system.nar"
gzip "$tmp/$system.nar"
artifactName=$system.nar.gz

body='Automated release.

The artifact is a gzip-compressed [Nix Archive](https://nixos.org/manual/nix/stable/command-ref/nix-store/export.html) of the [build closure](https://nixos.org/manual/nix/stable/glossary#gloss-closure).

To import it:
```bash
gzip -cd '"$artifactName"' | nix-store --import | tail -1
```
'

echo "Creating draft release"
if ! release=$(gh api \
  --method POST \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  /repos/"$repository"/releases \
  -f tag_name="$version" \
  -f name="Version $version" \
  -f body="$body" \
  -F draft=true); then
  echo "Failed to create release: $release"
fi

releaseId=$(jq .id <<< "$release")

abortRelease() {
  echo "Aborting, deleting the draft release"
  gh api \
    --method DELETE \
    -H "Accept: application/vnd.github+json" \
    -H "X-GitHub-Api-Version: 2022-11-28" \
    /repos/"$repository"/releases/"$releaseId"
  exit 1
}

echo "Uploading release artifact"
# GitHub docs say to use the releases' upload_url, but that's of the RFC 6570 form, see
# https://docs.github.com/en/rest/using-the-rest-api/getting-started-with-the-rest-api?apiVersion=2022-11-28#hypermedia
# And neither the GitHub CLI nor curl seem to have support for that right now, so let's do it
# manually instead
if ! uploadResult=$(curl -sSfL \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $(gh auth token)" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  -H "Content-Type: application/octet-stream" \
  "https://uploads.github.com/repos/$repository/releases/$releaseId/assets?name=$artifactName" \
  --data-binary "@$tmp/$artifactName"); then
  echo "Failed to upload artifact: $uploadResult"
  abortRelease
fi

if ! publishResult=$(gh api \
  --method PATCH \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  /repos/"$repository"/releases/"$releaseId" \
  -F draft=false); then
  echo "Failed to publish release: $publishResult"
  abortRelease
fi

echo "Published release: $(jq .html_url -r <<< "$publishResult")"
