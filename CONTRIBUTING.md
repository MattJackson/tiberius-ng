# Contributing to Tiberius

Thanks for your interest in improving Tiberius!

This repository — [`MattJackson/tiberius-ng`](https://github.com/MattJackson/tiberius-ng) —
is an actively-maintained **community fork** of the original
[`prisma/tiberius`](https://github.com/prisma/tiberius) TDS (Microsoft SQL Server)
driver for Rust. The upstream project is no longer maintained; this fork carries
the work forward. It is published on crates.io as
[`tiberius-ng`](https://crates.io/crates/tiberius-ng) (the library name stays
`tiberius`, so downstream code keeps `use tiberius::...`).

Contributions of all kinds are welcome and appreciated: bug reports, bug fixes,
new type/feature support, documentation, tests, and CI improvements. If you are
unsure whether a change fits, open an issue first to discuss it — that is never a
wasted step.

This project is dual-licensed under **MIT OR Apache-2.0**. By contributing you
agree that your contributions are licensed under the same terms.

---

## Branch lifecycle and where to target PRs

Work flows through three long-lived branches:

```
dev  ->  qa  ->  main
```

- **`dev`** — integration branch for feature work and fixes. **Target all pull
  requests at `dev`.**
- **`qa`** — promotion/staging branch where changes are validated (including the
  full integration suite against live SQL Server) before release.
- **`main`** — release branch; tagged and published to crates.io.

Please branch off `dev`, and open your PR against `dev`. Maintainers promote
`dev -> qa -> main` as changes are tested and cut into releases. PRs opened
against `qa` or `main` will normally be asked to re-target `dev`.

---

## Prerequisites

- A recent **stable** Rust toolchain (CI pins `stable`; see `rust-toolchain`).
  Install via [rustup](https://rustup.rs/).
- `rustfmt` and `clippy` components: `rustup component add rustfmt clippy`.
- For the integration suite: Docker with the Compose plugin.
- Some features need system libraries. On Debian/Ubuntu:
  `sudo apt install -y openssl libkrb5-dev` (the latter is needed for
  `integrated-auth-gssapi`).

A Nix flake (`flake.nix`) is also provided if you prefer a reproducible dev
shell.

---

## Building

```bash
# Default features (tds73, winauth, native-tls)
cargo build

# Everything the crate can do
cargo build --features=all
```

---

## Running tests

### Unit / doc tests (no database required)

Some tests and all doc examples build and run without a server:

```bash
cargo test --doc
cargo build --features=all   # make sure every feature still compiles
```

### Full integration suite (requires a live SQL Server)

Most of the test suite exercises a real server over TDS, so you need a running
SQL Server instance and the `TIBERIUS_TEST_CONNECTION_STRING` environment
variable pointing at it.

The repository ships a `docker-compose.yml` with services for SQL Server 2017,
2019, 2022, and Azure SQL Edge. Spin one up (2022 is the current primary
target):

```bash
DOCKER_BUILDKIT=1 docker compose up -d mssql-2022
```

Then export the connection string used by the tests and run them:

```bash
export TIBERIUS_TEST_CONNECTION_STRING="server=tcp:localhost,1433;user=SA;password=<YourStrong@Passw0rd>;TrustServerCertificate=true"

cargo test --features=all
```

Notes:

- The password above matches the `SA_PASSWORD` baked into the compose services;
  keep them in sync if you change one.
- The container takes a few seconds to accept connections after `up -d`. If the
  first run fails to connect, wait a moment and retry.
- Named-instance tests require the SQL Browser and the matching feature flag
  (`sql-browser-tokio` or `sql-browser-smol`); they are
  gated behind `required-features` and are skipped otherwise.
- When you are done: `docker compose down`.

CI runs the suite on Linux, Windows, and macOS across several feature
combinations (`--features=all`, `--no-default-features`, and individual
`chrono` / `time` / `rustls` / `vendored-openssl` selections) against each
supported SQL Server version. If you touch anything runtime- or type-related,
it is worth testing more than just the default feature set locally.

---

## Code style

Formatting and lints are enforced in CI and must be clean:

```bash
cargo fmt --check                       # formatting
cargo clippy --features=all -- -D warnings   # lint, warnings are errors
```

Run `cargo fmt` (without `--check`) to auto-format before committing. CI also
builds tests with `RUSTFLAGS="-Dwarnings"`, so keep the tree warning-free.

---

## Supply-chain gate (cargo-deny)

Tiberius is a foundational dependency, so the dependency graph is gated by
[cargo-deny](https://embarkstudios.github.io/cargo-deny/). The
`.github/workflows/security.yml` workflow runs this on every push, every PR, and
weekly on a schedule. It **must stay green**:

```bash
cargo install cargo-deny   # once
cargo deny check advisories bans sources
```

Policy (see `deny.toml`):

- **advisories** — any security vulnerability or yanked crate in the actually
  built graph fails the check. The `ignore` list contains only advisories that
  are provably not part of the shipped default library (dev-dependencies, or
  opt-in non-default features). Every ignore
  entry carries a written justification; if you add one, explain *why* it cannot
  affect a downstream user of the default crate.
- **bans** — duplicate versions are a warning, not a hard failure.
- **sources** — only crates.io is allowed; unknown registries and git sources
  are denied.

If your change pulls in a new dependency, run the check locally first. Prefer not
adding dependencies to the default feature set unless necessary.

---

## Feature flags

Optional functionality is gated behind Cargo features (see `[features]` in
`Cargo.toml`). Highlights:

- **`default = ["tds80", "winauth", "native-tls"]`** — the default build.
- **`all`** — turns on every feature; used for CI and docs.
- **TLS backends** — `native-tls` (default; links the OS TLS library), `rustls`
  (pure-Rust, no dynamic system dependency; recommended on Apple platforms), and
  `vendored-openssl` (statically links OpenSSL).
- **Async runtimes / SQL Browser** — `sql-browser-tokio` and
  `sql-browser-smol` enable named-instance resolution for each runtime.
- **Type support** — `chrono`, `time`, `rust_decimal`, `bigdecimal` map extra
  SQL types to those crates.
- **`tds73`** — enables TDS 7.3 protocol features (date/time types).
- **`integrated-auth-gssapi`** — Kerberos/GSSAPI integrated auth on \*nix.

When adding code behind a feature, gate it with `#[cfg(feature = "...")]`, keep
the default feature set minimal, and make sure the crate still builds both with
`--no-default-features` and with `--features=all`.

---

## Commit and PR conventions

- **Commits** — write clear, imperative-mood subject lines
  (e.g. "Fix decimal overflow when scale exceeds 28"). Conventional-commit-style
  prefixes (`fix:`, `feat:`, `chore:`, `ci:`, `test:`, `docs:`) are used in this
  repo's history and are encouraged. Keep unrelated changes in separate commits.
- **Pull requests** — target **`dev`**; give the PR a descriptive title and a
  body explaining what changed and why. Reference related issues
  (e.g. `Fixes #123`). Note any new dependencies or feature flags.
- **Before opening a PR**, make sure locally:
  - `cargo fmt --check` is clean
  - `cargo clippy --features=all -- -D warnings` is clean
  - `cargo build --features=all` succeeds
  - the integration suite passes against a local SQL Server (when your change
    affects runtime behavior)
  - `cargo deny check advisories bans sources` is green (especially if you
    changed dependencies)
- Add or update tests for behavioral changes, and update `CHANGELOG.md` and
  docs where relevant.

---

## Reporting security vulnerabilities

Please do **not** open a public issue for security vulnerabilities. Report them
privately through GitHub's private security advisories on this repository
(Security -> Advisories -> Report a vulnerability). See the repository's security
policy for details.

---

## Getting help

Open a [GitHub issue](https://github.com/MattJackson/tiberius-ng/issues) for bugs,
questions, or feature ideas. Thanks for helping keep Tiberius healthy and
maintained!
