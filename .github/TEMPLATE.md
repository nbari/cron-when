# GitHub Workflows Template

This directory contains reusable GitHub Actions workflows for Rust projects. The workflows automatically detect the package name from `Cargo.toml`, making them easy to copy to other projects.

## Files

- **test.yml** - Run tests, formatting, and clippy checks
- **build.yml** - Build binaries for multiple platforms (depends on tests)
- **coverage.yml** - Generate code coverage reports
- **security-audit.yml** - Security auditing (cargo-audit and cargo-deny)
- **SECURITY.md** - Vulnerability reporting and supported-version policy
- **release.yml** - Create releases and publish to crates.io

## How to Use as Template

### 1. Copy the automation and development-container files

```bash
cp -r .github /path/to/new-project/
cp -r .devcontainer /path/to/new-project/
cp deny.toml /path/to/new-project/
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

**Note:** The workflows will automatically replace `YOUR-BINARY-NAME` with the actual package name from Cargo.toml.

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

### 3. GitHub Secrets Required

Set these in your repository settings (Settings → Secrets and variables → Actions):

**For coverage.yml:**
- `CODECOV_TOKEN` - Token from codecov.io (optional, coverage will run without it)

**For release.yml:**
- `CRATES_TOKEN` - Token from crates.io for publishing

To get a crates.io token:
```bash
cargo login
# Find your token at: https://crates.io/settings/tokens
```

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

### On Tag Push

**release.yml** runs when you push a tag:

```bash
# Full release (creates GitHub release + publishes to crates.io)
git tag v0.1.0
git push origin v0.1.0

# Test release (builds everything but skips release/publish)
git tag t0.1.0
git push origin t0.1.0
```

Tags starting with `t` will:
- ✓ Run all tests
- ✓ Build all binaries (Linux with RPM/DEB, macOS, Windows)
- ✗ Skip creating GitHub release
- ✗ Skip publishing to crates.io

This lets you test the full release workflow without publishing.

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

When a non-test tag is pushed, the release includes:

### Linux
- `PACKAGE-VERSION-x86_64-unknown-linux-musl.tar.gz` (static binary)
- `PACKAGE-VERSION.rpm` (RPM package)
- `PACKAGE-VERSION.deb` (Debian package)

### macOS
- `PACKAGE-VERSION-x86_64-apple-darwin.tar.gz`

### Windows
- `PACKAGE-VERSION-x86_64-pc-windows-msvc.zip`

## Customization

### Change Target Platforms

Edit the matrix in `build.yml` and `release.yml`:

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

Remove or comment out jobs in `build.yml`:

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

Coverage is optional. If you don't need it:
1. Remove the `CODECOV_TOKEN` secret requirement
2. Or remove the coverage job from `build.yml`

### Release doesn't publish to crates.io

Check:
1. `CRATES_TOKEN` secret is set
2. Tag doesn't start with `t`
3. Version in `Cargo.toml` matches tag (without `v` prefix)

## License

These workflows are part of your project template and can be freely copied and modified.
