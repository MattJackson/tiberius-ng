<h1 align="center">Tiberius</h1>

<p align="center">
  <b>A modern, async, pure-Rust Microsoft SQL Server (TDS) driver.</b><br>
  Runtime-independent · TLS-first · TDS 7.2 → 8.0.
</p>

<p align="center">
  <a href="https://crates.io/crates/tiberius-ng"><img src="https://img.shields.io/crates/v/tiberius-ng.svg" alt="crates.io"></a>
  <a href="https://docs.rs/tiberius-ng"><img src="https://docs.rs/tiberius-ng/badge.svg" alt="docs.rs"></a>
  <a href="https://crates.io/crates/tiberius-ng"><img src="https://img.shields.io/crates/d/tiberius-ng.svg" alt="downloads"></a>
  <a href="#license"><img src="https://img.shields.io/crates/l/tiberius-ng.svg" alt="license"></a>
  <br>
  <a href="https://github.com/MattJackson/tiberius-ng/actions/workflows/test.yml"><img src="https://github.com/MattJackson/tiberius-ng/actions/workflows/test.yml/badge.svg?branch=dev" alt="tests"></a>
  <a href="https://github.com/MattJackson/tiberius-ng/actions/workflows/security.yml"><img src="https://github.com/MattJackson/tiberius-ng/actions/workflows/security.yml/badge.svg?branch=dev" alt="security audit"></a>
  <a href="https://codecov.io/gh/MattJackson/tiberius-ng"><img src="https://codecov.io/gh/MattJackson/tiberius-ng/branch/dev/graph/badge.svg" alt="coverage"></a>
  <a href="CONTRIBUTING.md"><img src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg" alt="PRs welcome"></a>
</p>

---

Tiberius speaks the TDS protocol directly, so you can talk to **Microsoft SQL
Server** and **Azure SQL** from Rust on Linux, macOS and Windows — with no ODBC,
no FreeTDS, and no C toolchain in the default build. It is not tied to any async
runtime: give it anything implementing `AsyncRead + AsyncWrite` (Tokio or
smol) and it does the rest.

