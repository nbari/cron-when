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

# Check if working directory is clean
check-clean:
    #!/usr/bin/env bash
    if [[ -n $(git status --porcelain) ]]; then
        echo "❌ Working directory is not clean. Commit or stash your changes first."
        git status --short
        exit 1
    fi
    echo "✅ Working directory is clean"

# Check if on develop branch
check-develop:
    #!/usr/bin/env bash
    current_branch=$(git branch --show-current)
    if [[ "$current_branch" != "develop" ]]; then
        echo "❌ Not on develop branch (currently on: $current_branch)"
        echo "Switch to develop branch first: git checkout develop"
        exit 1
    fi
    echo "✅ On develop branch"

# Releases stage the bump on the `release` branch, let CI test that exact commit, then
# move develop, main and the tag together; see scripts/release. Every deploy recipe is
# idempotent: rerunning it resumes the staged candidate or says nothing is left to do.

# Deploy: stage a patch bump, wait for CI on it, then release it
deploy:
    @scripts/release deploy patch

# Deploy with minor version bump
deploy-minor:
    @scripts/release deploy minor

# Deploy with major version bump
deploy-major:
    @scripts/release deploy major

# Release develop's version as is when it has no tag yet (no new bump)
deploy-current:
    @scripts/release deploy current

# Publish an existing release tag again if its own run cannot (recovery run on main)
release-republish version:
    @scripts/release republish {{version}}

# Show where a release stands: develop, main, the staged candidate and its CI run
release-status:
    @scripts/release status

# Check everything a release needs; changes nothing apart from fetching
release-preflight:
    @scripts/release preflight

# Apply the branch protection the release flow relies on (main requires "CI OK")
protect-branches:
    @scripts/release protect

# Create & push a test tag like t-YYYYMMDD-HHMMSS (skips publish/release in CI)
# Usage:
#   just t-deploy
#   just t-deploy "optional tag message"
t-deploy message="CI test": check-develop check-clean full-test
    #!/usr/bin/env bash
    set -euo pipefail

    TAG_MESSAGE="{{message}}"
    ts="$(date -u +%Y%m%d-%H%M%S)"
    tag="t-${ts}"

    echo "🏷️  Creating signed test tag: ${tag}"
    git fetch --tags --quiet

    if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
        echo "❌ Tag ${tag} already exists. Aborting." >&2
        exit 1
    fi

    git tag -s "${tag}" -m "${TAG_MESSAGE}"
    git push origin "${tag}"

    echo "✅ Pushed ${tag}"
    echo "🧹 To remove it:"
    echo "   git push origin :refs/tags/${tag} && git tag -d ${tag}"

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
