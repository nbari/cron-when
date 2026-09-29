# GitHub Workflows Template

This directory holds the GitHub Actions workflows, actions and policies for Rust
projects copied from this template. Jobs read the package name from `Cargo.toml`; the
values to adapt by hand are listed below and, for the release flow, in
[RELEASING.md](../RELEASING.md#adopting-this-flow-in-another-project).

## Files

- **workflows/build.yml** (Test & Build) - Runs on every branch push and pull request:
  tests, coverage, the security audit, the container integration test and the builds,
  ending with the aggregate **CI OK** check that branch protection requires
- **workflows/test.yml** - Formatting, clippy, feature checks, the MSRV check and tests
  on Linux, macOS and Windows (called by build.yml and release.yml)
- **workflows/coverage.yml** - Code coverage, uploaded to Codecov and Coveralls
- **workflows/containers.yml** - Container integration test of the static binary
- **workflows/security-audit.yml** - cargo-audit and cargo-deny, also daily
- **workflows/release.yml** (Release) - The build-once release workflow: a candidate
  run builds, packages and attests everything before any tag exists, and the tag's run
  publishes exactly those files and the crate (see [RELEASING.md](../RELEASING.md))
- **actions/release-is-latest** - Whether a tag is the highest promoted release, asked
  right before anything that follows the newest release acts
- **actions/rust-toolchain** - Installs a Rust toolchain with rustup, so no
  third-party toolchain action runs
- **actionlint.yaml** - Silences actionlint's unknown concurrency `queue` key only
- **dependabot.yml** - Weekly updates for crates, the pinned actions, the container
  image and the Dev Container features
- **SECURITY.md** - Vulnerability reporting and supported-version policy (replace its
  contact and response times with your own)

Outside this directory, the release flow also needs `scripts/release` and the release
recipes in `.justfile`.

## How to Use as Template

### 1. Copy the automation and development-container files

```bash
cp -r .github /path/to/new-project/
cp -r .devcontainer /path/to/new-project/
cp -r scripts /path/to/new-project/
cp .justfile deny.toml RELEASING.md /path/to/new-project/
cp .sops.yaml.example /path/to/new-project/
```

Update these project-specific values after copying:

- `cron-when` in `.devcontainer/devcontainer.json` and
  `initialize-secrets.sh`
- The Rust target and pinned Cargo tools in `.devcontainer/Dockerfile`
- The public age recipient and encrypted-file pattern in `.sops.yaml`
- Any container-engine access required by integration tests; do not enable
  privileged mode by default
- Whether `--security-opt label=disable` is needed by the target provider. It
  supports Fedora/Podman bind mounts without relabeling host paths, but can be
  removed where SELinux labeling is not involved.

The age private key is deliberately not part of the template. Contributors
provide it through a protected host file, a direct environment value, or a
1Password reference. The initializer stages it outside the repository and the
container mounts it read-only. See the root README for setup and rotation
guidance.

Dependabot tracks Cargo crates, GitHub Actions, the Docker base image, and Dev
Container Features. Pinned Cargo CLI versions inside the Dockerfile are not
updated automatically; review them periodically and validate a complete
container rebuild after changing them.

### 2. Required Cargo.toml Configuration

Ensure your `Cargo.toml` has these sections for RPM/DEB packaging:

```toml
[package.metadata.generate-rpm]
assets = [
    { source = "target/release/YOUR-BINARY-NAME", dest = "/usr/bin/YOUR-BINARY-NAME", mode = "0755" },
]

[package.metadata.deb]
assets = [
  ["target/release/YOUR-BINARY-NAME", "/usr/bin/YOUR-BINARY-NAME", "755"],
]
depends = ""
```

**Note:** Replace `YOUR-BINARY-NAME` with your binary's name yourself; nothing rewrites
these paths. The release workflow runs `cargo generate-rpm` and `cargo deb` with them.

#### Pure Rust TLS (No OpenSSL Required)

This template uses **rustls** for TLS, requiring no system dependencies:

```toml
# Example TLS dependencies (if your project needs TLS/HTTPS)
[features]
default = []
telemetry = ["dep:tonic", "dep:opentelemetry-otlp"]

[dependencies]
tonic = { version = "0.14", features = ["tls-webpki-roots"], optional = true }
opentelemetry-otlp = { version = "0.32", features = ["grpc-tonic", "tls-webpki-roots"], optional = true }
```

**Benefits:**
- No OpenSSL installation needed on any platform
- Static musl builds work out of the box
- Simplified cross-platform compilation (especially Windows)
- Same security guarantees as OpenSSL

#### Compile-Time Features and Runtime Configuration

Cargo features select capabilities while compiling. They do not behave like
runtime release toggles:

```bash
# Minimal binary
cargo build --release

# Different binary containing telemetry support
cargo build --release --features telemetry

# Runtime configuration for the telemetry-enabled binary
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 ./target/release/YOUR-BINARY-NAME
```

Changing the Cargo feature set requires rebuilding and redeploying the binary.
Changing an environment variable does not require rebuilding, but an
application that reads configuration only at startup must be restarted.

When adapting this template:

- Mark feature-specific dependencies `optional = true` and include them with
  `dep:dependency-name` in one user-facing feature.
- Prefer additive, independently useful features; avoid mutually exclusive
  feature combinations.
- Keep `default = []` when the optional capability is large or specialized.
  Add it to `default` only when most users should receive it.
- Test the default feature set and `--all-features` in CI.
- Decide and document which feature set published release artifacts contain.
  `build.yml` validates both configurations, while `release.yml` currently
  publishes the minimal default binary. Add `--features telemetry` to the
  release workflow if your project promises telemetry-enabled downloads.
- Use a runtime feature-management system—not Cargo features—when you need live
  rollouts, targeting, or a kill switch without rebuilding and restarting.

### 3. Secrets and one-time settings

**For coverage.yml:** `CODECOV_TOKEN`, a token from codecov.io (optional; coverage runs
and Coveralls uploads without it).

**For release.yml:** no secret. The crate is published with crates.io
[Trusted Publishing](https://crates.io/docs/trusted-publishing): add a trusted publisher
for your repository and the workflow file `release.yml` on the crate's settings page.
The rest of the one-time setup (branches, signing key, branch protection and the
release-tag rule) is in [RELEASING.md](../RELEASING.md#adopting-this-flow-in-another-project).

### 4. Binary Location

The workflows expect your binary to be in one of:
- `src/main.rs` (default binary)
- `src/bin/YOUR-PACKAGE-NAME.rs` (named binary)

The package name is auto-detected from `Cargo.toml`:
```toml
[package]
name = "your-package-name"  # ← This is used
```

## Workflow Behavior

### On Every Push or Pull Request

1. **test.yml** runs:
   - `cargo fmt --check`
   - `cargo clippy`
   - Minimal and all-feature checks
   - An MSRV check using the `rust-version` from `Cargo.toml`
   - Minimal and all-feature tests on Ubuntu, macOS, Windows

2. **build.yml** runs (if tests pass):
   - Builds both the minimal default configuration and `--features telemetry` for:
     - Linux (x86_64-unknown-linux-musl)
     - macOS (x86_64-apple-darwin)
     - Windows (x86_64-pc-windows-msvc)
   - Runs coverage and security audit in parallel

### Releases

Releases never start from a tag you push. `just deploy` stages the version bump on the
scratch `release` branch, starts a candidate run of **release.yml** there (every test,
build and package, kept as artifacts with a checksum manifest and provenance
attestations, nothing published), and only when that run and Test & Build pass does it
sign the `X.Y.Z` tag and move `develop` and `main`. The tag's run then publishes the
candidate's files and the crate; it builds nothing.

Any other tag, a `t-*` test tag for instance, only tests and builds. To try the whole
pipeline on a branch without a tag, run `just release-dry-run`. The complete
description, including recovery, is in [RELEASING.md](../RELEASING.md).

## Security Auditing

**security-audit.yml** runs automatically:
- **Daily at 00:00 UTC** (scheduled)
- As part of `build.yml` on every push and pull request (in parallel with coverage)
- Manually through GitHub Actions when an ad hoc audit is needed

It performs two types of checks:

### 1. cargo-audit
Checks dependencies against the RustSec Advisory Database for:
- Known security vulnerabilities
- Unmaintained crates
- Yanked crate versions

### 2. cargo-deny
Checks for:
- **Licenses** - Ensures all dependencies use approved licenses
- **Advisories** - Security vulnerabilities (similar to cargo-audit)
- **Bans** - Prevents use of specific crates or duplicate versions
- **Sources** - Ensures crates come from trusted registries

Configuration is in `deny.toml`. Common customizations:

```toml
[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-3-Clause",
    # Add your project's allowed licenses
]

[advisories]
ignore = [
    # Temporarily ignore specific advisories (with justification)
    # "RUSTSEC-2024-0001",
]
```

## Release Artifacts

Each release carries, for version `X.Y.Z`:

- `PACKAGE-X.Y.Z-x86_64-unknown-linux-musl.tar.gz` and
  `PACKAGE-X.Y.Z-aarch64-unknown-linux-musl.tar.gz` (static binaries)
- `PACKAGE-X.Y.Z-1.x86_64.rpm`, `PACKAGE-X.Y.Z-1.aarch64.rpm`,
  `PACKAGE_X.Y.Z-1_amd64.deb` and `PACKAGE_X.Y.Z-1_arm64.deb`
- `PACKAGE-X.Y.Z-x86_64-apple-darwin.tar.gz`
- `PACKAGE-X.Y.Z-x86_64-pc-windows-msvc.zip`
- `SHA256SUMS` for all of them, plus build-provenance attestations for every file and
  for the crate published to crates.io

## Customization

### Change Target Platforms

Edit the matrix in `build.yml` and `release.yml`. In `release.yml`, also update the
manifest job's `EXPECTED` inventory and its total file count, or the candidate run
refuses the new set of files:

```yaml
strategy:
  matrix:
    include:
      - build: linux
        os: ubuntu-latest
        target: x86_64-unknown-linux-musl

      # Add ARM support
      - build: linux-arm
        os: ubuntu-latest
        target: aarch64-unknown-linux-musl

      # Add Apple Silicon
      - build: macos-arm
        os: macos-latest
        target: aarch64-apple-darwin
```

### Skip Coverage or Security

Remove or comment out jobs in `build.yml`, and remove them from the `ci-ok` job's
`needs` as well; otherwise **CI OK**, which branch protection requires, can never pass:

```yaml
jobs:
  test:
    uses: ./.github/workflows/test.yml

  # coverage:
  #   uses: ./.github/workflows/coverage.yml
  #   secrets: inherit

  # security:
  #   uses: ./.github/workflows/security-audit.yml

  build:
    needs: test
```

### Customize Security Checks

Edit `deny.toml` to adjust:
- Allowed licenses
- Vulnerability severity levels
- Ignored advisories (with justification)

### Change Test Matrix

Edit `test.yml` to add more Rust versions or platforms:

```yaml
strategy:
  matrix:
    os:
      - ubuntu-latest
      - macOS-latest
      - windows-latest
    rust:
      - stable
      - beta  # Add beta testing
      - nightly  # Add nightly testing
```

## How It Works

All workflows use this step to get the package name:

```yaml
- name: Get package name from Cargo.toml
  id: package
  shell: bash
  run: |
    PACKAGE_NAME=$(awk -F '"' '/^name = / {print $2; exit}' Cargo.toml)
    echo "name=$PACKAGE_NAME" >> $GITHUB_OUTPUT
    echo "Package name: $PACKAGE_NAME"
```

Then reference it as: `${{ steps.package.outputs.name }}`

This works on all platforms (Linux, macOS, Windows) without installing additional tools.

## Troubleshooting

### Build fails on Linux with musl

The workflows install `musl-tools` automatically. If building locally:

```bash
# Ubuntu/Debian
sudo apt-get install musl-tools

# Install musl target
rustup target add x86_64-unknown-linux-musl

# Build static binary
cargo build --release --target x86_64-unknown-linux-musl
```

**Note:** No OpenSSL or special features needed. If you're using TLS, use rustls with `tls-webpki-roots` feature.

### Coverage fails

The coverage job is one of the gates **CI OK** requires. Its uploads never fail it:
`CODECOV_TOKEN` is optional (without it the Codecov upload is skipped) and the Coveralls
upload uses `fail-on-error: false`, so a red coverage job means the tests themselves
failed under instrumentation. If you do not want coverage at all, remove the `coverage`
job from `build.yml` and from the `ci-ok` job's `needs`.

### A release does not publish

Run `just release-status`: it shows the staged candidate and its runs, and the tag run
of `develop`'s version once that version is tagged.
The tag run's first job, the guard, explains any refusal in its log (an unsigned or
unverified tag, a commit not on `main`, a version mismatch, no green Test & Build run,
or no successful candidate run). A failed crates.io upload usually means Trusted
Publishing is not configured for `release.yml` on the crate's settings page; fix it
and re-run the failed job. [RELEASING.md](../RELEASING.md#when-something-fails) lists
what to do for every failure.

## License

These workflows are part of your project template and can be freely copied and modified.
