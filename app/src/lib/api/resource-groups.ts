import { authedFetch } from './http';

const API_BASE = '/api';

export interface ResourceGroup {
    id: string;
    organization_id: string;
    name: string;
    description: string | null;
    internal_cidr: string;
    status: string;
    created_at: string;
    updated_at: string | null;
}

export interface CreateResourceGroupRequest {
    name: string;
    description?: string;
    internal_cidr: string;
}

export interface PortMapping {
    container_port: number;
    protocol: string | null;
    rg_port: number | null;
    node_port: number | null;
}

export interface VolumeMount {
    volume_id: string;
    mount_path: string;
}

export interface Volume {
    id: string;
    name: string;
    size_gb: number;
    pool: string;
    status: string;
    attached_to_workload: string | null;
    resource_group_id: string | null;
    created_at: string;
}

export interface Workload {
    id: string;
    name: string;
    image: string;
    cpu_millicores: number;
    memory_bytes: number;
    disk_bytes: number;
    status: string;
    assigned_agent_id: string | null;
    container_id: string | null;
    env_vars: Record<string, string> | null;
    ports: PortMapping[] | null;
    volume_mounts: VolumeMount[] | null;
    resource_group_id: string | null;
    stack_id: string | null;
    service_name: string | null;
    restart_policy: string;
    max_restarts: number | null;
    restart_count: number;
    desired_state: string;
    cpu_usage_percent: number | null;
    memory_usage_bytes: number | null;
    network_rx_bytes: number | null;
    network_tx_bytes: number | null;
    stats_updated_at: string | null;
    created_at: string;
    updated_at: string | null;
}

export interface CreateWorkloadRequest {
    name: string;
    image: string;
    cpu_millicores: number;
    memory_bytes: number;
    disk_bytes: number;
    env_vars: Record<string, string> | null;
    ports: PortMapping[] | null;
    volume_mounts: VolumeMount[] | null;
    resource_group_id: string;
    restart_policy?: 'always' | 'on-failure' | 'never';
    max_restarts?: number | null;
}

export interface UpdateWorkloadRequest {
    image?: string;
    env_vars?: Record<string, string> | null;
    ports?: PortMapping[] | null;
    restart_policy?: 'always' | 'on-failure' | 'never';
    max_restarts?: number | null;
}

export interface CreateVolumeRequest {
    name: string;
    size_gb: number;
    resource_group_id: string;
}

export interface CreateStackRequest {
    name: string;
    resource_group_id: string;
    compose_yaml: string;
}

export interface CreateStackWorkloadResult {
    workload_id: string;
    status: string;
    assigned_agent_id: string | null;
    message: string;
}

export interface CreateStackResponse {
    stack_id: string;
    workloads: CreateStackWorkloadResult[];
}

