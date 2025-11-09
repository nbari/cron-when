# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-11-09

### Added
- Initial release
- Parse individual cron expressions with standard 5-field format
- Display next execution time in UTC
- Show time remaining using human-readable duration format (e.g., "2d 3h 15m 30s")
- Parse current user's crontab (`crontab -l`)
- Read and parse crontab files with `-f, --file` option
- Support for comments in crontab files
- Verbose output mode with `-v, --verbose` flag
- Show next N occurrences with `-n, --next` option
- OpenTelemetry integration for observability
- Comprehensive test suite with 23 tests
- GitHub Actions CI/CD workflows
- Multi-platform release builds (Linux, macOS, Windows)
- RPM and DEB package generation
- Benchmark suite for performance testing
- Static MUSL builds for Linux

### Features
- Support for standard cron syntax: `*`, `,`, `-`, `/`
- Environment variable parsing in crontab files
- Clean CLI interface with clap
- Zero runtime dependencies in release builds
