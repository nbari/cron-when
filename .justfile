# Justfile for Rust CLI Development
# This file is template-ready - works with any Rust project
# Just update project-specific references (URLs, container names, etc.)

default:
  @just --list

# Fast local test suite (no containers or cross-compilation)
test: clippy fmt-check unit-test
  @echo "✅ Local tests passed!"

# Full test suite, including the MUSL/container integration test
full-test: test integration-test
  @echo "✅ Full test suite passed!"

# Unit tests
unit-test:
  @echo "🧪 Running unit tests..."
  cargo test -- --nocapture

# Run tests with coverage
coverage:
  @echo "📊 Running tests with coverage..."
  cargo llvm-cov --all-features --workspace

# Linting
clippy:
  @echo "🔍 Running clippy..."
  cargo clippy --all-targets --all-features

# Formatting
fmt:
  @echo "🎨 Formatting code..."
  cargo fmt --all

# Verify formatting without modifying source files
fmt-check:
  @echo "🎨 Checking formatting..."
  cargo fmt --all -- --check

# Run benchmarks
bench:
  @echo "⚡ Running benchmarks..."
  cargo bench

# Build release version
build:
  @echo "🔨 Building release..."
  cargo build --release

# Build with musl for static linking
build-musl:
  @echo "🔨 Building with musl..."
  cargo build --release --target x86_64-unknown-linux-musl

# Update dependencies
update:
  @echo "⬆️  Updating dependencies..."
  cargo update

# Clean build artifacts
clean:
  @echo "🧹 Cleaning build artifacts..."
  cargo clean

# Get current version
version:
    @cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'

# Guard: refuse a dirty working tree (the bump commits only its own version files)
check-clean:
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ -n $(git status --short) ]]; then
        echo "❌ Working directory is not clean. Commit or stash your changes first."
        git status --short
        exit 1
    fi
    echo "✅ Working directory is clean"

# Guard: refuse while cargo watch runs (a watcher races builds and target cleanup)
check-no-watch:
    #!/usr/bin/env bash
    set -euo pipefail
    if pgrep -f '[c]argo watch|[c]argo-watch' >/dev/null; then
        echo "❌ cargo watch is running. Stop 'just watch' before bumping."
        exit 1
    fi
    echo "✅ No cargo watcher is running"

# Guard: refuse the fast-forward-only branches main and develop (bump from a working branch)
check-branch:
    #!/usr/bin/env bash
    set -euo pipefail
    current_branch="$(git branch --show-current)"
    if [[ "$current_branch" == "main" || "$current_branch" == "develop" ]]; then
        echo "❌ Branch '$current_branch' is fast-forward-only; bump from a working branch."
        exit 1
    fi
    echo "✅ Branch '$current_branch' is a working branch"

# Private: bump the version, refresh dependencies, run tests, and commit (no push)
_bump bump_kind: check-clean check-no-watch check-branch
    #!/usr/bin/env bash
    set -euo pipefail

    bump_kind="{{bump_kind}}"

    cleanup() {
        status=$?
        if [ $status -ne 0 ]; then
            echo "↩️  Restoring version files after failure..."
            if ! git restore --staged --worktree --source=HEAD -- Cargo.toml Cargo.lock; then
                echo "⚠️ Failed to restore Cargo.toml Cargo.lock from HEAD; fix them manually."
            fi
        fi
        exit $status
    }
    trap cleanup EXIT

    previous_version=$(just version)
    echo "ℹ️  Current version: ${previous_version}"

    echo "🔧 Bumping ${bump_kind} version..."
    cargo set-version --bump "${bump_kind}"

    new_version=$(just version)
    echo "📝 New version: ${new_version}"

    validate_bump() {
        local previous=$1 bump=$2 current=$3
        IFS=. read -r prev_major prev_minor prev_patch <<<"${previous}"
        IFS=. read -r new_major new_minor new_patch <<<"${current}"

        case "${bump}" in
            patch)
                (( new_major == prev_major && new_minor == prev_minor && new_patch == prev_patch + 1 )) || { echo "❌ Expected patch bump from ${previous}, got ${current}"; exit 1; }
                ;;
            minor)
                (( new_major == prev_major && new_minor == prev_minor + 1 && new_patch == 0 )) || { echo "❌ Expected minor bump from ${previous}, got ${current}"; exit 1; }
                ;;
            major)
                (( new_major == prev_major + 1 && new_minor == 0 && new_patch == 0 )) || { echo "❌ Expected major bump from ${previous}, got ${current}"; exit 1; }
                ;;
        esac
    }

    validate_bump "${previous_version}" "${bump_kind}" "${new_version}"

    echo "🔍 Verifying tag does not exist for ${new_version}..."
    git fetch --tags --quiet
    if git rev-parse -q --verify "refs/tags/${new_version}" >/dev/null 2>&1; then
        echo "❌ Tag ${new_version} already exists!"
        exit 1
    fi

    echo "🔄 Updating dependencies..."
    cargo update

    echo "🧪 Running tests with new version (via just test)..."
    just test

    git add Cargo.toml Cargo.lock
    git commit -m "bump version to ${new_version}"

    echo "✅ Version bumped to ${new_version} (commit $(git rev-parse --short HEAD))."
    echo "Next steps: fast-forward develop to this SHA when Test & Build is green, then run 'just release'."

# Bump version and commit (patch level)
bump:
  @just _bump patch

# Bump minor version
bump-minor:
  @just _bump minor

# Bump major version
bump-major:
  @just _bump major

# Releases (see RELEASING.md): the version bump is done with `just bump` (or
# `bump-minor`/`bump-major`) as the last commit of the work, and the Release workflow
# builds and publishes that exact commit once CI passed.
# `main` is moved here, after the run, because a protected branch refuses an update while
# a `CI OK` check of the same commit is still pending (develop's own Test & Build).

