# Tiberius (tiberius-ng)
[![crates.io](https://img.shields.io/crates/v/tiberius-ng.svg)](https://crates.io/crates/tiberius-ng)
[![docs.rs](https://docs.rs/tiberius-ng/badge.svg)](https://docs.rs/tiberius-ng)
[![Downloads](https://img.shields.io/crates/d/tiberius-ng.svg)](https://crates.io/crates/tiberius-ng)
[![License](https://img.shields.io/crates/l/tiberius-ng.svg)](#license)
[![Cargo tests](https://github.com/MattJackson/tiberius/actions/workflows/test.yml/badge.svg?branch=dev)](https://github.com/MattJackson/tiberius/actions/workflows/test.yml)
[![Security audit](https://github.com/MattJackson/tiberius/actions/workflows/security.yml/badge.svg?branch=dev)](https://github.com/MattJackson/tiberius/actions/workflows/security.yml)
[![codecov](https://codecov.io/gh/MattJackson/tiberius/branch/dev/graph/badge.svg)](https://codecov.io/gh/MattJackson/tiberius)
[![PRs welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

A native Microsoft SQL Server (TDS) client for Rust.

> ## ⚑ Actively maintained fork
>
> This is a **community continuation of [`tiberius`](https://github.com/prisma/tiberius)**, which
> the original authors ([Prisma](https://github.com/prisma/tiberius)) no longer maintain. This fork
> is **maintained full-time** at [**MattJackson/tiberius**](https://github.com/MattJackson/tiberius):
> we are working through the backlog of open PRs and issues, keeping dependencies current and the
> crate free of security advisories, and cutting regular releases.
>
> Because the `tiberius` name on crates.io belongs to the original project, releases are published
> under the package name **[`tiberius-ng`](https://crates.io/crates/tiberius-ng)**. The library name
> is still `tiberius`, so your `use tiberius::…` code does not change — see
> [Installation](#installation) below. Contributions, bug reports and feature requests are welcome
> [in the issue tracker](https://github.com/MattJackson/tiberius/issues).

## Installation

Add the dependency under its published name, `tiberius-ng`, while keeping the crate importable as
`tiberius`:

```toml
[dependencies]
tiberius = { package = "tiberius-ng", version = "0.13" }
```

Your code is unchanged:

```rust
use tiberius::{Client, Config, AuthMethod};
```

If you prefer, you can instead depend on it by its real name and import it as `tiberius_ng`:

```toml
[dependencies]
tiberius-ng = "0.13"
```

### Goals

- A perfect implementation of the TDS protocol.
- Asynchronous network IO.
- Independent of the network protocol.
- Support for latest versions of Linux, Windows and macOS.

### Non-goals

- Connection pooling (use [bb8](https://crates.io/crates/bb8), [mobc](https://crates.io/crates/mobc), [deadpool](https://crates.io/crates/deadpool) or any of the other asynchronous connection pools)
- Query building
- Object-relational mapping

### Supported SQL Server versions

| Version | Support level | Notes                               |
|---------|---------------|-------------------------------------|
|    2022 | Tested on CI  |                                     |
|    2019 | Tested on CI  |                                     |
|    2017 | Tested on CI  |                                     |
|    2016 | Should work   |                                     |
|    2014 | Should work   |                                     |
|    2012 | Should work   |                                     |
|    2008 | Should work   |                                     |
|    2005 | Should work   | With feature flag `tds73` disabled. |

### Feature flags

| Flag                     | Description                                                                                                                      | Default    |
|--------------------------|----------------------------------------------------------------------------------------------------------------------------------|------------|
| `tds73`                  | Support for new date and time types in TDS version 7.3. Disable if using version 7.2.                                            | `enabled`  |
| `native-tls`             | Use operating system's TLS libraries for traffic encryption.                                                                     | `enabled`  |
| `rustls`                 | Use the builtin TLS implementation from rustls instead of linking to the operating system implementation for traffic encryption. | `disabled` |
| `vendored-openssl`       | Statically link against OpenSSL instead of dynamically linking to the operating system implementation for traffic encryption.    | `disabled` |
| `chrono`                 | Read and write date and time values using `chrono`'s types. (for greenfield, using time instead of chrono is recommended)        | `disabled` |
| `time`                   | Read and write date and time values using `time` crate types.                                                                    | `disabled` |
| `rust_decimal`           | Read and write `numeric`/`decimal` values using `rust_decimal`'s `Decimal`.                                                      | `disabled` |
| `bigdecimal`             | Read and write `numeric`/`decimal` values using `bigdecimal`'s `BigDecimal`.                                                     | `disabled` |
| `sql-browser-async-std`  | SQL Browser implementation for the `TcpStream` of async-std.                                                                     | `disabled` |
| `sql-browser-tokio`      | SQL Browser implementation for the `TcpStream` of Tokio.                                                                         | `disabled` |
| `sql-browser-smol`       | SQL Browser implementation for the `TcpStream` of smol.                                                                          | `disabled` |
| `integrated-auth-gssapi` | Support for using Integrated Auth via GSSAPI                                                                                     | `disabled` |

### Supported protocols

Tiberius does not rely on any protocol when connecting to an SQL Server instance. Instead the `Client` takes a socket that implements the `AsyncRead` and `AsyncWrite` traits from the [futures-rs](https://crates.io/crates/futures) crate.

Currently there are good async implementations for TCP in the [async-std](https://crates.io/crates/async-std), [Tokio](https://crates.io/crates/tokio) and [Smol](https://crates.io/crates/smol) projects.

To be able to use them together with Tiberius on Windows platforms with SQL Server, you should make sure that the TCP protocol is enabled, as depending on the edition, this may not be the case. Standard and Enterprise editions will have the setting enabled by default, whereas Developer, Express editions and the Windows Internal Database feature of the Windows Server OS don't.
To enable the TCP/IP protocol you may want to use  the [server settings](https://docs.microsoft.com/en-us/sql/database-engine/configure-windows/enable-or-disable-a-server-network-protocol) the [command line](https://docs.microsoft.com/en-us/sql/powershell/how-to-enable-tcp-sqlps).
In the official [Docker image](https://hub.docker.com/_/microsoft-mssql-server) TCP is is enabled by default.

Named pipes should work by using the [NamedPipeClient](https://docs.rs/tokio/1.9.0/tokio/net/windows/named_pipe/struct.NamedPipeClient.html) from the latest Tokio versions.

The shared memory protocol is not documented and seems there are no Rust crates implementing it.

### Encryption (TLS/SSL)

Tiberius can be set to use two different implementations of TLS connection encryption. By default it uses `native-tls`, linking to the TLS library provided by the operating system. This is a good practice and in case of security vulnerabilities, upgrading the system libraries fixes the vulnerability in Tiberius without a recompilation. On Linux we link against OpenSSL, on Windows against schannel and on macOS against Security Framework.

Alternatively one can use the `rustls` feature flag to use the Rust native TLS implementation. This way there are no dynamic dependencies to the system. This might be useful in certain installations, but requires a rebuild to update to a new TLS version. For some reasons the Security Framework on macOS does not work with SQL Server TLS settings, and on Apple platforms if needing TLS it is recommended to use `rustls` instead of `native-tls`. The other option is to use the `vendored-openssl` feature flag, that statically links against the latest OpenSSL implementation.

The crate can also be compiled without TLS support, but not with both features enabled at the same time.

Tiberius has three runtime encryption settings:

| Encryption level | Description                                      |
|------------------|--------------------------------------------------|
| `Required`       | All traffic is encrypted. (default)              |
| `Off`            | Only the login procedure is encrypted.           |
| `NotSupported`   | None of the traffic is encrypted.                |

The encryption levels can be set when connecting to the database.

### Integrated Authentication (TrustedConnection) on \*nix

With the `integrated-auth-gssapi` feature enabled, the crate requires the GSSAPI/Kerberos libraries/headers installed:
  * [CentOS](https://pkgs.org/download/krb5-devel)
  * [Arch](https://www.archlinux.org/packages/core/x86_64/krb5/)
  * [Debian](https://tracker.debian.org/pkg/krb5) (you need the -dev packages to build)
  * [Ubuntu](https://packages.ubuntu.com/bionic-updates/libkrb5-dev)
  * NixOS: Run `nix-shell shell.nix` on the repository root.
  * Mac: as of version `0.4.2` the [libgssapi](https://crates.io/crates/libgssapi) crate used for this feature now uses Apple's [GSS Framework](https://developer.apple.com/documentation/gss?language=objc) which ships with MacOS 10.14+.

Additionally, your runtime system will need to be trusted by and configured for the Active Directory domain your SQL Server is part of. In particular, you'll need to be able to get a valid TGT for your identity, via `kinit` or a keytab. This setup varies by environment and OS, but your friendly network/system administrator should be able to help figure out the specifics.

## Redirects

With certain Azure firewall settings, a login might return `Error::Routing { host, port }`. This means the user must create a new `TcpStream` to the given address, and connect again.

A simple connection procedure would then be:

```rust
use tiberius::{Client, Config, AuthMethod, error::Error};
use tokio_util::compat::TokioAsyncWriteCompatExt;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new();

    config.host("0.0.0.0");
    config.port(1433);
    config.authentication(AuthMethod::sql_server("SA", "<Mys3cureP4ssW0rD>"));

    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let client = match Client::connect(config, tcp.compat_write()).await {
        // Connection successful.
        Ok(client) => client,
        // The server wants us to redirect to a different address
        Err(Error::Routing { host, port }) => {
            let mut config = Config::new();

            config.host(&host);
            config.port(port);
            config.authentication(AuthMethod::sql_server("SA", "<Mys3cureP4ssW0rD>"));

            let tcp = TcpStream::connect(config.get_addr()).await?;
            tcp.set_nodelay(true)?;

            // we should not have more than one redirect, so we'll short-circuit here.
            Client::connect(config, tcp.compat_write()).await?
        }
        Err(e) => Err(e)?,
    };

    Ok(())
}
```

## Security

Supply-chain security is enforced in CI via [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny)
(see [`deny.toml`](./deny.toml) and the [Security audit workflow](.github/workflows/security.yml)): any
vulnerability or yanked crate in the built dependency graph fails the build.

To report a security vulnerability, please use
[GitHub's private security advisory reporting](https://github.com/MattJackson/tiberius/security/advisories/new)
for this repository rather than opening a public issue.

## Contributing

Contributions are very welcome — this is a community-maintained project. Please
read [CONTRIBUTING.md](CONTRIBUTING.md) and the [Code of Conduct](CODE_OF_CONDUCT.md)
before opening a pull request, and target PRs at the `dev` branch (see the
`dev → qa → main` lifecycle in CONTRIBUTING). Good first issues and help-wanted
items are tracked in the [issue tracker](https://github.com/MattJackson/tiberius/issues).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE.txt) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT.txt) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