> ### ℹ️ About this project
>
> This is the **actively-maintained continuation** of the original
> [`tiberius`](https://github.com/prisma/tiberius), which is no longer maintained
> upstream. It is published on crates.io as **`tiberius-ng`** (the `tiberius`
> name belongs to the original project) — but the **library is still imported as
> `tiberius`**, so migrating is a one-line change. See [Installation](#installation).

## Highlights

- 🔌 **Runtime-agnostic** — works with Tokio and smol (any `AsyncRead + AsyncWrite`).
- 🔐 **TLS-first, incl. TDS 8.0 "strict"** — `native-tls`, `rustls`, or vendored
  OpenSSL; `hostname_in_certificate`, ALPN `tds/8.0`, and mutual-TLS client
  certificates.
- 🪪 **Every auth method** — SQL logins (password buffers zeroized), Windows
  integrated auth (SSPI on Windows **and NTLM on Linux/macOS without Kerberos**),
  Kerberos/GSSAPI, and Azure AD tokens.
- 🧱 **Rich data access** — typed rows, `chrono`/`time`/`rust_decimal`/`bigdecimal`,
  optional `serde`, streaming results, and column metadata (type, size, scale,
  nullability, identity).
- ⚡ **Real workloads** — bulk insert (whole-table or a column list), stored
  procedures with **named parameters, OUT params and table-valued parameters**,
  transactions, `IN (…)` list helpers, `MultiSubnetFailover`, and query
  cancellation.
- 🛡️ **Secure & audited** — CI enforces `cargo-deny` (zero advisories), and every
  release is cut from a fully green pipeline.

## Installation

The published crate is `tiberius-ng`; the library name stays `tiberius`:

```toml
[dependencies]
tiberius = { package = "tiberius-ng", version = "0.13" }
```

Your code imports it as usual — **no source changes**:

```rust
use tiberius::{Client, Config, AuthMethod};
```

<details>
<summary>Migrating from the original <code>tiberius</code>?</summary>

Replace `tiberius = "0.12"` with the line above. The API is compatible and the
import path is unchanged; you also gain the fixes and features in the
[changelog](CHANGELOG.md).
</details>

## Quick start

```rust
use tiberius::{Client, Config, AuthMethod};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut config = Config::new();
    config.host("localhost");
    config.port(1433);
    config.authentication(AuthMethod::sql_server("SA", "<YourStrong@Passw0rd>"));
    config.trust_cert(); // don't do this in production

    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let mut client = Client::connect(config, tcp.compat_write()).await?;

    let row = client
        .query("SELECT @P1 AS greeting", &[&"hello world"])
        .await?
        .into_row()
        .await?
        .unwrap();

    assert_eq!(Some("hello world"), row.get("greeting"));
    Ok(())
}
```

Prefer a connection string? `Config::from_ado_string("Server=tcp:localhost,1433;User Id=SA;Password=…;Encrypt=strict;")`
and `Config::from_jdbc_string(…)` are both supported, or build one fluently with
[`ConfigBuilder`](https://docs.rs/tiberius-ng/latest/tiberius/struct.ConfigBuilder.html).

## Documentation

- **[Guide](docs/GUIDE.md)** — a practical tour: connecting, config, TLS, auth,
  querying, bulk insert, stored procedures/TVPs, transactions, and more.
- **[API docs](https://docs.rs/tiberius-ng)** — the full reference on docs.rs.
- **[`examples/`](examples/)** — runnable examples (tokio, bulk, named pipes, AAD).
- **[TDS compatibility](docs/TDS_COMPATIBILITY.md)** — the protocol coverage matrix.

## Feature flags

| Flag | Purpose | Default |
|---|---|:---:|
| `tds73` | TDS 7.3 date/time types (`date`, `time`, `datetime2`, `datetimeoffset`) | ✅ |
| `tds80` | TDS 8.0 support incl. `EncryptionLevel::Strict` (requires a TLS backend) | ✅ |
| `native-tls` | Encryption via the OS TLS stack (schannel / Secure Transport / OpenSSL) | ✅ |
| `rustls` | Pure-Rust TLS via `rustls` (recommended on macOS/Apple) | |
| `vendored-openssl` | Statically-linked OpenSSL | |
| `chrono` / `time` | Date-time values via `chrono` / the `time` crate | |
| `rust_decimal` / `bigdecimal` | `numeric`/`decimal` via `Decimal` / `BigDecimal` | |
| `serde` | `Serialize`/`Deserialize` for result types | |
| `sql-browser-tokio` / `-smol` | Resolve named instances via SQL Browser | |
| `integrated-auth-gssapi` | Kerberos/GSSAPI integrated auth (Unix) | |
| `sspi-rs` | Windows-style NTLM auth on Linux/macOS without Kerberos | |

The three TLS backends are mutually exclusive. See the [docs](https://docs.rs/tiberius-ng)
for the full API.

## Runtimes

Tiberius takes any `AsyncRead + AsyncWrite` socket, so it runs under:

- **Tokio** — wrap the stream with `tokio_util::compat`.
- **smol** — pass the `TcpStream` directly.

Connection pooling is intentionally out of scope — use
[`bb8`](https://crates.io/crates/bb8), [`deadpool`](https://crates.io/crates/deadpool)
or [`mobc`](https://crates.io/crates/mobc).

## Encryption & authentication

Encryption levels: `NotSupported`, `Off`, `Required` (default), and **`Strict`**
(TDS 8.0, TLS before the pre-login). Validate against a specific certificate name
with `Config::hostname_in_certificate(…)`, and present a client certificate for
mutual TLS with `Config::client_certificate(…)`.

Authentication methods: `AuthMethod::sql_server(...)`, `AuthMethod::windows(...)`
(SSPI on Windows, `sspi-rs` NTLM on Unix), `AuthMethod::Integrated` (Kerberos on
Unix with `integrated-auth-gssapi`), and `AuthMethod::aad_token(...)` for Azure AD.

## Compatibility

| SQL Server | Status |
|---|---|
| 2022, 2019, 2017 | Tested in CI (Linux containers) |
| Azure SQL Database / Managed Instance | Supported (`Encrypt=strict` / AAD) |
| Azure SQL Edge | Tested in CI |
| 2016 → 2005 | Supported via the TDS protocol |

Protocol coverage spans **TDS 7.2 through 8.0** — see
[`docs/TDS_COMPATIBILITY.md`](docs/TDS_COMPATIBILITY.md) for the full
message/token/type matrix.

## Testing

Unit tests run with `cargo test`. Integration tests need a live server; the
easiest way is the bundled helper:

```bash
./docker/test-server.sh              # starts SQL Server (docker or podman)
export TIBERIUS_TEST_CONNECTION_STRING="server=tcp:localhost,1433;user=SA;password=<YourStrong@Passw0rd>;TrustServerCertificate=true"
cargo test --features all
```

CI follows the `dev → qa → main` lifecycle:

- **PRs and `dev`** run a fast lane — lint, unit tests, and one integration smoke
  (SQL Server 2022) — so day-to-day iteration stays quick.
- **`qa`** runs the full UAT: the integration suite against SQL Server
  2017 / 2019 / 2022 / 2025 and Azure SQL Edge across every feature combination,
  plus Windows (integrated auth) and macOS builds. A green `qa` means the release
  is ready.
- **`main`** is a fast promotion from an already-green `qa`; tagging `v*` triggers
  the release workflow (verify → publish to crates.io → GitHub release).

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## Contributing

Contributions are very welcome — see [CONTRIBUTING.md](CONTRIBUTING.md) and the
[Code of Conduct](CODE_OF_CONDUCT.md). Target PRs at the `dev` branch. Bugs and
feature requests go in the [issue tracker](https://github.com/MattJackson/tiberius-ng/issues).

## Security

Supply-chain security is enforced in CI with
[`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) (see [`deny.toml`](deny.toml)):
any vulnerability or yanked crate in the built graph fails the build. Report
vulnerabilities privately via
[GitHub Security Advisories](https://github.com/MattJackson/tiberius-ng/security/advisories/new).

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE.txt) or [MIT](LICENSE-MIT.txt)
at your option. Unless you explicitly state otherwise, any contribution you submit
shall be dual-licensed as above, without additional terms.

## Acknowledgements

Built on the excellent foundation of the original
[`tiberius`](https://github.com/prisma/tiberius) by Prisma and its contributors.
This fork carries that work forward — thank you to everyone who has contributed
patches, past and present.
