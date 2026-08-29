use crate::{
    client::{
        config::{ClientCertificate, Config},
        TrustConfig,
    },
    error::{Error, IoErrorKind},
};
pub(crate) use async_native_tls::TlsStream;
use async_native_tls::{Certificate, Identity, TlsConnector};
use futures_util::io::{AsyncRead, AsyncWrite};
use std::fs;
use tracing::{event, Level};

/// Reads the file at `path`, mapping any I/O failure onto a descriptive
/// [`Error::Io`].
fn read_file(path: &std::path::Path, kind: &str) -> crate::Result<Vec<u8>> {
    fs::read(path).map_err(|e| Error::Io {
        kind: e.kind(),
        message: format!(
            "Could not read client {} {}: {e}",
            kind,
            path.to_string_lossy()
        ),
    })
}

/// Builds a native-tls [`Identity`] from the configured client certificate so
/// it can be presented to the server for mutual TLS (TDS 8.0
/// `ENCRYPT_CLIENT_CERT`).
fn client_identity(cert: &ClientCertificate) -> crate::Result<Identity> {
    match cert {
        ClientCertificate::Pkcs12 { path, password } => {
            let der = read_file(path, "certificate")?;
            Ok(Identity::from_pkcs12(&der, password)?)
        }
        ClientCertificate::Pem { cert, key } => {
            let cert_pem = read_file(cert, "certificate")?;
            let key_pem = read_file(key, "key")?;
            Ok(Identity::from_pkcs8(&cert_pem, &key_pem)?)
        }
    }
}

pub(crate) async fn create_tls_stream<S: AsyncRead + AsyncWrite + Unpin + Send>(
    config: &Config,
    stream: S,
) -> crate::Result<TlsStream<S>> {
    let mut builder = TlsConnector::new();

    match &config.trust {
        TrustConfig::CaCertificateLocation(path) => {
            if let Ok(buf) = fs::read(path) {
                let cert = match path.extension() {
                        Some(ext)
                        if ext.to_ascii_lowercase() == "pem"
                            || ext.to_ascii_lowercase() == "crt" =>
                            {
                                Some(Certificate::from_pem(&buf)?)
                            }
                        Some(ext) if ext.to_ascii_lowercase() == "der" => {
                            Some(Certificate::from_der(&buf)?)
                        }
                        Some(_) | None => return Err(Error::Io {
                            kind: IoErrorKind::InvalidInput,
                            message: "Provided CA certificate with unsupported file-extension! Supported types are pem, crt and der.".to_string()}),
                    };
                if let Some(c) = cert {
                    builder = builder.add_root_certificate(c);
                }
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

            builder = builder.danger_accept_invalid_certs(true);
            builder = builder.danger_accept_invalid_hostnames(true);
            builder = builder.use_sni(false);
        }
        TrustConfig::Default => {
            event!(Level::INFO, "Using default trust configuration.");
        }
    }

    if let Some(cert) = &config.client_cert {
        event!(
            Level::INFO,
            "Presenting a client certificate for mutual TLS."
        );
        builder = builder.identity(client_identity(cert)?);
    }

    Ok(builder.connect(config.get_host(), stream).await?)
}
