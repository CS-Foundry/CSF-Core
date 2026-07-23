use std::collections::HashMap;

use crate::models::compose::ComposeServiceSpec;
use crate::models::workload::PortMapping;

const DEFAULT_CPU_MILLICORES: i32 = 500;
const DEFAULT_MEMORY_BYTES: i64 = 512 * 1024 * 1024;
const DEFAULT_DISK_BYTES: i64 = 1024 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ComposeParseError {
    #[error("invalid compose yaml: {0}")]
    InvalidYaml(String),
    #[error("compose file defines no services")]
    NoServicesDefined,
    #[error("service '{0}' uses build, only image is supported")]
    UnsupportedBuildDirective(String),
    #[error("service '{0}' has no image")]
    MissingImage(String),
}

#[derive(Debug, serde::Deserialize)]
struct RawCompose {
    services: HashMap<String, RawService>,
}

#[derive(Debug, serde::Deserialize)]
struct RawService {
    image: Option<String>,
    build: Option<serde_yaml::Value>,
    ports: Option<Vec<String>>,
    environment: Option<RawEnvironment>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum RawEnvironment {
    List(Vec<String>),
    Map(HashMap<String, String>),
}

pub fn parse_compose(yaml: &str) -> Result<Vec<ComposeServiceSpec>, ComposeParseError> {
    let raw: RawCompose =
        serde_yaml::from_str(yaml).map_err(|e| ComposeParseError::InvalidYaml(e.to_string()))?;

    if raw.services.is_empty() {
        return Err(ComposeParseError::NoServicesDefined);
    }

    raw.services
        .into_iter()
        .map(|(service_name, service)| build_service_spec(service_name, service))
        .collect()
}

fn build_service_spec(
    service_name: String,
    service: RawService,
) -> Result<ComposeServiceSpec, ComposeParseError> {
    if service.build.is_some() {
        return Err(ComposeParseError::UnsupportedBuildDirective(service_name));
    }
    let image = service
        .image
        .ok_or_else(|| ComposeParseError::MissingImage(service_name.clone()))?;

    Ok(ComposeServiceSpec {
        service_name,
        image,
        env_vars: normalize_environment(service.environment),
        ports: service.ports.map(|raw| parse_compose_ports(&raw)),
        cpu_millicores: DEFAULT_CPU_MILLICORES,
        memory_bytes: DEFAULT_MEMORY_BYTES,
        disk_bytes: DEFAULT_DISK_BYTES,
    })
}

fn normalize_environment(raw: Option<RawEnvironment>) -> Option<HashMap<String, String>> {
    match raw? {
        RawEnvironment::Map(map) => Some(map),
        RawEnvironment::List(list) => Some(
            list.into_iter()
                .filter_map(|entry| {
                    let (key, value) = entry.split_once('=')?;
                    Some((key.to_string(), value.to_string()))
                })
                .collect(),
        ),
    }
}

fn parse_compose_ports(raw: &[String]) -> Vec<PortMapping> {
    raw.iter()
        .filter_map(|entry| parse_compose_port(entry))
        .collect()
}

fn parse_compose_port(entry: &str) -> Option<PortMapping> {
    let (rg_port, container_port) = match entry.split_once(':') {
        Some((host, container)) => (host.parse().ok(), container),
        None => (None, entry),
    };

    Some(PortMapping {
        container_port: container_port.parse().ok()?,
        protocol: None,
        rg_port,
        node_port: None,
    })
}
