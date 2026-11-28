#!/usr/bin/env bash
# Report a published checksum or "absent" for a crates.io 404.
set -euo pipefail

package=${1:?Expected a crate name}
version=${2:?Expected a crate version}
body="$(mktemp)"
trap 'rm -f -- "$body"' EXIT
status="$(curl --silent --show-error --max-time 15 --output "$body" --write-out '%{http_code}' \
    --user-agent "$package-release" "https://crates.io/api/v1/crates/$package/$version")"
case "$status" in
    404) echo absent ;;
    200)
        checksum="$(jq -er '.version.checksum | select(type == "string")' "$body")"
        [[ "$checksum" =~ ^[0-9a-f]{64}$ ]] ||
            { echo "Invalid crates.io checksum for $package $version." >&2; exit 1; }
        echo "$checksum"
        ;;
    *)
        echo "crates.io returned HTTP $status for $package $version." >&2
        exit 1
        ;;
esac
