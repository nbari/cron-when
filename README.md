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

- Demonstrates production-grade observability patterns (OpenTelemetry)
- Shows how to integrate distributed tracing in Rust CLIs
- Exhibits modular CLI architecture with separation of concerns
- Includes comprehensive testing (unit + container integration tests)
- Documents tradeoffs and architectural decisions

The OpenTelemetry integration adds ~15-20 dependencies and 2-3 MB to the binary, but provides:
- **Zero runtime cost when disabled** (no `OTEL_EXPORTER_OTLP_ENDPOINT` set)
- Multi-backend support (Jaeger, Honeycomb, Grafana, AWS X-Ray, etc.)
- Production-ready patterns you can copy to your projects

See [`CLI_ARCHITECTURE.md`](CLI_ARCHITECTURE.md) for detailed discussion of design decisions.

## Features

- Parse individual cron expressions
- Display next execution time in UTC
- Show time remaining using human-readable duration format (e.g., "2d 3h 15m 30s")
- Parse current user's crontab (`crontab -l`)
- Read and parse crontab files
- Support for comments in crontab files
- Verbose output mode

## Installation

### From source

```bash
cargo install --path .
```

### From crates.io

```bash
cargo install cron-when
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

This CLI includes OpenTelemetry support for distributed tracing and observability.

> **📚 Educational Note:** This is intentionally over-engineered! A simple cron parser doesn't "need" distributed tracing. However, this project demonstrates production-grade observability patterns that you can learn from and apply to your own projects. See [CLI_ARCHITECTURE.md](CLI_ARCHITECTURE.md#educational-note-opentelemetry-in-a-tiny-cli) for detailed discussion.

### Enabling Traces

Traces are automatically sent when the `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable is set:

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

### Running tests

```bash
cargo test
# Or with justfile
just test
```

### Building

```bash
cargo build --release
```

### Running locally

```bash
cargo run -- "*/5 * * * *"
```
