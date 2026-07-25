use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub container_port: u16,
    pub protocol: Option<String>,
    pub rg_port: Option<u16>,
    pub node_port: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct VolumeMount {
    pub volume_id: String,
    pub mount_path: String,
    pub device_path: String,
}

#[derive(Debug, Clone)]
pub struct WorkloadSpec {
    pub workload_id: String,
    pub image: String,
    pub cpu_millicores: i32,
    pub memory_bytes: i64,
    pub env_vars: Option<HashMap<String, String>>,
    pub ports: Option<Vec<PortMapping>>,
    pub volume_mounts: Option<Vec<VolumeMount>>,
    pub service_name: Option<String>,
    pub resource_group_id: Option<String>,
    pub resource_group_cidr: Option<String>,
}

pub fn rg_network_name(resource_group_id: &str) -> String {
    format!("csfx-rg-{}", resource_group_id)
}

pub fn second_host_ip(cidr: &str) -> Option<String> {
    nth_host_ip(cidr, 2)
}

pub fn nth_host_ip(cidr: &str, offset: u32) -> Option<String> {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return None;
    }
    let octets: Vec<u8> = parts[0].split('.').filter_map(|o| o.parse().ok()).collect();
    if octets.len() != 4 {
        return None;
    }
    let n = u32::from_be_bytes([octets[0], octets[1], octets[2], octets[3]]);
    let host = n + offset;
    let [a, b, c, d] = host.to_be_bytes();
    Some(format!("{}.{}.{}.{}", a, b, c, d))
}

pub fn cidr_host_count(cidr: &str) -> Option<u32> {
    let prefix: u32 = cidr.split('/').nth(1)?.parse().ok()?;
    if prefix > 32 {
        return None;
    }
    Some(1u32 << (32 - prefix))
}

pub fn rg_wireguard_port(resource_group_id: &str) -> u16 {
    const FNV_OFFSET_BASIS: u32 = 0x811c9dc5;
    const FNV_PRIME: u32 = 0x01000193;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in resource_group_id.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    49152u16 + (hash % (65535 - 49152)) as u16
}

pub fn rg_bridge_iface_name(resource_group_id: &str) -> String {
    const FNV_OFFSET_BASIS: u32 = 0x811c9dc5;
    const FNV_PRIME: u32 = 0x01000193;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in resource_group_id.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("csfxrg{:08x}", hash)
}
