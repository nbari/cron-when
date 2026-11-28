# Agent instructions

Use this guide for contributors and agents. Projects that adopt this template
should update the commands, paths, and security assumptions to match their code.

## Working agreement

- Follow [AI_POLICY.md](AI_POLICY.md). A human owns and reviews each change.
- Read the relevant module, its callers, and the tests before changing behavior.
  Keep the diff focused and preserve unrelated work in the checkout.
- Check [.justfile](.justfile) for supported commands. See the
  [development guide](README.md#development) for the local environment and
  [RELEASING.md](RELEASING.md) for release work.

## Rust and CLI conventions

- This is a single Rust crate. The binary entry point is
  [src/bin/cron-when.rs](src/bin/cron-when.rs). Read
  [CLI_ARCHITECTURE.md](CLI_ARCHITECTURE.md) before changing how CLI declarations,
  dispatch, and execution fit together.
- Define CLI options and defaults in Clap and pass validated, typed values into
  the code that uses them. Preserve the documented flag and environment-variable
  precedence.
- Use rustfmt and idiomatic Rust names. Keep modules focused on one
  responsibility and keep public behavior stable within the requested change.
- Treat the Rust version and lint settings in [Cargo.toml](Cargo.toml) as the
  source of truth. Use checked access and return contextual errors for invalid
  inputs and unavailable resources. Keep panics, `unwrap`, and `expect` out of
  production paths. Fix lint findings at the source.
- Keep telemetry optional and preserve both the default and all-features
  builds.

## Security and configuration

- Validate untrusted input at the boundary that owns it. For this CLI, pay
  particular attention to cron expressions, crontab files, paths, terminal
  output, and telemetry configuration. Keep resource use bounded where inputs
  can drive work.
- Preserve authentication, authorization, token semantics, trust boundaries,
  and existing validation and rate controls when working in code that uses
  them. Get explicit approval for changes to those contracts beyond the
  request. Derive access decisions from verified identity and trusted data,
  enforce scopes at the owning boundary, and keep decision helpers
  side-effect-free.
- Use established libraries for security-sensitive protocols and cryptography.
  Keep certificate and hostname verification enabled. Trace relevant failure
  paths and preserve fail-closed behavior for access checks.
- Review security changes through callers, configuration, and persistence.
  Where relevant, check token lifetime, replay, privilege boundaries, and
  separation between users or tenants. Keep the fix at the boundary that
  makes the decision.
- Keep tokens, passwords, private keys, credential-bearing URLs, and sensitive
  input out of commits, fixtures, logs, errors, and telemetry. Pass credentials
  through the project's supported environment or secret manager and use
  synthetic credentials in tests.
- Keep GitHub Actions permissions scoped to each job. Pin external actions to
  full commit SHAs, keep checkout credentials out of jobs that only read
  source, and pass untrusted workflow inputs through environment variables
  with proper shell quoting.
- Use local fixtures or explicitly authorized test environments for defensive
  validation. Report suspected vulnerabilities privately using the
  [security policy](.github/SECURITY.md).

## Validation

- Run `just test` for Rust changes. It runs Clippy with
  `--all-targets --all-features`, checks formatting, and runs unit tests.
  For feature changes, run `cargo test --locked` and
  `cargo test --locked --all-features`.
- Use `just full-test` for changes that affect the static MUSL binary or its
  container integration. The integration test uses Podman. Use local services
  and dedicated test data for integration tests.
- Run `just audit` and `just deny` when changing dependencies. Keep the
  lockfile updates in scope and preserve the locked CI builds.
- Cover the behavior that changed with focused regression tests where the
  existing tests miss it. For security-sensitive changes, exercise relevant
  invalid or unauthorized inputs and assert both rejection and the expected
  protected state. Report skipped or blocked checks alongside the results.

## Documentation and commits

- Update usage, configuration, and operational docs in the same change as the
  behavior they describe. Use module docs for responsibilities and flow, and
  item docs for non-obvious invariants, errors, side effects, and access rules.
- Prepare changes on `sandbox` and sign commits that will be promoted to
  `develop` and `main`. Use plain imperative subjects such as
  `Restore release diagrams`. Omit Conventional Commit prefixes and scopes.
- In pull requests, describe the problem, the behavior change, and the checks
  run. Call out compatibility or operational effects when relevant.

## Releases

- For a new version, use `just bump`, `just bump-minor`, or `just bump-major`
  on `sandbox`. The recipe runs `just test`, changes the version, runs
  `cargo update`, and reruns `just test` before creating a signed local commit.
  Choose the next patch, minor, or major version from the highest stable tag.
- Wait for `CI OK` on `sandbox` and fast-forward `develop` to the tested SHA.
  Wait for `develop`'s own `CI OK` on that SHA. Run `just release` only when
  the operator authorizes publication.
- GitHub creates the tag when it publishes the verified draft. The recipe
  rechecks `develop` CI and fast-forwards protected `main` to the released SHA
  after the Release run succeeds. Keep published tags fixed.
- Rerun failed jobs on the original run for a fix that uses the same SHA.
  For a source fix, return to `sandbox` and qualify the new commit on both
  branches. Keep the existing version while it is untagged and eligible.
  Use the next version after tagging.
- Update the expected asset list when changing the release build matrix.
  Keep [release.yml](.github/workflows/release.yml) at its current path, which
  identifies the workflow for attestations and crates.io Trusted Publishing.
  Keep the runbook and its paired D2 and Mermaid diagrams in sync, and use
  repository-neutral adoption examples.
