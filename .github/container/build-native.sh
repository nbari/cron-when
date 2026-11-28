#!/usr/bin/env bash
# Package an already built binary in the runtime image.
set -euo pipefail
arch=${1:?Expected amd64 or arm64}
binary=${2:?Expected a binary path}
image=${3:?Expected an image tag}
engine=${CONTAINER_ENGINE:-docker}
[[ "$arch" == amd64 || "$arch" == arm64 ]] || { echo "Unsupported architecture: $arch" >&2; exit 1; }
context="$(mktemp -d)"
trap 'rm -rf -- "$context"' EXIT
install -D -m 0555 -- "$binary" "$context/linux/$arch/cron-when"
"$engine" build --platform "linux/$arch" --build-arg "TARGETARCH=$arch" \
    --file "$(dirname "$0")/Dockerfile" --tag "$image" "$context"
