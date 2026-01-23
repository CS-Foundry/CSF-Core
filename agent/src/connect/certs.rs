use anyhow::{Context, Result};
use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, KeyPair, KeyUsagePurpose,
};
use std::fs;
use std::path::Path;
use time::{Duration, OffsetDateTime};

/// Generates a self-signed CA certificate
pub fn generate_ca_cert(common_name: &str, output_dir: &Path) -> Result<()> {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, common_name);
    dn.push(DnType::OrganizationName, "CSF Agent Network");
    dn.push(DnType::CountryName, "US");
    params.distinguished_name = dn;

    // Set validity period (10 years)
    params.not_before = OffsetDateTime::now_utc();
    params.not_after = OffsetDateTime::now_utc() + Duration::days(3650);

    // Key usage for CA
    params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
    ];

    let key_pair = KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;

    // Create output directory
    fs::create_dir_all(output_dir).context("Failed to create certificate directory")?;

    // Save CA certificate
    let ca_cert_path = output_dir.join("ca.crt");
    fs::write(&ca_cert_path, cert.pem()).context("Failed to write CA certificate")?;

    // Save CA private key
    let ca_key_path = output_dir.join("ca.key");
    fs::write(&ca_key_path, key_pair.serialize_pem()).context("Failed to write CA private key")?;

    // Set restrictive permissions on private key (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&ca_key_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&ca_key_path, perms)?;
    }

    tracing::info!("Generated CA certificate at {:?}", ca_cert_path);
    Ok(())
}

/// Generates an agent certificate signed by the CA
pub fn generate_agent_cert(
    agent_name: &str,
    ca_cert_path: &Path,
    ca_key_path: &Path,
    output_dir: &Path,
) -> Result<()> {
    // Load CA certificate and key
    let ca_cert_pem = fs::read_to_string(ca_cert_path).context("Failed to read CA certificate")?;
    let ca_key_pem = fs::read_to_string(ca_key_path).context("Failed to read CA private key")?;

    let ca_key_pair = KeyPair::from_pem(&ca_key_pem).context("Failed to parse CA private key")?;
    let ca_cert_params = CertificateParams::from_ca_cert_pem(&ca_cert_pem)
        .context("Failed to parse CA certificate")?;
    let ca_cert = ca_cert_params.self_signed(&ca_key_pair)?;

    // Create agent certificate parameters
    let mut params = CertificateParams::default();

    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, agent_name);
    dn.push(DnType::OrganizationName, "CSF Agent Network");
    params.distinguished_name = dn;

    // Set validity period (1 year)
    params.not_before = OffsetDateTime::now_utc();
    params.not_after = OffsetDateTime::now_utc() + Duration::days(365);

    // Key usage for agent certificates
    params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyEncipherment,
    ];

    // Extended key usage for server and client auth
    params.extended_key_usages = vec![
        rcgen::ExtendedKeyUsagePurpose::ServerAuth,
        rcgen::ExtendedKeyUsagePurpose::ClientAuth,
    ];

    // Add SubjectAlternativeNames for localhost and common IPs
    params.subject_alt_names = vec![
        rcgen::SanType::DnsName("localhost".try_into().unwrap()),
        rcgen::SanType::IpAddress(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
        rcgen::SanType::IpAddress(std::net::IpAddr::V6(std::net::Ipv6Addr::new(
            0, 0, 0, 0, 0, 0, 0, 1,
        ))),
    ];

    // Generate agent key pair
    let agent_key_pair = KeyPair::generate()?;

    // Sign agent certificate with CA
    let agent_cert = params.signed_by(&agent_key_pair, &ca_cert, &ca_key_pair)?;

    // Create output directory
    fs::create_dir_all(output_dir).context("Failed to create certificate directory")?;

    // Save agent certificate
    let agent_cert_path = output_dir.join("agent.crt");
    fs::write(&agent_cert_path, agent_cert.pem()).context("Failed to write agent certificate")?;

    // Save agent private key
    let agent_key_path = output_dir.join("agent.key");
    fs::write(&agent_key_path, agent_key_pair.serialize_pem())
        .context("Failed to write agent private key")?;

    // Set restrictive permissions on private key (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&agent_key_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&agent_key_path, perms)?;
    }

    tracing::info!("Generated agent certificate at {:?}", agent_cert_path);
    Ok(())
}

/// Ensures certificates exist, generates them if needed
pub fn ensure_certificates(agent_name: &str, cert_dir: &Path, auto_generate: bool) -> Result<()> {
    let ca_cert_path = cert_dir.join("ca.crt");
    let ca_key_path = cert_dir.join("ca.key");
    let agent_cert_path = cert_dir.join("agent.crt");
    let agent_key_path = cert_dir.join("agent.key");

    // Check if certificates already exist
    let ca_exists = ca_cert_path.exists() && ca_key_path.exists();
    let agent_exists = agent_cert_path.exists() && agent_key_path.exists();

    if ca_exists && agent_exists {
        tracing::info!("Certificates already exist, skipping generation");
        return Ok(());
    }

    if !auto_generate {
        anyhow::bail!(
            "Certificates not found and auto-generation is disabled. \
             Please provide certificates at {:?}",
            cert_dir
        );
    }

    tracing::info!("Generating certificates...");

    // Generate CA if it doesn't exist
    if !ca_exists {
        let ca_common_name = format!("CSF-Agent-CA-{}", uuid::Uuid::new_v4());
        generate_ca_cert(&ca_common_name, cert_dir)?;
    }

    // Generate agent certificate if it doesn't exist
    if !agent_exists {
        generate_agent_cert(agent_name, &ca_cert_path, &ca_key_path, cert_dir)?;
    }

    Ok(())
}

/// Loads certificates for mTLS
pub fn load_certs(path: &Path) -> Result<Vec<rustls::pki_types::CertificateDer<'static>>> {
    let cert_file =
        fs::File::open(path).context(format!("Failed to open certificate file: {:?}", path))?;
    let mut reader = std::io::BufReader::new(cert_file);

    rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to parse certificates")
}

/// Loads private key
pub fn load_private_key(path: &Path) -> Result<rustls::pki_types::PrivateKeyDer<'static>> {
    let key_file =
        fs::File::open(path).context(format!("Failed to open private key file: {:?}", path))?;
    let mut reader = std::io::BufReader::new(key_file);

    // Try to read as PKCS8 first
    if let Some(key) = rustls_pemfile::pkcs8_private_keys(&mut reader)
        .next()
        .transpose()
        .context("Failed to parse PKCS8 private key")?
    {
        return Ok(rustls::pki_types::PrivateKeyDer::Pkcs8(key));
    }

    // Reset reader and try RSA
    let key_file = fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(key_file);

    if let Some(key) = rustls_pemfile::rsa_private_keys(&mut reader)
        .next()
        .transpose()
        .context("Failed to parse RSA private key")?
    {
        return Ok(rustls::pki_types::PrivateKeyDer::Pkcs1(key));
    }

    anyhow::bail!("No valid private key found in {:?}", path)
}
