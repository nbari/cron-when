# cron-when

[![Test & Build](https://github.com/nbari/cron-when/workflows/Test%20%26%20Build/badge.svg)](https://github.com/nbari/cron-when/actions)
[![codecov](https://codecov.io/gh/nbari/cron-when/branch/main/graph/badge.svg)](https://codecov.io/gh/nbari/cron-when)
[![Crates.io](https://img.shields.io/crates/v/cron-when.svg)](https://crates.io/crates/cron-when)
[![Downloads](https://img.shields.io/crates/d/cron-when.svg)](https://crates.io/crates/cron-when)
[![Documentation](https://docs.rs/cron-when/badge.svg)](https://docs.rs/cron-when)
[![License](https://img.shields.io/crates/l/cron-when.svg)](https://github.com/nbari/cron-when/blob/main/LICENSE)


A CLI cron expression parser that shows the next execution time and duration until then.

## Educational Template

**This project is intentionally over-engineered to serve as a learning template:**

- Demonstrates compile-time feature gating plus runtime configuration
- Shows how to integrate distributed tracing in Rust CLIs
- Exhibits modular CLI architecture with separation of concerns
- Uses pure Rust TLS implementation (rustls + webpki-roots, no OpenSSL)
- Includes comprehensive testing (unit + container integration tests)
- Applies strict clippy lints for code quality and safety
- Documents tradeoffs and architectural decisions

### Key Technical Decisions

- **TLS Implementation**: Pure Rust using `rustls` with `webpki-roots`
  - No system OpenSSL dependency required
  - Simplified cross-platform builds (especially Windows)
  - Same security guarantees, fully portable

- **OpenTelemetry Integration**: Compile-time optional through the `telemetry` feature
  - The default build omits the OTLP/gRPC/TLS dependency stack
  - Enabling the Cargo feature requires a rebuild; it is not a live feature toggle
  - When compiled in, `OTEL_EXPORTER_OTLP_ENDPOINT` activates export at process startup
  - Multi-backend support (Jaeger, Honeycomb, Grafana, AWS X-Ray, etc.)
  - Uses gRPC over TLS with rustls for secure trace export

- **Code Quality**: Strict clippy lints enforced
  - All `pedantic` lints enabled
  - Safety lints: no `unwrap()`, `expect()`, `panic!()`, or unsafe indexing in production code
  - Comprehensive error handling and documentation

See [`CLI_ARCHITECTURE.md`](CLI_ARCHITECTURE.md) for detailed discussion of design decisions.

### 🚀 Using This as a Template

This project is designed to be copied and adapted for your own Rust CLIs:

```bash
# 1. Copy the project
git clone https://github.com/nbari/cron-when my-new-cli
cd my-new-cli && rm -rf .git && git init

# 2. Update Cargo.toml
# - name = "my-new-cli"
# - authors, description, repository

# 3. What to keep vs replace:
# ✅ KEEP: src/cli/ (entire architecture)
# ✅ KEEP: .github/workflows/ (auto-detects package name)
# ✅ KEEP: Strict clippy lints, deny.toml
# 🔄 REPLACE: src/crontab.rs, src/output.rs (your domain logic)
# 🔄 UPDATE: src/cli/actions/mod.rs (your action enum)
```

**Why this makes a good template:**
- No system dependencies (pure Rust, no OpenSSL)
- Workflows auto-configure from Cargo.toml
- Strict quality standards enforced
- Production patterns included (observability, error handling, testing)

See [`.github/TEMPLATE.md`](.github/TEMPLATE.md) for detailed instructions.

## Features

- Parse individual cron expressions
- Display next execution time in UTC
- Show time remaining using human-readable duration format (e.g., "2d 3h 15m 30s")
- Parse current user's crontab (`crontab -l`)
- Read and parse crontab files
- Support for comments in crontab files
- Verbose output mode

## Installation

### Prerequisites

- Rust toolchain (no system dependencies required)
- This project uses pure Rust dependencies (rustls), so no OpenSSL installation needed

### From source

```bash
cargo install --path .

# Include the optional OpenTelemetry exporter
cargo install --path . --features telemetry
```

### From crates.io

```bash
cargo install cron-when

# Include the optional OpenTelemetry exporter
cargo install cron-when --features telemetry
```

### Building static binaries (Linux)

For fully static binaries that work on any Linux distribution:

```bash
# Install musl target
rustup target add x86_64-unknown-linux-musl

# Build static binary
cargo build --release --target x86_64-unknown-linux-musl
```

## Usage

### Basic usage with cron expression

```bash
# Run every 5 minutes
cron-when "*/5 * * * *"

# Daily at midnight
cron-when "0 0 * * *"

# Every Monday at 2:30 AM
cron-when "30 2 * * 1"
```

### Verbose mode

Show the cron expression along with the output:

```bash
cron-when -v "*/5 * * * *"
```

### Color Output

`cron-when` supports colored output for better readability. By default, color is enabled when the output is a terminal (TTY).

- Force color: `cron-when --color "*/5 * * * *"` or `-c`
- Disable color: `cron-when --no-color "*/5 * * * *"`

This tool respects the [NO_COLOR](https://no-color.org/) standard. If the `NO_COLOR` environment variable is present, even as an empty value, color output will be disabled by default unless explicitly overridden by the `--color` flag.

The detection hierarchy is:
1.  `--no-color` flag (always disable)
2.  `--color` flag (always enable)
3.  `NO_COLOR` environment variable (disable if present)
4.  `CLICOLOR_FORCE` environment variable (enable if set and not "0")
5.  Auto-detection (enable if output is a terminal)

### Show next N occurrences

Display the next N times a cron expression will run:

```bash
# Show next 10 occurrences
cron-when --next 10 "*/5 * * * *"

# Or use short flag
cron-when -n 5 "0 0 * * *"
```

**Output:**
```
Expression: */5 * * * *

  1. 2025-11-09 12:15:00 UTC (2m50s)
  2. 2025-11-09 12:20:00 UTC (7m50s)
  3. 2025-11-09 12:25:00 UTC (12m50s)
  4. 2025-11-09 12:30:00 UTC (17m50s)
  5. 2025-11-09 12:35:00 UTC (22m50s)
  ...
```

### Parse current user's crontab

```bash
cron-when --crontab
# or
cron-when -l
```

### Parse crontab from file

```bash
cron-when --file /path/to/crontab
# or
cron-when -f /path/to/crontab
```

### Example crontab file

```cron
# Backup database every day at 2 AM
0 2 * * * /usr/local/bin/backup.sh

# Clean temporary files every hour
0 * * * * /usr/local/bin/cleanup.sh

# Send weekly report every Monday at 9 AM
0 9 * * 1 /usr/local/bin/weekly-report.sh
```

## Output Format

```
Next: 2024-11-09 15:30:00 UTC
Left: 2h 15m 30s
```

With comments from crontab:

```
# Backup database every day at 2 AM
Next: 2024-11-10 02:00:00 UTC
Left: 10h 30m 0s

# Clean temporary files every hour
Next: 2024-11-09 16:00:00 UTC
Left: 2h 30m 0s
```

## Cron Expression Format

The tool supports standard cron expressions with 5 fields:

```
* * * * *
│ │ │ │ │
│ │ │ │ └─── Day of week (0-6, Sunday=0)
│ │ │ └───── Month (1-12)
│ │ └─────── Day of month (1-31)
│ └───────── Hour (0-23)
└─────────── Minute (0-59)
```

### Supported syntax

- `*` - Any value
- `,` - Value list separator (e.g., `1,3,5`)
- `-` - Range of values (e.g., `1-5`)
- `/` - Step values (e.g., `*/5` for every 5 units)

### Examples

- `*/5 * * * *` - Every 5 minutes
- `0 0 * * *` - Daily at midnight
- `0 */6 * * *` - Every 6 hours
- `30 2 * * 1-5` - At 2:30 AM, Monday through Friday
- `0 0 1 * *` - First day of every month at midnight
- `0 0 * * 0` - Every Sunday at midnight

## Options

```
Usage: cron-when [OPTIONS] [CRON_EXPRESSION]

Arguments:
  [CRON_EXPRESSION]  Cron expression (e.g., "*/5 * * * *")

Options:
  -f, --file <FILE>   Read from file (crontab format)
  -l, --crontab       Parse current user's crontab
  -v, --verbose...    Show verbose output with cron expression
  -n, --next <COUNT>  Show next N occurrences of the cron expression
  -h, --help          Print help
  -V, --version       Print version
```

## Observability & Tracing

This CLI offers OpenTelemetry support for distributed tracing and observability
through the optional `telemetry` Cargo feature. The default build remains a
smaller cron utility without the OTLP/gRPC/TLS dependency stack.

> **📚 Educational Note:** This is intentionally over-engineered! A simple cron parser doesn't "need" distributed tracing. However, this project demonstrates production-grade observability patterns that you can learn from and apply to your own projects. See the [compile-time versus runtime design](CLI_ARCHITECTURE.md#compile-time-feature-vs-runtime-configuration) for a detailed discussion.

### Two Controls, Two Purposes

This example deliberately separates compile-time capability from runtime
configuration:

| Control | What it does | Requires rebuilding? | When evaluated? |
| --- | --- | --- | --- |
| Cargo feature: `telemetry` | Includes the OpenTelemetry/OTLP implementation and its optional dependencies | Yes | At compile time |
| Environment: `OTEL_EXPORTER_OTLP_ENDPOINT` | Creates an OTLP exporter in a telemetry-enabled binary | No | Once, at process startup |

Cargo calls `telemetry` a *feature*. It is also commonly described as a
compile-time feature flag, but it is not the same as a runtime release toggle:
changing it produces a different binary and therefore requires rebuilding and
redeploying that binary.

The endpoint is runtime configuration. Adding or removing it does not require
a new binary, but this CLI reads it only during startup, so the process must be
restarted after the environment changes. It is not a live, remotely controlled
rollout flag.

The resulting behavior is:

| Binary | Endpoint set? | Result |
| --- | --- | --- |
| Default build | Either | Local structured logging only; OTLP code is not present |
| `--features telemetry` | No | Local structured logging only; no exporter is created |
| `--features telemetry` | Yes | Local structured logging plus OTLP trace export |

This is a good fit for an optional, dependency-heavy capability. Use Cargo
features to control what a binary can do. Use runtime configuration for values
that vary between environments. If a project needs instant rollouts, per-user
targeting, or a kill switch without restarting processes, use a dedicated
runtime feature-management mechanism instead.

CI checks and tests both feature sets, then performs cross-platform release
builds for both configurations. This catches code that compiles only when a
feature is enabled—or only when it is absent. Published release artifacts still
use the documented default feature set unless the release workflow explicitly
enables another feature.

### Enabling Traces

First build or install a binary that contains telemetry support:

```bash
cargo build --release --features telemetry
# or
cargo install cron-when --features telemetry
```

Then activate its OTLP exporter at startup by providing an endpoint:

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
cron-when -v "*/5 * * * *"
```

### Using direnv

For convenience, you can use [direnv](https://direnv.net/) to automatically set environment variables:

```bash
# Copy the example file
cp .envrc.example .envrc

# Edit .envrc and uncomment the OTEL settings
# Then allow the directory
direnv allow
```

Example `.envrc` file:
```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
```

### Viewing Traces with Jaeger

Start Jaeger locally using Docker/Podman:

```bash
podman run -d --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 16686:16686 \
  -p 4317:4317 \
  jaegertracing/all-in-one:latest
```

Or use the justfile recipe:
```bash
just jaeger
```

Then access the Jaeger UI at [http://localhost:16686](http://localhost:16686)

### Supported Backends

The OTLP exporter works with any OpenTelemetry-compatible backend:

- **Jaeger** - Open source tracing
- **Honeycomb** - `OTEL_EXPORTER_OTLP_ENDPOINT=https://api.honeycomb.io:443`
- **Grafana Tempo** - Self-hosted or cloud
- **AWS X-Ray** - Via OpenTelemetry Collector
- **Google Cloud Trace** - Via OpenTelemetry Collector

### Additional Configuration

```bash
# Custom headers (e.g., for authentication)
export OTEL_EXPORTER_OTLP_HEADERS="x-honeycomb-team=YOUR_API_KEY"

# Service instance ID (auto-generated if not set)
export OTEL_SERVICE_INSTANCE_ID=my-instance-123

# Override log level
export RUST_LOG=debug
```

### Verbosity Levels

Combine with `-v` flags for different log levels:

```bash
cron-when -v "*/5 * * * *"    # INFO level
cron-when -vv "*/5 * * * *"   # DEBUG level
cron-when -vvv "*/5 * * * *"  # TRACE level
```

### Known Behavior: Flush Timeout

When tracing is enabled, you may see a timeout error on exit:

```
ERROR BatchSpanProcessor.Shutdown.Timeout
```

**This is expected and harmless!** The CLI exits faster (~10ms) than the span processor can flush (~5s). Your traces are still sent and will appear in Jaeger/Honeycomb/etc.

To suppress these messages:
```bash
export RUST_LOG="warn,opentelemetry_sdk=error"
```

See [CLI_ARCHITECTURE.md](CLI_ARCHITECTURE.md) for details on why this happens and alternative approaches.

## Development

### Development container

The repository includes a development container based on the setup used by
`s3m`, updated to the current Dev Container schema. It provides Rust, Clippy,
rustfmt, the MUSL target, the Cargo tools used by the `just` recipes, SOPS,
age, zsh, rust-analyzer, and debugging extensions.

In VS Code, install the **Dev Containers** extension, open this repository, and
select **Dev Containers: Reopen in Container**. Other tools implementing the
[Development Container Specification](https://containers.dev/) can use the
same `.devcontainer/devcontainer.json` file.

The container uses the non-root `vscode` user. Run the fast validation suite
after creation:

```bash
just test
```

For Fedora/Podman compatibility, the container disables SELinux label
separation for this development container. This lets the workspace and the
read-only staged secret be mounted without relabeling either host path. It does
not make the container privileged, but derived projects should keep this option
only when their container provider requires it.

`just full-test` additionally requires access to a Podman service. The base
container does not request privileged access or mount a host container-engine
socket automatically; configure that explicitly if your derived project needs
container integration tests.

#### Optional SOPS and age secrets

The container supports SOPS-encrypted files without storing a private age key
in Git. Before creating or rebuilding it, choose one host-side source:

```bash
# Recommended: a protected file outside this repository
export SOPS_AGE_KEY_FILE="$HOME/.config/sops/age/keys.txt"

# Or resolve the identity through the 1Password CLI
export SOPS_AGE_KEY_OP_REF='op://YOUR_VAULT/YOUR_ITEM/password'

# Direct values are supported for automation but are easier to leak
# export SOPS_AGE_KEY='AGE-SECRET-KEY-...'
```

The host initializer copies the selected identity to
`~/.cache/devcontainer-secrets/cron-when/sops-age-key` with mode `0600`. The
container receives that file read-only at `/run/secrets/sops-age-key` and sets
`SOPS_AGE_KEY_FILE` accordingly. If no identity is configured, container
creation still succeeds, but decryption is unavailable.

To start using encrypted repository files:

```bash
# Generate an identity outside the repository if you do not already have one
mkdir -p "$HOME/.config/sops/age"
age-keygen -o "$HOME/.config/sops/age/keys.txt"
chmod 600 "$HOME/.config/sops/age/keys.txt"

# Configure only the public recipient in Git
cp .sops.yaml.example .sops.yaml
age-keygen -y "$HOME/.config/sops/age/keys.txt"
# Replace the placeholder in .sops.yaml with the printed age1... recipient

# Create or edit an encrypted file
sops secrets/example.sops.yaml
```

Commit `.sops.yaml` and encrypted `*.sops.*` files. Never commit the age
identity, plaintext secret files, or decrypted output. Removing access to a
password manager does not revoke an age identity someone already possesses;
rotate the identity and re-encrypt affected files when access is revoked.

### Running tests

```bash
# Test the default, minimal feature set
cargo test

# Test every optional Cargo feature
cargo test --all-features

# Fast local checks
just test

# Includes the MUSL/Podman integration test
just full-test
```

### Building

```bash
# Minimal binary (the default)
cargo build --release

# Binary with optional OTLP telemetry support
cargo build --release --features telemetry
```

### Running locally

```bash
cargo run -- "*/5 * * * *"
```
