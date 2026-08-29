# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

This is a maintained community fork of [`tiberius`](https://github.com/prisma/tiberius),
published on crates.io under the package name
[`tiberius-ng`](https://crates.io/crates/tiberius-ng). The library name remains
`tiberius`, so `use tiberius::…` code is unaffected. Entries up to and including
version 0.12.3 reflect the history of the upstream project prior to the fork.

## [Unreleased]

We are working through the backlog of open pull requests and issues from
upstream. Additional fixes and features are being integrated and will appear
here as they land.

### Added

- (Backlog) Further community PRs and issue fixes are being triaged and
  integrated; see the [issue tracker](https://github.com/MattJackson/tiberius/issues).

### Fixed

- (Backlog) Outstanding bug reports carried over from upstream are being
  reviewed and resolved.

### Changed

- (Backlog) Ongoing dependency maintenance to keep the crate free of security
  advisories.

## [0.13.0] - 2026-08-29

First maintained release of the fork. Published on crates.io as
[`tiberius-ng`](https://crates.io/crates/tiberius-ng); the importable library
name is still `tiberius`.

### Added

- **cargo-deny supply-chain gate.** Added `deny.toml` and a
  `.github/workflows/security.yml` workflow that runs `cargo deny` to check for
  security advisories, banned/duplicate dependencies, and license compliance on
  every push and pull request.
- PR-triggered code security workflow (upstream #409).

### Changed

- **Rebrand to a maintained fork.** The crate is now published as
  `tiberius-ng` while keeping `tiberius` as the library name, so downstream
  `use tiberius::…` code needs no changes. Documentation, badges, and metadata
  updated to point at [MattJackson/tiberius](https://github.com/MattJackson/tiberius).
- **Modernized CI.** Reworked the GitHub Actions test workflow and refreshed
  development dependencies.
- Bumped `azure_identity` from 0.5.0 to 0.20.0.
- Bumped `libgssapi` to 0.8.1 (#372).
- Avoid unnecessary `Vec` reallocations when a row contains more than four
  elements (#370).
- Removed the top-level `SECURITY.md` in favor of the `.github` policy (#406).

### Fixed

- **Security: upgraded the TLS stack to clear known advisories.** Bumped
  `tokio-rustls` 0.24 → 0.26 and `rustls` 0.21 → 0.23, resolving
  RUSTSEC-2026-0098, RUSTSEC-2026-0099, and RUSTSEC-2026-0104. This addresses
  the concern raised in upstream #419.
- **Renewed expired test certificates** so the integration test suite runs
  against a live SQL Server again.
- Ported stranded `async-std` tests to `#[test_on_runtimes]` so they run under
  the supported runtimes.

### Notes

- `0.13.0-alpha.1` was published to crates.io to claim the `tiberius-ng`
  package name ahead of the first full release.

## [0.12.3]

- feat: improve column type accuracy (#347)
- fix: encoding of zero-length values for large varlen columns (#315)
- update tokio_rustls (#306)
- Allow iterating over the cells in a row. (#303)
- Send ReadOnlyIntent when ApplicationIntent=ReadOnly specified (#297)
- Replace encoding with encoding_rs (#285)
- Disable chrono's oldtime feature (#284)

## [0.12.2]

- Update connection-string crate to 0.2 (#286)

## [0.12.1]

- fix: bigdecimal conversion overflow (#271)
- Reduce futures crate dependency footprint (#270)

## [0.12.0]

- BREAKING: Correctly convert DateTimeOffset to/from database (#269)
  Please read the [issue](https://github.com/prisma/tiberius/issues/260)
  carefully before upgrading.

## [0.11.8]

- feat: improve column type info (#347)

## [0.11.7]

- chore: Update connection string to 0.2 (#286)

## [0.11.6]

- fix: bigdecimal conversion overflow (#271)

## [0.11.5]

- Close connection explicitly (#268)

## [0.11.4]

- Fix buffer overrun on finalize (#266)
- Correctly parse (local) server name (#259)

## [0.11.3]

- Cleanup TokenRow public API (#255)
- Fix null values in NBC rows (#253)

## [0.11.2]

- Fix error ordering (#248)

## [0.11.1]

- Don't load native roots for trust-all config (#243)
- Propagate errors correctly (#247)

## [0.11.0]

- BREAKING: bigdecimal crate upgraded to 0.3 major and has to be of
  the same major in other crates using Tiberius.
- Handle negative scale from a BigDecimal (#240)

## [0.10.0]

- BREAKING: uuid crate upgraded to 1.0 major and has to be of the same
  major in other crates using Tiberius.

## [0.9.5]

- Add fractional seconds precision for datetime2 (#235)

## [0.9.4]

- Fix SQL Browser response parsing error (#229)
- Bulk uploads (#227)

## [0.9.3]

- Enable SSL if using vendored-openssl feature (#225)

## [0.9.2]

- Allow statically linking against OpenSSL (#222)

## [0.9.1]

- Support AAD token authentication (#215)

## [0.9.0]

- (BREAKING) support rustls, switch between native-tls and rustls.
  the feature flag vendored-openssl is gone. instead if needing vendored TLS,
  use feature flag rustls

## [0.8.0]

- (BREAKING) fix: correctly decode null integers (#209)

## [0.7.3]

- Fixing an accidentally renamed time module, that would've been a breaking change.

## [0.7.2]

- Dynamic query interface (#196)
- Support for `time` 0.3.x (#201)
- Additional option to add custom-ca to root certificates (#203, thx @lostiniceland)

## [0.7.1]

- Support all pre-login tokens

## [0.7.0]

- Remove async-std from deps if using tokio
- show TokioAsyncWriteCompatExt in Client docs (#183)
- Upgrade to Rust edition 2021 (#180)

## [0.6.5]

- Constrain UUID features and optionalize winauth dependency (smaller binaries)

## [0.6.4]

- Use bundled bigint from bigdecimal

## [0.6.3]

- Bignum/bigint compilation problems fixed.

## [0.6.2]

- Improvement on waker calls. We used to wake the runtime too often, this should improve performance.

## [0.6.1]

- SQL Browser for the smol runtime.

## [0.6.0]

- Refactor stream handling to something more rusty (#166). This is a breaking
  change, if relying on the asynchronous stream handling of QueryResult. Please
  refer to the updated documentation.

## [0.5.16]

- Allow setting application name per connection (#161)

## [0.5.15]

- Split column decoding into modules (speeding up TEXT/NTEXT/IMAGE decoding a lot) (#153)

## [0.5.14]

- Handle collations for CHAR and TEXT values (#153)

## [0.5.13]

- Add Config parsing for "Integrated Security" (two words)
- Unified bitflag setup
- Correct default ports
- Update to enumflags2 0.7

## [0.5.12]

- Warnings should not affect metadata fetching (#139)

## [0.5.11]

- Fixing of all clippy warnings. This might have some performance benefits and
  might also fix some weird bugs in environments where we can't guarantee the
  evaluation order. (#136)
- Add info of LCID and sort id to colation errors (#138)

## [0.5.10]

- Remove a rogue `dbg!`

## [0.5.9]

- Set the `app_name` in LOGIN7 to `tiberius`. This allows connecting to servers
  that expect the value to not be empty (see issue #127).

## [0.5.8]

- Try out all resolved IP addresses (#124)

## [0.5.7]

- Set server name in the login packet (#122)

## [0.5.6]

- Fix for handling nullable values (#119 #121)

## [0.5.5] and [0.4.21]

Catastropichal build failures with feature flags fixed.

## [0.5.4] and [0.4.20]

Removed the tls feature flag to simplify dependencies. This means you will
always get a TLS-enabled build, and can disable it on runtime. This also means
we don't always compile async-std if wanting to use tokio, and so forth.

Fixes certain issues with vendored OpenSSL on macOS platforms too.

## [0.5.3]

Changed futures-codec2 to asynchronous-codec, due to former was yanked.

## [0.5.2] and [0.4.19]

Introducing working TLS support on macOS platforms.

Please read the issue:

https://github.com/prisma/tiberius/issues/65

## [0.5.1]

Internally upgrade bytes to 1.0. Should have no visible change to the apis.

## [0.5.0]

If using Tiberius with Tokio and SQL Browser, this PR will upgrade Tokio to 1.0.

0.4 branch will be updated for a short while if needed and until the ecosystem
has completely settled on Tokio 1.0.

## [0.4.18]

- Allow `databaseName` in connection string to define the database (#108)
- Implement reader functions for standard string data (#107)
- Fix a time conversion error (#106)

## [0.4.17]

- Fixing error swallowing with `simple_query` and MARS (#105)
- Fixing transaction descriptor reading (#105)
- Fixing envchange token reads (#105)

## [0.4.16]

- Handle all MARS results properly (#102)

## [0.4.14]

- Support alternatively `BigNumber` when dealing with numeric values.
- Document feature flags

## [0.4.13]

- Realizing UTF-16 works just fine with SQL Server. Reverting the UCS2, but
  still keeping the faster writes.

## [0.4.12]

*SKIP this, go directly to 0.4.13*

- A typo fix in README (#94)
- Faster string writes with better length handling. UCS2 for writes (#95).

## [0.4.11]

- Allow disabling TLS in connection string (#89)
- Use connection-string for ado.net parsing (#91)
- Handle JDBC connection strings (#92)

## [0.4.10]

- Handling nullable int values, fix for #78 (#80)
- Reflect tweaks to upstream libgssapi crate (#81)
- Skip default features in libgssapi (for macOS support)
- Handle env change Routing request (#87)

## [0.4.9]

- BREAKING: `AuthMethod::WindowsIntegrated` renamed to `AuthMethod::Integrated`.
- Use GSSAPI for IntegratedSecurity on Unix platforms
- Fix module docs for examples
- Make `packet_id` wrapping explicit
- Add DNS feature to Tokio

## [0.4.8]

- BREAKING: `ColumnData::I8(i8)` is now `ColumnData::U8(u8)` due to misunderstanding how `tinyint` works. (#71)
- Skip any received `done_rows` amounts and avoid creating extra resultsets (#67)
- Actually run the chrono tests (#72)
- Fix GUID byte ordering (#69)
- Fix null time/datetime2/datetimeoffset handling (#73)
- Null image data should be `Binary`, not `String`

## [0.4.7]

- Pass hostname to TLS handshake, allowing usage with AzureSQL using
  `TrustServerCertificate=no`
  ([#62](https://github.com/prisma/tiberius/pull/62))

## [0.4.5]

- Documenting type conversions and re-exporting chrono types
  ([#60](https://github.com/prisma/tiberius/pull/60))

## [0.4.4]

- Fixing multi-part table names in `IMAGE`, `TEXT` and `NTEXT` column metadata
  ([#58](https://github.com/prisma/tiberius/pull/58))

## [0.4.3]

- Starting transactions with `simple_query` now works without crashing
  ([#55](https://github.com/prisma/tiberius/pull/55))

## [0.4.2]

- Fixing old and wrong `ExecuteResult` docs
- Adding `rows_affected` method to `ExecuteResult`

## [0.4.1]

- Add all feature flags for docs.rs build

## [0.4.0]

- A complete rewrite from 0.3.0
- Not bound to Tokio anymore, independent of the runtime
- Support for many more types
- Async/await, futures 0.3

[Unreleased]: https://github.com/MattJackson/tiberius/compare/v0.13.0...HEAD
[0.13.0]: https://github.com/MattJackson/tiberius/compare/v0.12.3...v0.13.0
</content>
</invoke>