# Release develop: start the release workflow, follow it, then fast-forward main
release:
    #!/usr/bin/env bash
    set -euo pipefail
    git fetch --quiet origin develop
    sha="$(git rev-parse origin/develop)"
    last="$(gh run list --workflow release.yml --branch develop --event workflow_dispatch --limit 1 --json databaseId --jq '.[0].databaseId // 0')"
    gh workflow run release.yml --ref develop
    # The dispatched run is the first new one for this commit; never just "the latest".
    run=""
    for _ in $(seq 30); do
        run="$(gh run list --workflow release.yml --branch develop --event workflow_dispatch --limit 10 --json databaseId,headSha --jq "[.[] | select(.databaseId > ${last} and .headSha == \"${sha}\")][0].databaseId // empty")"
        [[ -z "${run}" ]] || break
        sleep 2
    done
    [[ -n "${run}" ]] || { echo "The dispatched release run did not appear." >&2; exit 1; }
    gh run watch "${run}" --exit-status
    # develop's own Test & Build must have finished, and passed, before main can move.
    ci=""
    for _ in $(seq 30); do
        ci="$(gh run list --workflow build.yml --branch develop --commit "${sha}" --limit 1 --json databaseId --jq '.[0].databaseId // empty')"
        [[ -z "${ci}" ]] || break
        sleep 2
    done
    [[ -n "${ci}" ]] || { echo "No Test & Build run of develop for ${sha}; push it to main yourself when CI OK passed." >&2; exit 1; }
    echo "Waiting for develop's Test & Build (run ${ci}) before moving main..."
    gh run watch "${ci}" --exit-status > /dev/null || { echo "Test & Build failed on ${sha}; re-run its failed jobs, then: git push origin ${sha}:refs/heads/main" >&2; exit 1; }
    git push origin "${sha}:refs/heads/main"

# Check for security vulnerabilities
audit:
  @echo "🔒 Checking for security vulnerabilities..."
  cargo audit

# Check dependency licenses
deny:
  @echo "📜 Checking dependency licenses..."
  cargo deny --all-features check

# Full CI check (what runs in CI)
ci: full-test audit deny
  @echo "✅ All CI checks passed!"

# Build RPM package
build-rpm: build
  @echo "📦 Building RPM package..."
  cargo generate-rpm

# Build DEB package
build-deb: build
  @echo "📦 Building DEB package..."
  cargo deb

# Build all packages
build-packages: build-rpm build-deb
  @echo "✅ All packages built!"

# Show documentation
doc:
  @echo "📚 Building and opening documentation..."
  cargo doc --open --no-deps

# Check outdated dependencies
outdated:
  @echo "📅 Checking for outdated dependencies..."
  cargo outdated --root-deps-only

# Expand macros for debugging
expand:
  @echo "🔍 Expanding macros..."
  cargo expand

# Run example with sample crontab
example:
  @echo "📖 Running example with sample.crontab..."
  cargo run -- -f sample.crontab

# Integration tests - test with real crontab in container
integration-test: build-musl
  @echo "🐳 Running integration tests..."
  @if ! podman ps | grep -q cron-when-test; then \
    echo "📦 Starting test container..."; \
    podman run -d --rm --name cron-when-test alpine:latest sh -c "while true; do sleep 3600; done" > /dev/null 2>&1; \
    sleep 2; \
    echo "📝 Installing cron..."; \
    podman exec cron-when-test apk add --no-cache dcron > /dev/null 2>&1; \
    echo "✍️  Creating test crontab..."; \
    podman exec cron-when-test sh -c 'echo "# Backup every day at 2 AM" > /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "0 2 * * * /usr/local/bin/backup.sh" >> /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "" >> /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "# Clean logs every hour" >> /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "0 * * * * /usr/local/bin/cleanup.sh" >> /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "" >> /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "# Environment variables" >> /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "SHELL=/bin/sh" >> /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "PATH=/usr/local/bin:/usr/bin:/bin" >> /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "MAILTO=admin@example.com" >> /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "" >> /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "# Monitor every 5 minutes" >> /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "*/5 * * * * /usr/local/bin/monitor.sh" >> /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "" >> /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "# Weekly report on Monday at 9 AM" >> /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "0 9 * * 1 /usr/local/bin/weekly-report.sh" >> /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "" >> /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "# Midday checks at 12, 15, and 18 (range with step)" >> /tmp/test.cron'; \
    podman exec cron-when-test sh -c 'echo "0 12-18/3 * * * /usr/local/bin/midday-check.sh" >> /tmp/test.cron'; \
    podman exec cron-when-test crontab /tmp/test.cron; \
    echo "✅ Container ready"; \
  else \
    echo "✅ Using existing container"; \
  fi
  @echo "📦 Copying binary to container..."
  @podman cp target/x86_64-unknown-linux-musl/release/cron-when cron-when-test:/usr/local/bin/ 2>&1 | grep -v "Error: destination is a directory" || true
  @echo "🧪 Testing and validating crontab -l parsing..."
  @./scripts/validate-integration-test.sh
  @echo "✅ Integration tests passed!"

# Clean up test container
clean-test:
  @echo "🧹 Cleaning up test container..."
  @podman stop cron-when-test 2>/dev/null || true
  @echo "✅ Cleanup complete"

jaeger:
  podman run --rm -d --name jaeger \
    -e COLLECTOR_OTLP_ENABLED=true \
    -p 16686:16686 \
    -p 4317:4317 \
    -p 4318:4318 \
    jaegertracing/all-in-one:latest

stop-containers:
  @podman stop jaeger 2>/dev/null || true
  @just clean-test
