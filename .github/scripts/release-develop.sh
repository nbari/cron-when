#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

github_repo() {
    local url=$1
    case "$url" in
        git@github.com:*) url=${url#git@github.com:} ;;
        ssh://git@github.com/*) url=${url#ssh://git@github.com/} ;;
        https://github.com/*) url=${url#https://github.com/} ;;
        *) echo "Expected a GitHub.com origin remote." >&2; return 1 ;;
    esac
    url=${url%/}
    url=${url%.git}
    [[ "$url" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ]] ||
        { echo "Expected a single GitHub owner and repository for origin." >&2; return 1; }
    printf '%s\n' "$url"
}
repo="$(github_repo "$(git remote get-url origin)")"
push_repo="$(github_repo "$(git remote get-url --push --all origin)")"
shopt -s nocasematch
[[ "$repo" == "$push_repo" ]] ||
    { echo "The origin fetch and push URLs must refer to the same GitHub repository." >&2; exit 1; }

git fetch --quiet origin refs/heads/develop
sha="$(git rev-parse FETCH_HEAD)"
bash .github/scripts/ci-ok-status.sh "$repo" "$sha"

run="$(gh api --hostname github.com -X POST "repos/$repo/actions/workflows/release.yml/dispatches" \
    -f ref=develop -f "inputs[expected_sha]=$sha" -F return_run_details=true)"
id="$(jq -er '.workflow_run_id | select(type == "number")' <<<"$run")" ||
    { echo "The dispatch returned no run ID. Inspect Actions for $sha before retrying." >&2; exit 1; }
echo "Release run for $sha: https://github.com/$repo/actions/runs/$id"
gh run watch "$id" --repo "github.com/$repo" --exit-status
bash .github/scripts/ci-ok-status.sh "$repo" "$sha"
git push origin "$sha:refs/heads/main"
