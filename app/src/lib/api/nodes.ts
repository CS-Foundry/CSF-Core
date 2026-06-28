const API_BASE = '/api';

export interface Node {
    id: string;
    name: string;
    hostname: string;
    ip_address: string | null;
    os_type: string;
    os_version: string;
    architecture: string;
    agent_version: string;
    status: string;
    last_heartbeat: string | null;
    registered_at: string;
}

export interface NodeMetrics {
    agent_id: string;
    hostname: string;
    status: string;
    cpu_usage_percent: number | null;
    memory_total_bytes: number | null;
    memory_used_bytes: number | null;
    disk_total_bytes: number | null;
    disk_used_bytes: number | null;
}

export interface NodeMetricsLatest {
    id: string;
    agent_id: string;
    timestamp: string;
    cpu_model: string | null;
    cpu_cores: number | null;
    cpu_threads: number | null;
    cpu_usage_percent: number | null;
    memory_total_bytes: number | null;
    memory_used_bytes: number | null;
    memory_usage_percent: number | null;
    disk_total_bytes: number | null;
    disk_used_bytes: number | null;
    disk_usage_percent: number | null;
    network_rx_bytes: number | null;
    network_tx_bytes: number | null;
    os_name: string | null;
    os_version: string | null;
    kernel_version: string | null;
    hostname: string | null;
    uptime_seconds: number | null;
}

export interface ClusterStats {
    node_count: number;
    online_count: number;
    total_cpu_cores: number;
    avg_cpu_usage_percent: number;
    total_memory_bytes: number;
    used_memory_bytes: number;
    total_disk_bytes: number;
    used_disk_bytes: number;
    nodes: NodeMetrics[];
}

export async function listNodes(token: string): Promise<Node[]> {
    const res = await fetch(`${API_BASE}/agents`, {
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`agents fetch failed: ${res.status}`);
    return res.json();
}

export async function getClusterStats(token: string): Promise<ClusterStats> {
    const res = await fetch(`${API_BASE}/system/stats`, {
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`system/stats fetch failed: ${res.status}`);
    return res.json();
}

export async function getNode(token: string, id: string): Promise<Node> {
    const res = await fetch(`${API_BASE}/agents/${id}`, {
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`agent fetch failed: ${res.status}`);
    return res.json();
}

export async function getNodeMetricsLatest(token: string, id: string): Promise<NodeMetricsLatest> {
    const res = await fetch(`${API_BASE}/agents/${id}/metrics/latest`, {
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`metrics fetch failed: ${res.status}`);
    return res.json();
}

export interface HealthHistoryPoint {
    bucket: string;
    online_count: number;
}

export async function getHealthHistory(token: string, range: '1h' | '7d' | '30d'): Promise<HealthHistoryPoint[]> {
    const res = await fetch(`${API_BASE}/system/stats/history?range=${range}`, {
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`health history fetch failed: ${res.status}`);
    return res.json();
}
