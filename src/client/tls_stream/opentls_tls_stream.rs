use crate::{
    client::{
        config::{ClientCertificate, Config},
        TrustConfig,
    },
    error::{Error, IoErrorKind},
};
use futures_util::io::{AsyncRead, AsyncWrite};
pub(crate) use opentls::async_io::{TlsConnector, TlsStream};
use opentls::{Certificate, Identity};
use std::fs;
use tracing::{event, Level};

/// Builds an opentls [`Identity`] from the configured client certificate so it
/// can be presented to the server for mutual TLS (TDS 8.0
/// `ENCRYPT_CLIENT_CERT`).
///
/// The OpenSSL backend can only load a PKCS#12 archive; a PEM certificate is
/// rejected with a descriptive error.
fn client_identity(cert: &ClientCertificate) -> crate::Result<Identity> {
    match cert {
        ClientCertificate::Pkcs12 { path, password } => {
            let der = fs::read(path).map_err(|e| Error::Io {
                kind: e.kind(),
                message: format!(
                    "Could not read client certificate {}: {e}",
                    path.to_string_lossy()
                ),
            })?;
            Ok(Identity::from_pkcs12(&der, password)?)
        }
        ClientCertificate::Pem { .. } => Err(Error::Tls(
            "The vendored-openssl TLS backend requires a PKCS#12 client \
             certificate; use Config::client_certificate_pkcs12 instead of \
             client_certificate_pem."
                .to_string(),
        )),
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
