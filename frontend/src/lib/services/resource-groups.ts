import type { ResourceGroup, CreateResourceGroupRequest, UpdateResourceGroupRequest } from '$lib/types/resource-group';
import { ApiClient } from './api-client';

async function handleResponse<T>(response: Response): Promise<T> {
  const contentType = response.headers.get('content-type');
  
  if (!response.ok) {
    // Check if response is JSON
    if (contentType && contentType.includes('application/json')) {
      const error = await response.json();
      throw new Error(error.error || error.message || `HTTP ${response.status}`);
    } else {
      // If not JSON, it might be HTML (error page or redirect)
      const text = await response.text();
      console.error('Non-JSON response:', response.status, text.substring(0, 200));
      
      if (response.status === 401) {
        throw new Error('Not authenticated. Please log in again.');
      }
      throw new Error(`Request failed with status ${response.status}`);
    }
  }
  
  // Successful response
  if (contentType && contentType.includes('application/json')) {
    return response.json();
  } else {
    throw new Error('Expected JSON response but got: ' + contentType);
  }
}

export async function listResourceGroups(): Promise<ResourceGroup[]> {
  const response = await ApiClient.get('/resource-groups');
  return handleResponse<ResourceGroup[]>(response);
}

export async function getResourceGroup(id: string): Promise<ResourceGroup> {
  const response = await ApiClient.get(`/resource-groups/${id}`);
  return handleResponse<ResourceGroup>(response);
}

export async function createResourceGroup(data: CreateResourceGroupRequest): Promise<ResourceGroup> {
  const response = await ApiClient.post('/resource-groups', data);
  return handleResponse<ResourceGroup>(response);
}

export async function updateResourceGroup(id: string, data: UpdateResourceGroupRequest): Promise<ResourceGroup> {
  const response = await ApiClient.put(`/resource-groups/${id}`, data);
  return handleResponse<ResourceGroup>(response);
}

export async function deleteResourceGroup(id: string): Promise<void> {
  const response = await ApiClient.delete(`/resource-groups/${id}`);
  
  if (!response.ok) {
    await handleResponse(response);
  }
}
