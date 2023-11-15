use std::sync::Arc;

use rustls::{
    client::ServerCertVerifier, server::ClientCertVerifier, version::TLS13, Certificate,
    ClientConfig, PrivateKey, RootCertStore, ServerConfig,
};

fn ca_store(
    ca_certs: impl IntoIterator<Item = Certificate>,
) -> Result<RootCertStore, rustls::Error> {
    let mut ca_store = RootCertStore::empty();
    for cert in ca_certs {
        ca_store.add(&cert)?;
    }
    Ok(ca_store)
}

pub fn client(
    private_key: PrivateKey,
    client_certs: impl IntoIterator<Item = Certificate>,
    // ca_certs: impl IntoIterator<Item = Certificate>,
) -> Result<ClientConfig, rustls::Error> {
    let mut config = rustls::ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&TLS13])
        .unwrap()
        .with_custom_certificate_verifier(Arc::new(NoServerVerify))
        .with_client_auth_cert(client_certs.into_iter().collect(), private_key)?;

    config.alpn_protocols = vec![b"peer2package".to_vec()];
    Ok(config)
}

pub fn server(
    private_key: PrivateKey,
    server_certs: impl IntoIterator<Item = Certificate>,
    // ca_certs: impl IntoIterator<Item = Certificate>,
) -> Result<ServerConfig, rustls::Error> {
    let mut config = rustls::ServerConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&TLS13])
        .unwrap()
        .with_client_cert_verifier(Arc::new(NoClientVerify))
        .with_single_cert(server_certs.into_iter().collect(), private_key)?;

    config.alpn_protocols = vec![b"peer2package".to_vec()];
    Ok(config)
}

struct NoServerVerify;
impl ServerCertVerifier for NoServerVerify {
    fn verify_server_cert(
        &self,
        end_entity: &Certificate,
        intermediates: &[Certificate],
        server_name: &rustls::ServerName,
        scts: &mut dyn Iterator<Item = &[u8]>,
        ocsp_response: &[u8],
        now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

struct NoClientVerify;
impl ClientCertVerifier for NoClientVerify {
    fn client_auth_root_subjects(&self) -> &[rustls::DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        end_entity: &Certificate,
        intermediates: &[Certificate],
        now: std::time::SystemTime,
    ) -> Result<rustls::server::ClientCertVerified, rustls::Error> {
        Ok(rustls::server::ClientCertVerified::assertion())
    }
}
