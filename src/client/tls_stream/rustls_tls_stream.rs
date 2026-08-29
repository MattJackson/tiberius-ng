use crate::{
    client::{
        config::{ClientCertificate, Config},
        TrustConfig,
    },
    error::IoErrorKind,
    Error,
};
use futures_util::io::{AsyncRead, AsyncWrite};
use std::{
    fs, io,
    path::Path,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::SystemTime,
};
use tokio_rustls::{
    rustls::{
        client::{
            HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier,
            WantsTransparencyPolicyOrClientCert,
        },
        Certificate, ClientConfig, ConfigBuilder, DigitallySignedStruct, Error as RustlsError,
        PrivateKey, RootCertStore, ServerName, WantsVerifier,
    },
    TlsConnector,
};
use tokio_util::compat::{Compat, FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};
use tracing::{event, Level};

impl From<tokio_rustls::rustls::Error> for Error {
    fn from(e: tokio_rustls::rustls::Error) -> Self {
        crate::Error::Tls(e.to_string())
    }
}

pub(crate) struct TlsStream<S: AsyncRead + AsyncWrite + Unpin + Send>(
    Compat<tokio_rustls::client::TlsStream<Compat<S>>>,
);

struct NoCertVerifier;

impl ServerCertVerifier for NoCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, RustlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &Certificate,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }
}

