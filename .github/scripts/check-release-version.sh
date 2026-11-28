#!/usr/bin/env bash
set -euo pipefail

version=${1:?Expected a stable X.Y.Z version}
stable='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$'
[[ "$version" =~ $stable ]] || { echo "::error::Invalid release version $version."; exit 1; }
tags="$(gh api --paginate "repos/$GITHUB_REPOSITORY/tags?per_page=100" --jq '.[].name')"
latest="$(awk '/^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/' <<<"$tags" | LC_ALL=C sort -V | tail -n 1)"
[[ -n "$latest" ]] || exit 0

IFS=. read -r major minor patch <<<"$latest"
next_patch="$major.$minor.$((patch + 1))"
next_minor="$major.$((minor + 1)).0"
next_major="$((major + 1)).0.0"
if [[ "$version" != "$next_patch" && "$version" != "$next_minor" && "$version" != "$next_major" ]]; then
    echo "::error::After $latest, the next release must be $next_patch, $next_minor, or $next_major (got $version)."
    exit 1
fi
