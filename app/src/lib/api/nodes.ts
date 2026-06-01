const API_BASE = (import.meta.env.VITE_API_URL ?? 'http://localhost:8000') + '/api';

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
