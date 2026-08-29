# Security Policy

## Supported Versions

Security updates are provided for the latest published `cron-when` release
line. Users should upgrade to the newest patch release before reporting a
problem.

| Version | Supported |
| ------- | --------- |
| 0.5.x   | ✅        |
| < 0.5   | ❌        |

## Reporting a Vulnerability

Please do not open a public GitHub issue or discussion for a suspected
security vulnerability.

Report vulnerabilities privately by emailing
[nbari@tequila.io](mailto:nbari@tequila.io). Include, when possible:

- A description of the vulnerability and its potential impact
- The affected `cron-when` version and platform
- Steps to reproduce the issue or a minimal proof of concept
- Whether the issue involves cron-expression parsing, crontab/file handling,
  terminal output, or OpenTelemetry export
- Any suggested mitigation or fix

You can expect an initial response within 48 hours and a status update within
seven days. If the report is accepted, the maintainer will coordinate a fix
and release timeline based on its severity and complexity. If it is declined,
the response will explain why it is not considered a vulnerability.

Please keep the report confidential until a fixed release is available or a
disclosure timeline has been agreed upon.

## Scope

Reports about vulnerabilities in `cron-when` itself or in the way it uses its
dependencies are in scope. General support questions and vulnerabilities that
only affect an upstream dependency should be reported to the relevant upstream
project unless `cron-when` uses the dependency in an exploitable way.
