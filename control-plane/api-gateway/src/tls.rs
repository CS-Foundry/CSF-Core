use axum_server::tls_rustls::RustlsConfig;
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair, SanType};

fn detect_primary_ip() -> Option<std::net::IpAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip())
}

pub async fn generate_tls_config() -> anyhow::Result<RustlsConfig> {
    let cert_path = std::env::var("TLS_CERT").unwrap_or_default();
    let key_path = std::env::var("TLS_KEY").unwrap_or_default();

    if !cert_path.is_empty() && !key_path.is_empty() {
        let cert_pem = std::fs::read(&cert_path)?;
        let key_pem = std::fs::read(&key_path)?;
        let config = RustlsConfig::from_pem(cert_pem, key_pem).await?;
        tracing::info!(cert = %cert_path, "TLS loaded from files");
        return Ok(config);
    }

    let key_pair = KeyPair::generate()?;

    let mut params = CertificateParams::default();
    params.distinguished_name = DistinguishedName::new();
    params
        .distinguished_name
        .push(DnType::CommonName, "csfx-gateway");

    let mut san_hosts = std::env::var("TLS_SANS").unwrap_or_else(|_| "localhost,127.0.0.1".to_string());
    if let Some(primary_ip) = detect_primary_ip() {
        let primary_ip = primary_ip.to_string();
        if !san_hosts.split(',').any(|s| s.trim() == primary_ip) {
            san_hosts.push(',');
            san_hosts.push_str(&primary_ip);
        }
    }

    for san in san_hosts.split(',') {
        let san = san.trim();
        if let Ok(ip) = san.parse::<std::net::IpAddr>() {
            params.subject_alt_names.push(SanType::IpAddress(ip));
        } else {
            params
                .subject_alt_names
                .push(SanType::DnsName(san.try_into()?));
        }
    }

    let cert = params.self_signed(&key_pair)?;
    let cert_pem = cert.pem().into_bytes();
    let key_pem = key_pair.serialize_pem().into_bytes();

    let config = RustlsConfig::from_pem(cert_pem, key_pem).await?;

    tracing::info!(sans = %san_hosts, "self-signed TLS certificate generated");
    Ok(config)
}
