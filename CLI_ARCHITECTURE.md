# CLI Architecture

This document describes the modular CLI architecture used in this project. This pattern provides excellent separation of concerns and is ideal for template projects.

> **📋 Template Note:** This architecture is designed to be copied to other Rust CLI projects. The separation allows you to keep the entire `src/cli/` directory and only replace your domain-specific logic (like `src/crontab.rs` and `src/output.rs` in this example). See [`.github/TEMPLATE.md`](.github/TEMPLATE.md) for usage instructions.

## Directory Structure

```
src/cli/
├── actions/           # Action definitions and execution
│   ├── mod.rs        # Action enum
│   └── run.rs        # Execution logic
├── commands/          # CLI command definitions
│   └── mod.rs        # Clap command structure
├── dispatch/          # ArgMatches → Action conversion
│   └── mod.rs        # Handler logic
├── mod.rs            # Module exports
├── start.rs          # Main orchestrator
└── telemetry.rs      # Logging/verbosity handling
```

## Data Flow

```
bin/cron-when.rs
    ↓
cli::start()
    ↓
┌─────────────────────────────────────────────┐
│ 1. commands::new().get_matches()            │  Parse CLI arguments
│    ↓                                         │
│ 2. telemetry::Level::from(verbose_count)    │  Extract verbosity
│    ↓                                         │
│ 3. telemetry::init(level)                   │  Initialize logging
│    ↓                                         │
│ 4. dispatch::handler(&matches)              │  Convert to Action
│    ↓                                         │
│ 5. action.execute()                         │  Execute action
└─────────────────────────────────────────────┘
```

## Module Responsibilities

### 1. `commands/mod.rs` - CLI Definition

**Purpose:** Define the CLI structure using clap
**Responsibility:** ONLY command-line argument definitions
**No business logic**

```rust
pub fn new() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .arg(Arg::new("cron")...)
        .arg(Arg::new("file")...)
        // etc.
}
```

**Key Points:**
- Pure clap definitions
- Uses `env!()` macros for metadata
- Fully testable in isolation
- No dependencies on other modules

### 2. `dispatch/mod.rs` - ArgMatches → Action

**Purpose:** Convert clap's `ArgMatches` into typed `Action` enum
**Responsibility:** Validation and routing logic

```rust
pub fn handler(matches: &ArgMatches) -> Result<Action> {
    if matches.get_flag("crontab") {
        Ok(Action::Crontab { verbose })
    } else if let Some(file) = matches.get_one::<String>("file") {
        Ok(Action::File { path, verbose })
    }
    // etc.
}
```

**Key Points:**
- Single source of truth for argument → action mapping
- Error handling for missing/invalid arguments
- Extracts and validates all parameters
- Returns strongly-typed Action

### 3. `actions/` - Action Definition & Execution

**Purpose:** Define all possible actions and their execution

#### `actions/mod.rs` - Action Enum

```rust
#[derive(Debug)]
pub enum Action {
    Single { expression: String, verbose: bool },
    File { path: PathBuf, verbose: bool },
    Crontab { verbose: bool },
}

impl Action {
    pub fn execute(&self) -> Result<()> {
        run::execute(self)
    }
}
```

#### `actions/run.rs` - Execution Logic

```rust
pub fn execute(action: &Action) -> Result<()> {
    match action {
        Action::Single { expression, verbose } => {
            // Implementation
        }
        // etc.
    }
}
```

**Key Points:**
- Action enum is the core contract
- Clear separation between definition and execution
- Easy to add new actions
- Execution logic can be complex without cluttering definitions

### 4. `telemetry.rs` - Observability/Tracing

**Purpose:** Production-ready telemetry initialization with OpenTelemetry support
**Responsibility:** Set up logging and distributed tracing compatible with multiple providers

```rust
pub fn init(verbosity_level: Option<tracing::Level>) -> Result<()> {
    // Initialize tracing-subscriber with console and optional OpenTelemetry layers
}

pub fn shutdown_tracer() {
    // Gracefully shutdown tracer provider and flush pending spans
}
```

**Key Points:**
- Uses `tracing::Level` directly (INFO, DEBUG, TRACE)
- Initializes `tracing-subscriber` for structured logging
- **Production-ready** OpenTelemetry gRPC exporter with:
  - TLS support (native roots)
  - Custom headers (e.g., Honeycomb API keys)
  - Binary metadata support (base64-encoded)
  - Compression (gzip)
  - Proper resource attributes (service name, version, instance ID)
- **Multi-provider compatible:** Works with Honeycomb, Jaeger, Grafana Tempo, etc.
- Graceful shutdown with span flushing
- **Educational template** showing enterprise-grade observability

