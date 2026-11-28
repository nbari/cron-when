#!/usr/bin/env bash
# Build and verify the index before the workflow assigns the release version tag.
set -euo pipefail
package=${PACKAGE:?Expected a package name}
version=${VERSION:?Expected a release version}
image="ghcr.io/${GITHUB_REPOSITORY,,}"

lookup_digest() {
    local ref=$1 result
    if result="$(docker buildx imagetools inspect "$ref" --format '{{json .Manifest}}' 2>&1)"; then
        jq -er '.digest | select(test("^sha256:[a-f0-9]{64}$"))' <<<"$result"
    elif [[ "$result" == *": not found" || "$result" == *"manifest unknown"* ]]; then
        return 0
    else
        printf '%s\n' "$result" >&2
        return 1
    fi
}

context="$(mktemp -d)"
trap 'rm -rf -- "$context"' EXIT
for pair in amd64:x86_64 arm64:aarch64; do
    arch=${pair%%:*}
    archive="$package-$version-${pair#*:}-unknown-linux-musl"
    dir="$context/linux/$arch"
    mkdir -p "$dir"
    tar -xOzf "dist/$archive.tar.gz" "$archive/$package" >"$dir/cron-when"
    chmod 0555 "$dir/cron-when"
done

candidate="$image:sha-$GITHUB_SHA"
# An initial read can be denied before GHCR creates the package. Only the
# commit-scoped staging tag can be written before the version lookup succeeds.
if ! staged="$(lookup_digest "$candidate")" || [[ -z "$staged" ]]; then
    export SOURCE_DATE_EPOCH
    SOURCE_DATE_EPOCH="$(git show -s --format=%ct "$GITHUB_SHA")"
    docker buildx create --driver docker-container --use
    docker buildx build --platform linux/amd64,linux/arm64 \
        --file .github/container/Dockerfile --provenance=false --sbom=false \
        --label "org.opencontainers.image.source=https://github.com/$GITHUB_REPOSITORY" \
        --label "org.opencontainers.image.version=$version" \
        --label "org.opencontainers.image.revision=$GITHUB_SHA" \
        --annotation "index:org.opencontainers.image.source=https://github.com/$GITHUB_REPOSITORY" \
        --tag "$candidate" --output type=registry,oci-mediatypes=true,rewrite-timestamp=true "$context"
    staged="$(lookup_digest "$candidate")"
fi
digest="$(lookup_digest "$image:$version")"
publish=false
if [[ -z "$digest" ]]; then
    digest="$staged"
    publish=true
fi
[[ -n "$digest" ]] || { echo "The image digest is missing." >&2; exit 1; }

ref="$image@$digest"
index="$(docker buildx imagetools inspect --raw "$ref")"
jq -e '
    .mediaType == "application/vnd.oci.image.index.v1+json" and
    ([.manifests[].platform | "\(.os)/\(.architecture)"] | sort == ["linux/amd64", "linux/arm64"])
' <<<"$index" >/dev/null
for arch in amd64 arm64; do
    child="$(jq -er --arg arch "$arch" '.manifests[] | select(.platform.architecture == $arch) | .digest' <<<"$index")"
    variant="$image@$child"
    docker pull --quiet --platform "linux/$arch" "$variant"
    docker image inspect --format '{{json .Config}}' "$variant" | jq -e --arg sha "$GITHUB_SHA" '
        .User == "65532:65532" and .Entrypoint == ["/cron-when"] and
        .Labels["org.opencontainers.image.revision"] == $sha
    ' >/dev/null
    container="$(docker create --platform "linux/$arch" "$variant")"
    if ! docker cp "$container:/cron-when" "$context/$arch"; then
        docker rm "$container" >/dev/null
        exit 1
    fi
    docker rm "$container" >/dev/null
    cmp "$context/$arch" "$context/linux/$arch/cron-when"
    if [[ "$arch" == amd64 ]]; then
        bash .github/container/smoke-test.sh "$variant" "$version"
    fi
done
printf 'image=%s\ndigest=%s\npublish=%s\n' "$image" "$digest" "$publish" >>"$GITHUB_OUTPUT"
