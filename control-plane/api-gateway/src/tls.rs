use axum_server::tls_rustls::RustlsConfig;
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair, SanType};

pub fn generate_tls_config() -> anyhow::Result<RustlsConfig> {
    let cert_path = std::env::var("TLS_CERT").unwrap_or_default();
    let key_path = std::env::var("TLS_KEY").unwrap_or_default();

    if !cert_path.is_empty() && !key_path.is_empty() {
        let cert_pem = std::fs::read(&cert_path)?;
        let key_pem = std::fs::read(&key_path)?;
        let config = tokio::runtime::Handle::current()
            .block_on(RustlsConfig::from_pem(cert_pem, key_pem))?;
        tracing::info!(cert = %cert_path, "TLS loaded from files");
        return Ok(config);
    }

    let key_pair = KeyPair::generate()?;

    let mut params = CertificateParams::default();
    params.distinguished_name = DistinguishedName::new();
    params.distinguished_name.push(DnType::CommonName, "csfx-gateway");

    let san_hosts = std::env::var("TLS_SANS").unwrap_or_else(|_| "localhost,127.0.0.1".to_string());
    for san in san_hosts.split(',') {
        let san = san.trim();
        if let Ok(ip) = san.parse::<std::net::IpAddr>() {
            params.subject_alt_names.push(SanType::IpAddress(ip));
        } else {
            params.subject_alt_names.push(SanType::DnsName(san.try_into()?));
        }
    }

    let cert = params.self_signed(&key_pair)?;
    let cert_pem = cert.pem().into_bytes();
    let key_pem = key_pair.serialize_pem().into_bytes();

    let config = tokio::runtime::Handle::current()
        .block_on(RustlsConfig::from_pem(cert_pem, key_pem))?;

    tracing::info!(sans = %san_hosts, "self-signed TLS certificate generated");
    Ok(config)
}
