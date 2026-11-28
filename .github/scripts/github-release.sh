#!/usr/bin/env bash
set -euo pipefail

releases="repos/$GITHUB_REPOSITORY/releases"
marker="<!-- Release run $GITHUB_RUN_ID at $GITHUB_SHA -->"

fail() { echo "::error::$*"; exit 1; }

find_release() {
    gh api --paginate --slurp "$releases?per_page=100" |
        jq -c --arg version "$VERSION" '
            [.[][] | select(.tag_name == $version)] |
            if length <= 1 then .[0] else error("Multiple releases for this version") end
        '
}

owned_draft() {
    jq -e --arg sha "$GITHUB_SHA" --arg marker "$marker" '
        .draft == true and .target_commitish == $sha and (.body | contains($marker))
    ' <<<"$1" >/dev/null
}

verify_draft_assets() {
    local expected actual
    expected="$(cd dist && sha256sum -- * | awk '{print $2, "sha256:" $1}' | LC_ALL=C sort)"
    actual="$(jq -er '
        .assets[] |
        if .state == "uploaded" and (.digest | type == "string")
        then [.name, .digest] | join(" ")
        else error("Incomplete draft asset") end
    ' <<<"$1" | LC_ALL=C sort)"
    [[ "$actual" == "$expected" ]] || fail "The uploaded draft assets differ from the verified build."
}

local_sums="$(cd dist && sha256sum -- *)"
crate_sum="$(sha256sum -- "dist/$PACKAGE-$VERSION.crate" | cut -d ' ' -f 1)"
existing="$(find_release)"
if [[ "$existing" == null ]] || jq -e '.draft == true' <<<"$existing" >/dev/null; then
    bash .github/scripts/check-release-version.sh "$VERSION"
    if [[ "$existing" != null ]]; then
        owned_draft "$existing" ||
            fail "An unrelated draft already exists for $VERSION. Inspect $(jq -r .html_url <<<"$existing")."
        gh api -X DELETE "$releases/$(jq -er .id <<<"$existing")"
    fi

    printf '\n%s\n' "$marker" >>"$RUNNER_TEMP/notes.md"
    gh release create "$VERSION" dist/* --draft --target "$GITHUB_SHA" \
        --title "$VERSION" --notes-file "$RUNNER_TEMP/notes.md"
    draft="$(find_release)"
    owned_draft "$draft" || fail "The draft changed during upload."
    verify_draft_assets "$draft"

    held="$(bash .github/scripts/crates-io-checksum.sh "$PACKAGE" "$VERSION")"
    [[ "$held" == absent || "$held" == "$crate_sum" ]] ||
        fail "crates.io already holds $PACKAGE $VERSION with different bytes. Reconcile the package or choose another allowed version."
    bash .github/scripts/check-release-version.sh "$VERSION"
    gh api -X PATCH "$releases/$(jq -er .id <<<"$draft")" -F draft=false -f make_latest=true >/dev/null
else
    echo "Verifying the already published release $VERSION."
fi

tag_sha="$(gh api "repos/$GITHUB_REPOSITORY/git/ref/tags/$VERSION" --jq '.object | select(.type == "commit").sha')"
[[ "$tag_sha" == "$GITHUB_SHA" ]] || fail "Tag $VERSION points at $tag_sha instead of $GITHUB_SHA."
published="$(mktemp -d)"
trap 'rm -rf -- "$published"' EXIT
gh release download "$VERSION" --dir "$published"
published_sums="$(cd "$published" && sha256sum -- *)"
[[ "$published_sums" == "$local_sums" ]] || fail "The published assets differ from the verified build."
echo "Verified release $VERSION at $GITHUB_SHA."
echo "crate_checksum=$crate_sum" >>"$GITHUB_OUTPUT"
