# Bump the patch version and commit the tested candidate
bump: (_release-bump "patch")

# Bump the minor version and commit the tested candidate
bump-minor: (_release-bump "minor")

# Bump the major version and commit the tested candidate
bump-major: (_release-bump "major")

_release-bump kind:
    #!/usr/bin/env bash
    set -euo pipefail

    kind={{quote(kind)}}
    fail() { printf '%s\n' "$*" >&2; exit 1; }

    branch="$(git symbolic-ref --quiet --short HEAD)" || fail "Switch to a working branch before bumping."
    bump_branch="${RELEASE_BUMP_BRANCH:-}"
    if [[ "$branch" == main || ( -z "$bump_branch" && "$branch" == develop ) ||
          ( -n "$bump_branch" && "$branch" != "$bump_branch" ) ]]; then
        fail "Bump from ${bump_branch:-a working branch such as sandbox} (current branch: $branch)."
    fi
    start="$(git rev-parse HEAD)"
    status="$(git status --porcelain --untracked-files=normal)"
    [[ -z "$status" ]] || fail "Commit or stash your changes before bumping."
    if pgrep -f '[c]argo watch|[c]argo-watch' >/dev/null; then
        fail "Stop cargo watch before bumping."
    fi

    repo_root="$(git rev-parse --show-toplevel)"
    metadata="$(cargo metadata --no-deps --format-version 1)"
    workspace="$(jq -er '.workspace_root' <<<"$metadata")"
    manifest="$workspace/Cargo.toml"
    version_files=("$manifest" "$workspace/Cargo.lock")
    member_manifests="$(jq -r '.packages[].manifest_path' <<<"$metadata")"
    while IFS= read -r member; do
        [[ "$member" == "$manifest" ]] || version_files+=("$member")
    done <<<"$member_manifests"
    while IFS= read -r extra; do
        if [[ -n "$extra" ]]; then
            version_files+=("$workspace/$extra")
        fi
    done <<<"${RELEASE_BUMP_FILES:-}"

    exclusions=()
    for file in "${version_files[@]}"; do
        relative="${file#"$repo_root"/}"
        [[ "$relative" != "$file" && -f "$file" && ! -L "$file" ]] ||
            fail "Expected a regular version file inside the repository: $file"
        IFS= read -r -d '' tracked < <(git -C "$repo_root" --literal-pathspecs ls-files -z --error-unmatch -- "$relative") ||
            fail "Version file must be tracked: $file"
        [[ "$tracked" == "$relative" ]] || fail "Use a normalized path for the version file: $file"
        exclusions+=(":(exclude,literal,top)$relative")
    done

    same_checkout() {
        [[ "$(git symbolic-ref --quiet --short HEAD)" == "$branch" &&
           "$(git rev-parse HEAD)" == "$start" ]]
    }

    cleanup() {
        local status=$?
        if (( status != 0 )); then
            if same_checkout; then
                git --literal-pathspecs restore --staged --worktree --source="$start" -- "${version_files[@]}" ||
                    printf 'Could not restore version files from %s. Inspect git status.\n' "$start" >&2
            else
                printf 'The checkout changed during the bump. Inspect git status for remaining changes.\n' >&2
            fi
        fi
    }
    trap cleanup EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM

    shared_version() {
        jq -er '
            [.packages[].version] | unique |
            if length == 1 and (.[0] | test("^(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)$"))
            then .[0] else error("Expected one stable X.Y.Z version across the workspace") end
        '
    }

    previous="$(shared_version <<<"$metadata")"
    echo "Checking the current version ($previous)..."
    just --justfile {{quote(justfile())}} test
    same_checkout || fail "The checkout changed during the baseline checks."
    status="$(git status --porcelain --untracked-files=normal)"
    [[ -z "$status" ]] || fail "The baseline checks changed the working tree. Inspect git status."

    cargo set-version --manifest-path "$manifest" --workspace --bump "$kind"
    metadata="$(cargo metadata --manifest-path "$manifest" --no-deps --format-version 1)"
    version="$(shared_version <<<"$metadata")"
    [[ "$version" != "$previous" ]] || fail "Cargo did not change the workspace version."

    git fetch --quiet --tags origin
    latest="$(git tag --list | awk '/^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/' |
        LC_ALL=C sort -t. -k1,1n -k2,2n -k3,3n | tail -n 1)"
    if [[ -n "$latest" ]]; then
        IFS=. read -r major minor patch <<<"$latest"
        next_patch="$major.$minor.$((patch + 1))"
        next_minor="$major.$((minor + 1)).0"
        next_major="$((major + 1)).0.0"
        if [[ "$version" != "$next_patch" && "$version" != "$next_minor" && "$version" != "$next_major" ]]; then
            fail "After $latest, the next release must be $next_patch, $next_minor, or $next_major (got $version)."
        fi
    fi

    cargo update --manifest-path "$manifest"
    if [[ -n "${RELEASE_BUMP_FILES:-}" ]]; then
        just --justfile {{quote(justfile())}} _release-bump-metadata "$version"
    fi

    echo "Checking the release candidate ($version)..."
    just --justfile {{quote(justfile())}} test
    same_checkout || fail "The checkout changed while preparing the bump."
    metadata="$(cargo metadata --manifest-path "$manifest" --no-deps --format-version 1)"
    [[ "$(shared_version <<<"$metadata")" == "$version" ]] ||
        fail "The candidate checks or metadata recipe changed the release version."
    status="$(git -C "$repo_root" status --porcelain --untracked-files=normal -- . "${exclusions[@]}")"
    [[ -z "$status" ]] || fail "The bump changed undeclared files. Inspect git status."
    git --literal-pathspecs commit -S --only -m "Bump version to $version" -- "${version_files[@]}"
    printf 'Prepared %s at %s. Push the commit and wait for CI OK.\n' \
        "$version" "$(git rev-parse HEAD)"
