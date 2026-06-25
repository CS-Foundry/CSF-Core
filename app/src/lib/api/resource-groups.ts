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

export async function listResourceGroups(token: string): Promise<ResourceGroup[]> {
    const res = await fetch(`${API_BASE}/resource-groups`, {
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`Failed to list resource groups: ${res.status}`);
    return res.json();
}

export async function createResourceGroup(
    token: string,
    req: CreateResourceGroupRequest,
): Promise<ResourceGroup> {
    const res = await fetch(`${API_BASE}/resource-groups`, {
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
    const res = await fetch(`${API_BASE}/resource-groups/${id}`, {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error(`Failed to delete resource group: ${res.status}`);
}
