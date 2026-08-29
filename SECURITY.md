# Security Policy

`tiberius-ng` is an actively-maintained community continuation of the
[tiberius](https://github.com/prisma/tiberius) TDS (Microsoft SQL Server) driver
for Rust. It is published on crates.io as the package `tiberius-ng` (the library
name remains `tiberius`). We take the security of the driver and its dependency
graph seriously and appreciate responsible disclosure.

## Supported Versions

Security fixes are provided for the current release series. Older series are not
patched; please upgrade to a supported release.

| Version   | Supported          |
| --------- | ------------------ |
| 0.13.x    | :white_check_mark: |
| < 0.13    | :x:                |

Pre-release builds (e.g. `0.13.0-alpha.N`) are covered on a best-effort basis
while the series is in development. Where practical, fixes are developed on the
`dev` branch and promoted through `qa` to `main` for release.

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues,
pull requests, or discussions.**

Instead, report them privately through GitHub Security Advisories:

- **[Open a private security advisory](https://github.com/MattJackson/tiberius-ng/security/advisories/new)**

This creates a confidential channel visible only to the maintainers, allowing us
to investigate and coordinate a fix before any public disclosure.

To help us triage quickly, please include as much of the following as you can:

- A description of the vulnerability and its potential impact.
- The affected version(s) and the platform / target (OS, TLS backend, feature
  flags such as `native-tls`, `rustls`, `integrated-auth-gssapi`).
- Steps to reproduce, ideally a minimal proof of concept.
- Any known mitigations or workarounds.

If you prefer, you may also reach the maintainer directly at
**dev@getbusbar.com** to initiate contact, but the GitHub Security Advisory is
the preferred channel for the details.

## Response Expectations

- **Acknowledgement:** we aim to acknowledge new reports within **72 hours**.
- **Assessment:** we will provide an initial assessment, including severity and
  whether we can reproduce the issue, within **7 days**.
- **Updates:** we will keep you informed of progress as we work toward a fix.
- **Disclosure:** once a fix is available, we will coordinate a release and a
  public advisory. With your permission, we are happy to credit you for the
  discovery.

These are targets rather than contractual guarantees; this is a volunteer-run
open-source project, and complex issues may take longer to resolve.

## Supply-Chain Security

Because a database driver sits on the critical path of applications that handle
sensitive data, we treat the health of our dependency graph as a first-class
security concern.

- **`cargo-deny` gate.** Every push to `main` and every pull request is checked
  with [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) via
  [`.github/workflows/security.yml`](.github/workflows/security.yml), running
  `cargo deny check advisories bans sources`.
- **Advisories.** Any known security vulnerability or **yanked** crate in the
  actually-built dependency graph fails the build (`yanked = "deny"`).
- **Scoped exceptions.** The `ignore` list in
  [`deny.toml`](deny.toml) is deliberately narrow: it contains only advisories
  that are provably **not** part of the shipped default library — reachable only
  through dev-dependencies (test harness / examples) or opt-in, non-default
  feature flags — and every entry carries a written justification.
- **Trusted sources.** Only the official crates.io registry is permitted; unknown
  registries and unknown git sources fail the build.
- **Scheduled re-checks.** The workflow also runs weekly so that newly-published
  advisories are caught even in the absence of new commits.

You can reproduce these checks locally with:

```sh
cargo deny check advisories bans sources
```

## Scope

This policy covers the `tiberius-ng` crate and its source in this repository.
Vulnerabilities in upstream dependencies should be reported to their respective
projects; if such an issue affects `tiberius-ng` users, we are glad to help
coordinate an upgrade or mitigation — please open an advisory so we can track it.
