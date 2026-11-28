#!/usr/bin/env bash
# Require CI OK from the Test & Build push to develop at the requested SHA.
set -euo pipefail

repo=${1:?Usage: ci-ok-status.sh OWNER/REPO SHA}
sha=${2:?Usage: ci-ok-status.sh OWNER/REPO SHA}
[[ "$sha" =~ ^[0-9a-f]{40}$ ]] || { echo "Expected a full commit SHA." >&2; exit 1; }
actions="repos/$repo/actions"

run="$(gh api --hostname github.com -X GET "$actions/workflows/build.yml/runs" \
    -f branch=develop -f event=push -f "head_sha=$sha" -F per_page=1 |
    jq -c --arg sha "$sha" '
        .workflow_runs[0] |
        select(.head_sha == $sha and .head_branch == "develop" and .event == "push")
    ')"
[[ -n "$run" && "$run" != null ]] || {
    echo "No Test & Build push on develop for $sha. Check https://github.com/$repo/actions/workflows/build.yml" >&2
    exit 1
}
read -r id attempt status url < <(jq -r '[.id, .run_attempt, .status, .html_url] | @tsv' <<<"$run")
[[ "$id" =~ ^[0-9]+$ && "$attempt" =~ ^[1-9][0-9]*$ && -n "$status" && -n "$url" ]] ||
    { echo "Incomplete Test & Build metadata for $sha." >&2; exit 1; }

while (( attempt > 0 )); do
    jobs="$(gh api --hostname github.com --paginate --slurp "$actions/runs/$id/attempts/$attempt/jobs?per_page=100" |
        jq -ce '[.[].jobs[]]')"
    verdict="$(jq -r '
        [.[] | select(.name == "CI OK")] |
        if length == 0 then "missing"
        elif length != 1 then "ambiguous"
        elif .[0].status != "completed" then "pending"
        else .[0].conclusion // "missing" end
    ' <<<"$jobs")"
    if [[ "$verdict" == success ]]; then
        echo "CI OK passed for $sha on develop ($url)."
        exit 0
    fi
    # Only a completed attempt made up of coverage jobs can inherit an older gate.
    if [[ "$verdict" == missing && "$status" == completed ]] &&
        jq -e 'length > 0 and all(.[]; .name == "coverage / Coverage" and .status == "completed")' <<<"$jobs" >/dev/null; then
        attempt=$((attempt - 1))
        continue
    fi
    if [[ "$verdict" == missing && "$status" != completed ]]; then
        verdict=pending
    fi
    echo "CI OK is $verdict for $sha on develop ($url). Wait for CI or rerun the failed jobs." >&2
    exit 1
done
echo "No CI OK job in Test & Build for $sha ($url)." >&2
exit 1