export async function listResourceGroups(token: string): Promise<ResourceGroup[]> {
    const res = await authedFetch(`${API_BASE}/resource-groups`, {
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`Failed to list resource groups: ${res.status}`);
    return res.json();
}

export async function getResourceGroup(token: string, id: string): Promise<ResourceGroup> {
    const res = await authedFetch(`${API_BASE}/resource-groups/${id}`, {
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`Failed to get resource group: ${res.status}`);
    return res.json();
}

export async function createResourceGroup(
    token: string,
    req: CreateResourceGroupRequest,
): Promise<ResourceGroup> {
    const res = await authedFetch(`${API_BASE}/resource-groups`, {
        method: 'POST',
        headers: {
            Authorization: `Bearer ${token}`,
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(req),
    });
    if (!res.ok) throw new Error(`Failed to create resource group: ${res.status}`);
    return res.json();
}

export async function deleteResourceGroup(token: string, id: string): Promise<void> {
    const res = await authedFetch(`${API_BASE}/resource-groups/${id}`, {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`Failed to delete resource group: ${res.status}`);
}

export async function listResourceGroupWorkloads(token: string, rgId: string): Promise<Workload[]> {
    const res = await authedFetch(`${API_BASE}/resource-groups/${rgId}/workloads`, {
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`Failed to list workloads: ${res.status}`);
    return res.json();
}

export async function createWorkload(token: string, req: CreateWorkloadRequest): Promise<{ workload_id: string; status: string; assigned_agent_id: string | null; message: string }> {
    const res = await authedFetch(`${API_BASE}/workloads`, {
        method: 'POST',
        headers: {
            Authorization: `Bearer ${token}`,
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(req),
    });
    if (!res.ok) {
        const err = await res.json().catch(() => ({ error: res.status }));
        throw new Error(err.error ?? `Failed to create workload: ${res.status}`);
    }
    return res.json();
}

export async function createWorkloadStack(
    token: string,
    req: CreateStackRequest,
): Promise<CreateStackResponse> {
    const res = await authedFetch(`${API_BASE}/workload-stacks`, {
        method: 'POST',
        headers: {
            Authorization: `Bearer ${token}`,
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(req),
    });
    if (!res.ok) {
        const err = await res.json().catch(() => ({ error: res.status }));
        throw new Error(err.error ?? `Failed to create stack: ${res.status}`);
    }
    return res.json();
}

export interface Stack {
    id: string;
    resource_group_id: string;
    name: string;
    compose_source: string | null;
    status: string;
    created_at: string;
    updated_at: string | null;
}

export async function getStack(token: string, id: string): Promise<Stack> {
    const res = await authedFetch(`${API_BASE}/workload-stacks/${id}`, {
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`Failed to get stack: ${res.status}`);
    return res.json();
}

export async function deleteStack(token: string, id: string): Promise<void> {
    const res = await authedFetch(`${API_BASE}/workload-stacks/${id}`, {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`Failed to delete stack: ${res.status}`);
}

export async function restartStack(token: string, id: string): Promise<void> {
    const res = await authedFetch(`${API_BASE}/workload-stacks/${id}/restart`, {
        method: 'POST',
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`Failed to restart stack: ${res.status}`);
}

export async function redeployStack(token: string, id: string, compose_yaml: string): Promise<void> {
    const res = await authedFetch(`${API_BASE}/workload-stacks/${id}`, {
        method: 'PATCH',
        headers: {
            Authorization: `Bearer ${token}`,
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ compose_yaml }),
    });
    if (!res.ok) {
        const err = await res.json().catch(() => ({ error: res.status }));
        throw new Error(err.error ?? `Failed to redeploy stack: ${res.status}`);
    }
}

export async function deleteWorkload(token: string, id: string): Promise<void> {
    const res = await authedFetch(`${API_BASE}/workloads/${id}`, {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`Failed to delete workload: ${res.status}`);
}

export async function updateWorkload(
    token: string,
    id: string,
    req: UpdateWorkloadRequest,
): Promise<Workload> {
    const res = await authedFetch(`${API_BASE}/workloads/${id}`, {
        method: 'PATCH',
        headers: {
            Authorization: `Bearer ${token}`,
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(req),
    });
    if (!res.ok) {
        const err = await res.json().catch(() => ({ error: res.status }));
        throw new Error(err.error ?? `Failed to update workload: ${res.status}`);
    }
    return res.json();
}

export async function stopWorkload(token: string, id: string): Promise<Workload> {
    const res = await authedFetch(`${API_BASE}/workloads/${id}/stop`, {
        method: 'POST',
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`Failed to stop workload: ${res.status}`);
    return res.json();
}

export async function restartWorkload(token: string, id: string): Promise<Workload> {
    const res = await authedFetch(`${API_BASE}/workloads/${id}/restart`, {
        method: 'POST',
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`Failed to restart workload: ${res.status}`);
    return res.json();
}

export async function streamWorkloadLogs(
    token: string,
    id: string,
    signal: AbortSignal,
): Promise<ReadableStream<Uint8Array>> {
    const res = await authedFetch(`${API_BASE}/workloads/${id}/logs`, {
        headers: { Authorization: `Bearer ${token}` },
        signal,
    });
    if (!res.ok || !res.body) {
        const err = await res.json().catch(() => ({ error: res.status }));
        throw new Error(err.error ?? `Failed to stream logs: ${res.status}`);
    }
    return res.body;
}

export async function openWorkloadExecSocket(token: string, id: string): Promise<WebSocket> {
    const res = await authedFetch(`${API_BASE}/workloads/${id}/exec/ticket`, {
        method: 'POST',
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) {
        const err = await res.json().catch(() => ({ error: res.status }));
        throw new Error(err.error ?? `Failed to issue exec ticket: ${res.status}`);
    }
    const { ticket } = await res.json();

    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${wsProtocol}//${window.location.host}${API_BASE}/workloads/${id}/exec?ticket=${encodeURIComponent(ticket)}`;
    return new WebSocket(url);
}

export async function listResourceGroupVolumes(token: string, rgId: string): Promise<Volume[]> {
    const res = await authedFetch(`${API_BASE}/resource-groups/${rgId}/volumes`, {
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`Failed to list volumes: ${res.status}`);
    return res.json();
}

export async function createVolume(token: string, req: CreateVolumeRequest): Promise<Volume> {
    const res = await authedFetch(`${API_BASE}/volumes`, {
        method: 'POST',
        headers: {
            Authorization: `Bearer ${token}`,
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(req),
    });
    if (!res.ok) {
        const err = await res.json().catch(() => ({ error: res.status }));
        throw new Error(err.error ?? `Failed to create volume: ${res.status}`);
    }
    return res.json();
}

export async function deleteVolume(token: string, id: string): Promise<void> {
    const res = await authedFetch(`${API_BASE}/volumes/${id}`, {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`Failed to delete volume: ${res.status}`);
}
