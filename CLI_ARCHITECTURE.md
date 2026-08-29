# CLI Architecture

This document describes the modular CLI architecture used in this project. This pattern provides excellent separation of concerns and is ideal for template projects.

> **📋 Template Note:** This architecture is designed to be copied to other Rust CLI projects. The separation allows you to keep the entire `src/cli/` directory and only replace your domain-specific logic (like `src/crontab.rs` and `src/output.rs` in this example). See [`.github/TEMPLATE.md`](.github/TEMPLATE.md) for usage instructions.

## Directory Structure

```
src/cli/
├── actions/           # Action definitions and execution
│   ├── mod.rs        # Action enum
│   ├── single.rs     # Single expression logic
│   ├── file.rs       # File parsing logic
│   └── crontab.rs    # System crontab logic
├── commands/          # CLI command definitions
│   └── mod.rs        # Clap command structure
├── dispatch/          # ArgMatches → Action conversion
│   └── mod.rs         # Handler logic
├── mod.rs             # Module exports
├── start.rs           # Main orchestrator
└── telemetry.rs       # Tracing and logging
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
│ 5. Execute via binary match                 │  Execute action
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
        .arg(arg_cron())
        .arg(arg_file())
        // etc.
}
```

**Key Points:**
- Pure clap definitions
- Uses `env!()` macros for metadata
- Fully testable in isolation
- Helper functions for each argument to keep code clean and satisfy lints

### 2. `dispatch/mod.rs` - ArgMatches → Action

**Purpose:** Convert clap's `ArgMatches` into typed `Action` enum
**Responsibility:** Validation and routing logic

```rust
pub fn handler(matches: &ArgMatches) -> Result<Action> {
    if matches.get_flag("crontab") {
        Ok(Action::Crontab { verbose, color })
    } else if let Some(file) = matches.get_one::<String>("file") {
        Ok(Action::File { path, verbose, color })
    }
    // etc.
}
```

**Key Points:**
- Single source of truth for argument → action mapping
- Error handling for missing/invalid arguments
- Extracts and validates all parameters (including environment variables for colors)
- Returns strongly-typed Action

### 3. `actions/` - Action Definition & Execution

**Purpose:** Define all possible actions and their execution logic

#### `actions/mod.rs` - Action Enum

```rust
#[derive(Debug)]
pub enum Action {
    Single { expression: String, verbose: bool, next: Option<u32>, color: bool },
    File { path: PathBuf, verbose: bool, color: bool },
    Crontab { verbose: bool, color: bool },
}
```

#### `actions/*.rs` - Execution Logic

Each variant has a corresponding module in `src/cli/actions/` that handles its specific execution logic, calling into domain services like `crontab` and `output`.

**Key Points:**
- Action enum is the core contract
- Clear separation between definition and execution
- Easy to add new actions by adding variants and corresponding modules

### 4. `telemetry.rs` - Observability/Tracing

**Purpose:** Structured logging with feature-gated OpenTelemetry support
**Responsibility:** Always set up local logging and optionally add distributed tracing

```rust
pub fn init(verbosity_level: Option<tracing::Level>) -> Result<()> {
    // Initialize tracing-subscriber with console and optional OpenTelemetry layers
}

#[must_use]
pub fn shutdown_tracer() -> bool {
    // Flush pending spans and report whether a provider existed
}
```

**Key Points:**
- Uses `std::sync::OnceLock` for safe, one-time initialization (Rust 2024)
- Initializes `tracing-subscriber` for structured logging
- Keeps the OpenTelemetry dependency stack behind the default-off `telemetry` feature
- Provides an OpenTelemetry gRPC exporter with TLS and compression when enabled
- Graceful shutdown with span flushing

#### Compile-Time Feature vs Runtime Configuration

These are separate architectural decisions:

1. `cargo build` creates a minimal binary without the OTLP implementation.
2. `cargo build --features telemetry` creates a binary that can export traces.
3. A telemetry-enabled binary creates an exporter only when
   `OTEL_EXPORTER_OTLP_ENDPOINT` is present during process startup.

The Cargo feature is a compile-time feature flag: it avoids compiling and
shipping dependencies that a user does not need, but changing it requires a
new binary. The environment variable is runtime configuration: the same binary
can be configured differently in development, staging, and production, though
this CLI must be restarted to observe a changed environment.

This two-stage design follows several reusable practices:

- Keep optional dependencies marked `optional = true` and group them under one
  user-facing feature.
- Keep features additive. Enabling a feature should add capability rather than
  remove or conflict with another capability.
- Ensure the default and all-feature configurations both compile and pass tests.
- Keep baseline behavior, such as local logging, available in either build.
- Document which feature set is used for distributed release binaries.

Do not use a compile-time Cargo feature when a requirement calls for dynamic
rollouts, per-user targeting, or an emergency switch without rebuilding. Those
cases need a runtime feature-management design, usually with dynamic
configuration rather than a startup-only environment variable.

### 5. `start.rs` - Main Orchestrator

**Purpose:** Coordinate the initial CLI setup
**Responsibility:** Execute the setup steps in order

```rust
pub fn start() -> Result<Action> {
    let matches = commands::new().get_matches();
    let verbosity = get_verbosity_level(matches.get_count("verbose"));
    telemetry::init(verbosity)?;
    let action = dispatch::handler(&matches)?;
    Ok(action)
}
```

**Key Points:**
- **No business logic** - pure orchestration
- Returns the `Action` to the binary for execution
- Easy to understand at a glance

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
- Only `start` is re-exported for simple usage in `main.rs`
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

Every module can be tested independently. Output logic is also testable by using generic `io::Write` targets in the `output` module.

### 3. Maintainability

**Adding a new command/action:**

1. Add argument in `commands/mod.rs` (using a helper function)
2. Add variant in `actions/mod.rs`
3. Add routing in `dispatch/mod.rs`
4. Add execution module in `actions/`
5. Done!

### 4. Scalability

As your CLI grows, the structure remains clean and organized. The use of strict lints ensures high code quality across all modules.

## Summary

This architecture provides a solid foundation for building production-grade Rust CLIs. It emphasizes modularity, testability, and observability, making it an excellent template for new projects.
