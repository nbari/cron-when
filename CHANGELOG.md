# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

OpenTelemetry adds ~2-3 MB to binary but has zero runtime cost when disabled.
See `CLI_ARCHITECTURE.md` for detailed discussion of design decisions.