**Supported Environment Variables:**

```bash
# Required: OTLP endpoint (defaults to http://localhost:4317)
OTEL_EXPORTER_OTLP_ENDPOINT=https://api.honeycomb.io:443

# Optional: Custom headers (comma-separated key=value pairs)
# Supports binary metadata (keys ending with -bin, values base64-encoded)
OTEL_EXPORTER_OTLP_HEADERS="x-honeycomb-team=YOUR_API_KEY"

# Optional: Protocol (only 'grpc' supported, others ignored)
OTEL_EXPORTER_OTLP_PROTOCOL=grpc

# Optional: Service instance ID (auto-generated ULID if not set)
OTEL_SERVICE_INSTANCE_ID=my-instance-123

# Optional: Override log level via RUST_LOG
RUST_LOG=debug
```

**Instrumentation Example:**

```rust
use tracing::{info, instrument};

#[instrument(level = "info", fields(path = %path.display()))]
pub fn parse_file(path: &Path) -> Result<Vec<CronEntry>> {
    info!(entry_count = entries.len(), "Parsed file entries");
    // ...
}
```

**Running with Different Providers:**

```bash
# Console output only (no OTEL endpoint set)
cron-when -vv "*/5 * * * *"

# Jaeger (local)
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
cron-when -v "*/5 * * * *"

# Honeycomb
export OTEL_EXPORTER_OTLP_ENDPOINT=https://api.honeycomb.io:443
export OTEL_EXPORTER_OTLP_HEADERS="x-honeycomb-team=YOUR_API_KEY"
cron-when -v "*/5 * * * *"

# Grafana Cloud / Tempo
export OTEL_EXPORTER_OTLP_ENDPOINT=https://tempo-prod-us-central1.grafana.net:443
export OTEL_EXPORTER_OTLP_HEADERS="authorization=Basic YOUR_BASE64_TOKEN"
cron-when -v "*/5 * * * *"
```

### 5. `start.rs` - Main Orchestrator

**Purpose:** Coordinate the entire CLI flow
**Responsibility:** Execute the 5-step process in order

```rust
pub fn start() -> Result<()> {
    let matches = commands::new().get_matches();
    let verbosity = telemetry::Level::from(matches.get_count("verbose"));
    telemetry::init(verbosity)?;
    let action = dispatch::handler(&matches)?;
    action.execute()?;
    Ok(())
}
```

**Key Points:**
- **No business logic** - pure orchestration
- Clear, linear flow
- Easy to understand at a glance
- Entry point for the entire CLI

### 6. `mod.rs` - Module Exports

**Purpose:** Control public API of the cli module

```rust
pub mod actions;
pub mod commands;
pub mod dispatch;
pub mod telemetry;

mod start;
pub use self::start::start;
```

**Key Points:**
- Only `start` is re-exported
- Other modules are public but not re-exported
- Clear public API: `cli::start()`

## Benefits of This Architecture

### 1. Separation of Concerns

Each module has ONE job:
- `commands` → Define CLI
- `dispatch` → Route arguments
- `actions` → Define & execute
- `telemetry` → Handle logging
- `start` → Orchestrate

### 2. Testability

Every module can be tested independently:

```rust
// Test command parsing
#[test]
fn test_parse_cron() {
    let matches = commands::new()
        .get_matches_from(vec!["app", "*/5 * * * *"]);
    assert!(matches.contains_id("cron"));
}

// Test dispatch logic
#[test]
fn test_handler() {
    let matches = commands::new()
        .get_matches_from(vec!["app", "--crontab"]);
    let action = dispatch::handler(&matches).unwrap();
    assert!(matches!(action, Action::Crontab { .. }));
}
```

### 3. Maintainability

**Adding a new command/action:**

1. Add argument in `commands/mod.rs`
2. Add variant in `actions/mod.rs`
3. Add routing in `dispatch/mod.rs`
4. Add execution in `actions/run.rs`
5. Done!

Each change is localized and predictable.

### 4. Template-Friendly

This structure is **generic** and works for any CLI:

```bash
# Copy to new project
cp -r src/cli /path/to/new-project/src/

# Update:
# - commands/mod.rs: Your CLI args
# - actions/mod.rs: Your action variants
# - dispatch/mod.rs: Your routing logic
# - actions/run.rs: Your execution logic
```

### 5. Scalability

As your CLI grows:
- Add more action variants
- Add more modules in `actions/`
- Add subcommands in `commands/`
- Never modify `start.rs` (it stays clean)

## Comparison with Simple Approach

### Simple (for small CLIs):

