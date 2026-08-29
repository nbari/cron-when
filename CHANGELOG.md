# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-03-31

### Added
- Support for cron aliases: `@reboot`, `@yearly`, `@annually`, `@monthly`, `@weekly`, `@daily`, `@midnight`, `@hourly`.
- Multi-line comment support: The parser now collects all preceding comment lines for a cron job, providing better context.
- Inline comment support: Cron entries can now have trailing comments when `#` starts an unquoted, whitespace-delimited comment (e.g., `0 0 * * * command # backup`).
- Generic output writers: Refactored output functions to support `std::io::Write`, enabling easier testing and potential redirection to different targets.

### Changed
- Refactored CLI argument definitions: Extracted into helper functions to satisfy strict code length lints.
- Modernized internal state management: Replaced `once_cell::sync::OnceCell` with the standard library's `std::sync::OnceLock` (modern Rust/2024 edition).
- Improved parsing robustness: Switched to safer `.get()` and `.first()` methods to comply with strict `indexing_slicing` lints.
- Simplified environment variable detection: Now uses `split_once` for cleaner logic.

### Dependencies
- Removed `once_cell` dependency.

### Technical Details
- Added `_with_writer` variants to the output module for testability.
- Updated `src/crontab.rs` with stricter alias validation, safer inline comment parsing, and broader unit test coverage.
- Satisfied all strict `clippy` lints including `too_many_lines`, `indexing_slicing`, and `unwrap_used`.

## [0.4.0] - 2025-11-23

### Changed
- **BREAKING**: Migrated from OpenSSL to pure Rustls for all TLS operations
  - Removed OpenSSL dependency completely
  - Now uses `rustls` with `webpki-roots` for certificate validation
  - Binary size unchanged, but eliminates system OpenSSL requirement
  - Cross-platform builds simplified (no OpenSSL needed on Windows)
- Removed `[features]` section from `Cargo.toml` (no longer needed)
- Updated GitHub Actions workflows to remove OpenSSL installation steps
- Applied strict clippy lints for code quality:
  - All `pedantic` lints enabled and enforced
  - Safety-critical lints: `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`
  - Comprehensive error handling throughout codebase
  - Added `# Errors` documentation to all public functions returning `Result`

### Dependencies
- Added `tonic` with `tls-webpki-roots` feature (replaces `tls-native-roots`)
- Added `opentelemetry-otlp` with `tls-webpki-roots` feature
- Removed `openssl` dependency
- Added `CDLA-Permissive-2.0` to allowed licenses (required by `webpki-roots`)

### Technical Details
For educational purposes, this migration demonstrates:
- How to migrate from OpenSSL to pure Rust TLS implementation
- Proper error handling patterns that satisfy strict lints
- Safe alternatives to panicking code (`.get()` vs `[]`, `try_from` vs `as`)
- Documentation standards for library code

## [0.3.0] - 2025-11-10

### Added
- Command display: Shows the command that will be executed by each cron job
- `--color` / `-c` flag: Enable colored output for better readability
- `CRON_WHEN_COLOR` environment variable: Set to `1` to enable colors globally
- Extended help documentation with detailed examples and constraints for all options
- Color scheme: Green (Cron), Red (Command), Blue (Next), Yellow (Left/expression)

### Changed
- Help text now includes detailed long-form documentation (use `--help` vs `-h`)
- `--next` constraint (1-100) is now documented in help text (fixes #1)

### Dependencies
- Added `colored` v3 crate for ANSI color support

## [0.2.0] - 2025-11-10

### Added
- Local timezone display alongside UTC for better readability
- Cron expressions now always shown with quotes for clarity
- Improved output formatting with consistent spacing between entries

### Changed
- Datetime output now shows both UTC and local time: `2025-11-10 15:35:00 UTC (16:35:00 +01:00)`
- Cron expression displayed in all modes (not just verbose): `Cron: "*/5 * * * *"`
- Better visual separation between multiple crontab entries

### Testing
- Added 13 comprehensive new tests (50 total tests, up from 37)
- Enhanced datetime formatting validation
- Added tests for multiple entry display scenarios
- Added validation for various cron expression patterns

## [0.1.0] - 2025-11-09

### Added
- Parse cron expressions with standard 5-field format (`* * * * *`)
- Display next execution time in UTC with human-readable duration
- Parse current user's crontab (`--crontab` flag)
- Read crontab files with `-f, --file` option
- Show next N occurrences with `-n, --next` option
- Support for comments and environment variables in crontab files
- Verbose output with multiple levels: `-v` (INFO), `-vv` (DEBUG), `-vvv` (TRACE)
- OpenTelemetry integration for distributed tracing (educational - see docs)
- Modular CLI architecture (actions/commands/dispatch pattern)
- Container integration tests with real crontab validation
- Multi-platform builds (Linux, macOS, Windows) with static MUSL binaries
- Comprehensive test suite (37 unit tests + container integration)
- GitHub Actions CI/CD workflows
- DEB/RPM package generation

### Documentation
- Detailed architecture guide (`CLI_ARCHITECTURE.md`)
- OpenTelemetry educational notes with tradeoff analysis
- Real-world usage examples ([ssh-vault](https://github.com/ssh-vault/ssh-vault), [pg_exporter](https://github.com/nbari/pg_exporter), [s3m](https://github.com/s3m/s3m))

### Notes
This project is intentionally over-engineered as an educational template demonstrating:
- Production-grade observability patterns (OpenTelemetry)
- Modular CLI architecture with clean separation of concerns
- Comprehensive testing including container integration tests
- CI/CD best practices with GitHub Actions

At the time of this release, OpenTelemetry added ~2-3 MB to every binary while
the exporter remained inactive when it was not configured. Current releases
place that dependency stack behind the default-off `telemetry` Cargo feature;
see `README.md` and `CLI_ARCHITECTURE.md` for the current behavior and tradeoffs.