fn get_server_name(config: &Config) -> crate::Result<ServerName> {
    match (ServerName::try_from(config.get_host()), &config.trust) {
        (Ok(sn), _) => Ok(sn),
        (Err(_), TrustConfig::TrustAll) => {
            Ok(ServerName::try_from("placeholder.domain.com").unwrap())
        }
        (Err(e), _) => Err(crate::Error::Tls(e.to_string())),
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send> TlsStream<S> {
    pub(super) async fn new(config: &Config, stream: S) -> crate::Result<Self> {
        event!(Level::INFO, "Performing a TLS handshake");

        let builder = ClientConfig::builder().with_safe_defaults();

        let with_roots = match &config.trust {
            TrustConfig::CaCertificateLocation(path) => {
                if let Ok(buf) = fs::read(path) {
                    let cert = match path.extension() {
                            Some(ext)
                            if ext.to_ascii_lowercase() == "pem"
                                || ext.to_ascii_lowercase() == "crt" =>
                                {
                                    let pem_cert = rustls_pemfile::certs(&mut buf.as_slice())?;
                                    if pem_cert.len() != 1 {
                                        return Err(crate::Error::Io {
                                            kind: IoErrorKind::InvalidInput,
                                            message: format!("Certificate file {} contain 0 or more than 1 certs", path.to_string_lossy()),
                                        });
                                    }

                                    Certificate(pem_cert.into_iter().next().unwrap())
                                }
                            Some(ext) if ext.to_ascii_lowercase() == "der" => {
                                Certificate(buf)
                            }
                            Some(_) | None => return Err(crate::Error::Io {
                                kind: IoErrorKind::InvalidInput,
                                message: "Provided CA certificate with unsupported file-extension! Supported types are pem, crt and der.".to_string(),
                            }),
                        };
                    let mut cert_store = RootCertStore::empty();
                    cert_store.add(&cert)?;
                    builder.with_root_certificates(cert_store)
                } else {
                    return Err(Error::Io {
                        kind: IoErrorKind::InvalidData,
                        message: "Could not read provided CA certificate!".to_string(),
                    });
                }
            }
            TrustConfig::TrustAll => {
                event!(
                    Level::WARN,
                    "Trusting the server certificate without validation."
                );
                builder.with_root_certificates(RootCertStore::empty())
            }
            TrustConfig::Default => {
                event!(Level::INFO, "Using default trust configuration.");
                builder.with_native_roots()
            }
        };

        let mut client_config = apply_client_auth(with_roots, config)?;

        if let TrustConfig::TrustAll = &config.trust {
            client_config
                .dangerous()
                .set_certificate_verifier(Arc::new(NoCertVerifier {}));
            // config.enable_sni = false;
        }

        let connector = TlsConnector::from(Arc::new(client_config));

        let tls_stream = connector
            .connect(get_server_name(config)?, stream.compat())
            .await?;

        Ok(TlsStream(tls_stream.compat()))
    }

    pub(crate) fn get_mut(&mut self) -> &mut S {
        self.0.get_mut().get_mut().0.get_mut()
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send> AsyncRead for TlsStream<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let inner = Pin::get_mut(self);
        Pin::new(&mut inner.0).poll_read(cx, buf)
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send> AsyncWrite for TlsStream<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let inner = Pin::get_mut(self);
        Pin::new(&mut inner.0).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let inner = Pin::get_mut(self);
        Pin::new(&mut inner.0).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let inner = Pin::get_mut(self);
        Pin::new(&mut inner.0).poll_close(cx)
    }
}

/// Applies the client-certificate configuration (if any) to a rustls
/// [`ClientConfig`] builder, enabling mutual TLS for the TDS 8.0
/// `ENCRYPT_CLIENT_CERT` flow.
///
/// rustls can only consume a PEM certificate chain plus a PEM private key; a
/// PKCS#12 archive is rejected with a descriptive error.
fn apply_client_auth(
    builder: ConfigBuilder<ClientConfig, WantsTransparencyPolicyOrClientCert>,
    config: &Config,
) -> crate::Result<ClientConfig> {
    match &config.client_cert {
        None => Ok(builder.with_no_client_auth()),
        Some(ClientCertificate::Pem { cert, key }) => {
            event!(
                Level::INFO,
                "Presenting a client certificate for mutual TLS."
            );
            let (chain, key) = read_pem_identity(cert, key)?;
            builder
                .with_client_auth_cert(chain, key)
                .map_err(Error::from)
        }
        Some(ClientCertificate::Pkcs12 { .. }) => Err(Error::Tls(
            "The rustls TLS backend requires a PEM client certificate; use \
             Config::client_certificate_pem instead of \
             client_certificate_pkcs12."
                .to_string(),
        )),
    }
}

/// Reads the certificate chain and private key from the given PEM files.
fn read_pem_identity(
    cert_path: &Path,
    key_path: &Path,
) -> crate::Result<(Vec<Certificate>, PrivateKey)> {
    let read = |path: &Path, kind: &str| -> crate::Result<Vec<u8>> {
        fs::read(path).map_err(|e| Error::Io {
            kind: e.kind(),
            message: format!(
                "Could not read client {} {}: {e}",
                kind,
                path.to_string_lossy()
            ),
        })
    };

    let cert_pem = read(cert_path, "certificate")?;
    let key_pem = read(key_path, "key")?;

    parse_pem_identity(&cert_pem, &key_pem)
}

/// Parses a PEM certificate chain and a PEM private key into the rustls types
/// used for client authentication. Pure helper: it performs no I/O so it can be
/// unit-tested directly.
fn parse_pem_identity(
    cert_pem: &[u8],
    key_pem: &[u8],
) -> crate::Result<(Vec<Certificate>, PrivateKey)> {
    let chain: Vec<Certificate> = rustls_pemfile::certs(&mut &cert_pem[..])?
        .into_iter()
        .map(Certificate)
        .collect();

    if chain.is_empty() {
        return Err(Error::Io {
            kind: IoErrorKind::InvalidInput,
            message: "Client certificate PEM contained no certificates.".to_string(),
        });
    }

    let key = parse_private_key(key_pem)?;

    Ok((chain, key))
}

/// Parses the first private key from a PEM buffer, accepting PKCS#8, PKCS#1
/// (RSA) and SEC1 (EC) encodings.
fn parse_private_key(key_pem: &[u8]) -> crate::Result<PrivateKey> {
    if let Some(key) = rustls_pemfile::pkcs8_private_keys(&mut &key_pem[..])?
        .into_iter()
        .next()
    {
        return Ok(PrivateKey(key));
    }

    if let Some(key) = rustls_pemfile::rsa_private_keys(&mut &key_pem[..])?
        .into_iter()
        .next()
    {
        return Ok(PrivateKey(key));
    }

    if let Some(key) = rustls_pemfile::ec_private_keys(&mut &key_pem[..])?
        .into_iter()
        .next()
    {
        return Ok(PrivateKey(key));
    }

    Err(Error::Io {
        kind: IoErrorKind::InvalidInput,
        message:
            "Client key PEM contained no supported private key (expected PKCS#8, PKCS#1 or SEC1)."
                .to_string(),
    })
}

trait ConfigBuilderExt {
    fn with_native_roots(self) -> ConfigBuilder<ClientConfig, WantsTransparencyPolicyOrClientCert>;
}

impl ConfigBuilderExt for ConfigBuilder<ClientConfig, WantsVerifier> {
    fn with_native_roots(self) -> ConfigBuilder<ClientConfig, WantsTransparencyPolicyOrClientCert> {
        let mut roots = RootCertStore::empty();
        let mut valid_count = 0;
        let mut invalid_count = 0;

        for cert in rustls_native_certs::load_native_certs().expect("could not load platform certs")
        {
            let cert = Certificate(cert.0);
            match roots.add(&cert) {
                Ok(_) => valid_count += 1,
                Err(err) => {
                    tracing::event!(Level::TRACE, "invalid cert der {:?}", cert.0);
                    tracing::event!(Level::DEBUG, "certificate parsing failed: {:?}", err);
                    invalid_count += 1
                }
            }
        }
        tracing::event!(
            Level::TRACE,
            "with_native_roots processed {} valid and {} invalid certs",
            valid_count,
            invalid_count
        );
        assert!(!roots.is_empty(), "no CA certificates found");

        self.with_root_certificates(roots)
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_pem_identity, parse_private_key};

    const TEST_CERT: &[u8] = b"-----BEGIN CERTIFICATE-----
MIIDETCCAfmgAwIBAgIUVHzRmfyATgYTXx64GgRj59ZTRAEwDQYJKoZIhvcNAQEL
BQAwGDEWMBQGA1UEAwwNdGliZXJpdXMtdGVzdDAeFw0yNjA4MjkxODA4MzBaFw0z
NjA4MjYxODA4MzBaMBgxFjAUBgNVBAMMDXRpYmVyaXVzLXRlc3QwggEiMA0GCSqG
SIb3DQEBAQUAA4IBDwAwggEKAoIBAQDB44Vm9m1prIfU4ikL+EpZxwR/0psZ1PNP
1sixSXo/EVEGFgg7a2JcQqkAImEdlubFL7ktVugdAZObcpxkwLi+WPWXLwNHW2xN
wwWglx7YPixdPxWYkLgSqSIn7w0opY4VLC1mSsmpIhDay1CfPPubN+Cz0oc6G85V
KH+mOPcyx2jRWMJRoNgdXhMXVlXBUUEOmhgTRiQZTl7ZC1y5MuQ9tE3uAjHOy1KI
0AW9c40hHWJPCxGuyn4XJTENb/L4S9XqJHgsIjvH3R3GTMSLTAEWu4EVSeIU0ilb
9TDJq65GDpdxDujtpTJ/ND9YTowAvUx01/MFd2UocQC54/xcLsVrAgMBAAGjUzBR
MB0GA1UdDgQWBBQVwYIZS+ERIsVTQGEW8pFUXKm8+jAfBgNVHSMEGDAWgBQVwYIZ
S+ERIsVTQGEW8pFUXKm8+jAPBgNVHRMBAf8EBTADAQH/MA0GCSqGSIb3DQEBCwUA
A4IBAQCTVjfE3ejpG20cqB432I6x35W9OnETemxdzfRfCSsCdR6Ie6Nlc0LAQ0yt
dQAFq98LChMttkFojI21RqiWLZt8w7qLxUs1KU/Hw6EXQwEg07ePlHqtJKi+7pBT
NHkkOVn0sC2l8ojCUJRmraOmBuZ5FhLIjDqSLcw+2gMFEw0PatICUjssgZrPERHi
Rb9t1hOYvCTMUyS7jtGaS3gy3vD3Zmt5ulHDRHeHBS5chuUbLrihvJ+iNRVQ5gSh
yodVdtL+mvukrYWp9l88ywjsp/y6He8ikUCHCvvKDZ2451ku0zcW95PBirdWR2l4
HOlVZNG7cAQzEbQ6wfNWuYdiVQbJ
-----END CERTIFICATE-----
";

    const TEST_KEY: &[u8] = b"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDB44Vm9m1prIfU
4ikL+EpZxwR/0psZ1PNP1sixSXo/EVEGFgg7a2JcQqkAImEdlubFL7ktVugdAZOb
cpxkwLi+WPWXLwNHW2xNwwWglx7YPixdPxWYkLgSqSIn7w0opY4VLC1mSsmpIhDa
y1CfPPubN+Cz0oc6G85VKH+mOPcyx2jRWMJRoNgdXhMXVlXBUUEOmhgTRiQZTl7Z
C1y5MuQ9tE3uAjHOy1KI0AW9c40hHWJPCxGuyn4XJTENb/L4S9XqJHgsIjvH3R3G
TMSLTAEWu4EVSeIU0ilb9TDJq65GDpdxDujtpTJ/ND9YTowAvUx01/MFd2UocQC5
4/xcLsVrAgMBAAECggEABkgkLFUo76w4Ame0GL/Z+NVVNKb9gMrAz1dld95U6EeG
roHMO9Cbho21ArIadLuiN8VMufFU4ouzMpggr7WmxqlZ26TESxg6hr0eKOPR4K5d
qu++TWEdoE0miXtZ8R/cCJzpI6VMqf7WjkrZY9oKn4Qh8vGL5rq75qUXQwAZfOWh
NBUVr29HvdcjUQQf1TAoU7548s+zs6qBIpgKR6DYDY/34vIsJa39dHnDXIU7+KT/
v67LwJVUvxZNiZ4ePuoTfMrR7aHjxSB0KoOhQjkTKkggeweT+T1nGwuLPxQrwTwJ
j4thnSjHlUBM2XlvLLenGmbMPk4GToZHnJI1KYiNYQKBgQD4Z9D7A7+i/DLLOVfM
gNduysYNBChtqUoQTmPsLRunTefs1l4tjmAfT62Uxr3F4eA7jzBeCO/YAIiuyCnk
DAEwRYAtCs6ZMBQ4tyVCBjM7rbm1cUAwIHqIoiawXzX/3j0QumdVe+IUusG5i+CZ
mJdjq8gNdbNztkY6Nvjngp4MEwKBgQDH0QVa5MqPaxcMDFvU44++BZnEVV4IVloN
dvsWPmRHA+PV2ZUtBT6yAGcFtPhuhpOrcxv0ljX1DFt9N2j1UbAx4FMbQF/DgVUo
N8TqOULhx8a0/u/ahsiYjcMiI6M8GeDIRV7bJ9k19QJN9sD6bRywCGkyq7RGvWjP
VKSfh+/cSQKBgQCYnegmoLn34C0w8O1BhxNVTZ3610gjf/QyKod3zosD8niA6X/5
S1VBR4nlM2nbDxjeXu4fiCwbsNBJWk9qffmo97p1cgNW2NRDuDpa40ZM70J++LKw
HvRJyB4vFIAv0RIBmhTsz20qwUdOwWLf24F/ykXiByOW/zEMiUPJsVV7IwKBgBIF
qTz8e8SZvRdqGfJGoBVcffT2WifYWgDy5UypTfQVxrvoBwtreK8nWCNsoied3b3O
AQx7a9xxQ+M0VzQhLQoimHxRvxFsHdklxo31ojGpCiQTBmEoXPldd+chXbyy/NIz
Z43Ot0mlkpKjmd48byT1bT+TuwvSU5y3nq2A3kJ5AoGBAKh8+FpQNLmdx7wEGj8o
fuygBMNCY/KAy5IjXvy6wO/zyfCOZU7G0AMIYzj/FSUBLJ04oP/+Jz59yRByria5
7TfQxCplmPvgyTFENczkqbQU83N0xBhMzCEfV97ML820OkhF12Xd7SRpRStC9TqZ
R5Fba75UfRs/SPRA9RBfBrd4
-----END PRIVATE KEY-----
";

    #[test]
    fn parses_pem_certificate_chain_and_key() {
        let (chain, _key) = parse_pem_identity(TEST_CERT, TEST_KEY)
            .expect("valid PEM certificate and key should parse");

        assert_eq!(chain.len(), 1, "expected exactly one leaf certificate");
        assert!(
            !chain[0].0.is_empty(),
            "certificate DER should be non-empty"
        );
    }

    #[test]
    fn parses_pkcs8_private_key() {
        let key = parse_private_key(TEST_KEY).expect("PKCS#8 key should parse");
        assert!(!key.0.is_empty(), "private key DER should be non-empty");
    }

    #[test]
    fn empty_certificate_chain_is_rejected() {
        let err = parse_pem_identity(b"not a certificate", TEST_KEY)
            .expect_err("a buffer with no certificates must be rejected");

        assert!(format!("{err:?}").contains("no certificates"));
    }

    #[test]
    fn missing_private_key_is_rejected() {
        let err = parse_private_key(b"-----BEGIN CERTIFICATE-----\n-----END CERTIFICATE-----\n")
            .expect_err("a buffer with no private key must be rejected");

        assert!(format!("{err:?}").contains("no supported private key"));
    }
}