```rust
fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli {
        Cli { cron: Some(expr), .. } => process_cron(&expr)?,
        Cli { file: Some(path), .. } => process_file(&path)?,
        // Mix of parsing and execution
    }
    Ok(())
}
```

**Pros:** Simple, minimal code
**Cons:** Grows messy, hard to test, mixed concerns

### Modular (this architecture):

```rust
pub fn start() -> Result<()> {
    let matches = commands::new().get_matches();
    let verbosity = telemetry::Level::from(matches.get_count("verbose"));
    telemetry::init(verbosity)?;
    let action = dispatch::handler(&matches)?;
    action.execute()?;
    Ok(())
}
```

**Pros:** Clean separation, testable, scales well
**Cons:** More files (but small and focused)

## When to Use This Pattern

✅ **Use this pattern when:**
- Building a template/reference project
- CLI has 3+ distinct operations
- You want excellent testability
- Multiple people work on the code
- You value long-term maintainability

❌ **Consider simpler approach when:**
- Single operation CLI
- Prototype/one-off tool
- Extreme simplicity is the priority

## Real-World Usage

This pattern is used in production projects:
- [ssh-vault](https://github.com/ssh-vault/ssh-vault)
- [pg_exporter](https://github.com/nbari/pg_exporter)
- [s3m](https://github.com/s3m/s3m)

It's proven to scale from simple to complex CLIs while maintaining clarity.

## Observability & Instrumentation

This project includes comprehensive telemetry using OpenTelemetry and `tracing` as an **educational example** for modern Rust observability patterns.

### Why Instrument?

**For educational purposes**, this project demonstrates:
1. **Structured logging** with `tracing` instead of `println!` debugging
2. **Distributed tracing** with OpenTelemetry for production environments
3. **Performance monitoring** with automatic span timing
4. **Debugging aid** with contextual information in logs

### Instrumentation Levels

The codebase uses semantic instrumentation levels:

| Level | Usage | Example |
|-------|-------|---------|
| `info` | Function entry/exit, major operations | `parse_file()`, `execute()` |
| `debug` | Detailed processing information | Loop iterations, intermediate values |
| `trace` | Very detailed debugging | Not used in this simple CLI |

### Instrumented Modules

**1. `crontab.rs`** - File and crontab parsing
```rust
#[instrument(level = "info", fields(path = %path.display()))]
pub fn parse_file(path: &Path) -> Result<Vec<CronEntry>> {
    debug!("Reading crontab file");
    info!(entry_count = entries.len(), "Parsed file entries");
}
```

**2. `output.rs`** - Display and formatting
```rust
#[instrument(level = "info", fields(expression = %expression))]
pub fn display_single(expression: &str, ...) -> Result<()> {
    info!(next_execution = %next, "Calculated next time");
}
```

**3. `cli/actions/*.rs`** - Action execution
```rust
#[instrument(level = "info")]
pub fn execute(expression: &str, ...) -> Result<()> {
    info!("Displaying single execution time");
}
```

### Using Telemetry

**Console Logging (Development):**

```bash
# Info level
cron-when -v "*/5 * * * *"

# Debug level
cron-when -vv "*/5 * * * *"

# Trace level (everything)
cron-when -vvv "*/5 * * * *"
```

**OpenTelemetry (Production):**

```bash
# Start Jaeger (docker/podman)
podman run -d --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 16686:16686 \
  -p 4317:4317 \
  jaegertracing/all-in-one:latest

# Run with OTLP exporter
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
cron-when -v "*/5 * * * *"

# View traces at http://localhost:16686

# Honeycomb example
export OTEL_EXPORTER_OTLP_ENDPOINT=https://api.honeycomb.io:443
export OTEL_EXPORTER_OTLP_HEADERS="x-honeycomb-team=YOUR_API_KEY"
cron-when -v "*/5 * * * *"
```

**Environment Variables:**

```bash
# Control log level via environment
RUST_LOG=debug cron-when "*/5 * * * *"

# OpenTelemetry endpoint (gRPC only)
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317

# Custom headers (comma-separated)
OTEL_EXPORTER_OTLP_HEADERS="key1=value1,key2=value2"

# Service instance ID (auto-generated if not set)
OTEL_SERVICE_INSTANCE_ID=my-instance-123

# Service name and version are automatically set from Cargo.toml
```

### Educational Note: OpenTelemetry in a Tiny CLI

**This is intentionally over-engineered for educational purposes!**

A simple cron parser doesn't "need" distributed tracing. However, this project demonstrates:

1. **How to add production-grade observability** to any Rust CLI
2. **OpenTelemetry integration patterns** that scale from tiny tools to large systems
3. **Async runtime considerations** for short-lived processes

#### The Tradeoff

**Cost:**
- Adds ~15-20 dependencies (OpenTelemetry ecosystem)
- Binary size increases by ~2-3 MB
- Adds Tokio async runtime overhead (~1-2ms startup)
- Slightly slower build times (~5-10 seconds)

**Benefit:**
- **Optional at runtime** - Zero cost if `OTEL_EXPORTER_OTLP_ENDPOINT` not set
- Learn production observability patterns
- Template for adding tracing to your own CLIs
- Works with any OTLP backend (Jaeger, Honeycomb, Grafana, etc.)

#### Known Limitation: Flush Timeout

Short-lived CLIs have a challenge with OpenTelemetry:

```
Problem:
  CLI execution:  ~10ms
  Span flush:     ~5000ms (timeout)
  Result:         Timeout error on exit

Solution implemented:
  1. force_flush() - Try to send spans immediately
  2. tokio::time::sleep(200ms) - Give time for async operations
  3. Spans are sent asynchronously anyway!

Result:
  ✅ Traces appear in Jaeger
  ⚠️  You may see "BatchSpanProcessor.Shutdown.Timeout" (cosmetic)
```

To suppress timeout errors:
```bash
export RUST_LOG="warn,opentelemetry_sdk=error"
```

#### Key Architectural Decisions

**1. Tokio Runtime (`current_thread` flavor)**

```rust
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()>
```

Why: OpenTelemetry OTLP uses gRPC which requires async runtime
Choice: `current_thread` instead of `multi_thread` (3x faster startup)

**2. Conditional Initialization**

Tracing only initializes if `OTEL_EXPORTER_OTLP_ENDPOINT` is set:

```rust
pub fn init(level: Option<Level>) -> Result<()> {
    // Only init OTLP if endpoint configured
    if env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() {
        let tracer = init_tracer()?;
        // ... setup tracing
    } else {
        // Simple logging only
    }
}
```

This means **zero overhead** when tracing is disabled!

**3. Graceful Degradation**

If OTLP export fails:
- ✅ CLI still works normally
- ✅ Console logging continues
- ⚠️  Timeout errors appear (can be suppressed)

#### What You Can Learn

Use this code as a reference for:

1. **Adding OpenTelemetry to your CLI tools**
   - Copy `src/cli/telemetry.rs`
   - Add `#[instrument]` to functions you want to trace
   - Set `OTEL_EXPORTER_OTLP_ENDPOINT` when needed

2. **Handling short-lived process challenges**
   - force_flush() pattern
   - Brief sleep before exit
   - Accepting cosmetic timeout errors

3. **Tokio runtime selection**
   - When `current_thread` is sufficient
   - How to minimize async overhead

4. **Production-ready patterns**
   - Multiple backend support (Jaeger, Honeycomb, etc.)
   - Header authentication
   - TLS support
   - Compression (gzip)

#### Real-World Usage

**When to add this to your CLI:**

✅ **Good fit:**
- Long-running CLIs or daemons
- Complex multi-step operations
- Tools used in production systems
- When debugging timing issues

❌ **Overkill:**
- Simple one-shot commands
- Development-only tools
- When milliseconds matter

**This project chooses "overkill" intentionally** - it's a learning template!

### Benefits for Templates

Including telemetry demonstrates:
- ✅ **Best practices** for production-ready Rust CLIs
- ✅ **Debugging techniques** beyond `println!`
- ✅ **Observability patterns** that scale from CLI to services
- ✅ **Performance insights** with automatic span timing
- ✅ **Context propagation** for distributed systems

### Minimal Impact

The telemetry has near-zero runtime overhead when:
- Verbosity is set to `Normal` (default)
- No OTLP endpoint is configured
- Compiler optimizations are enabled (`--release`)

### For Simple Projects

If you don't need telemetry, it's easy to remove:

```bash
# 1. Remove dependencies from Cargo.toml
# Remove: opentelemetry, opentelemetry-otlp, 
#         opentelemetry_sdk, tracing-opentelemetry

# 2. Simplify telemetry.rs
pub fn init(_level: Level) -> Result<()> {
    Ok(())
}

# 3. Remove tracing imports and #[instrument] macros
```

The architecture remains the same - telemetry is completely orthogonal.

## Summary

This architecture provides:
- **Clear data flow:** commands → dispatch → actions → execute
- **Single responsibility:** Each module has one job
- **Easy testing:** Every component tests independently
- **Simple maintenance:** Changes are localized
- **Great template:** Copy and modify for new projects
- **Modern observability:** Production-ready telemetry patterns

The slight overhead of more files pays dividends in code quality and developer experience.
