import type { Resource, CreateResourceRequest, UpdateResourceRequest } from '$lib/types/resource';
import { ApiClient } from './api-client';

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    let errorMessage = 'Request failed';
    try {
      const errorData = await response.json();
      errorMessage = errorData.error || errorData.message || errorMessage;
    } catch {
      errorMessage = response.statusText || errorMessage;
    }
    throw new Error(errorMessage);
  }
  
  // Handle 204 No Content
  if (response.status === 204) {
    return undefined as T;
  }
  
  return response.json();
}

export async function listResources(): Promise<Resource[]> {
  const response = await ApiClient.get('/resources');
  return handleResponse<Resource[]>(response);
}

export async function listResourcesByGroup(resourceGroupId: string): Promise<Resource[]> {
  const response = await ApiClient.get(`/resource-groups/${resourceGroupId}/resources`);
  return handleResponse<Resource[]>(response);
}

export async function getResource(id: string): Promise<Resource> {
  const response = await ApiClient.get(`/resources/${id}`);
  return handleResponse<Resource>(response);
}

export async function createResource(data: CreateResourceRequest): Promise<Resource> {
  const response = await ApiClient.post('/resources', data);
  return handleResponse<Resource>(response);
}

export async function updateResource(id: string, data: UpdateResourceRequest): Promise<Resource> {
  const response = await ApiClient.put(`/resources/${id}`, data);
  return handleResponse<Resource>(response);
}

export async function deleteResource(id: string): Promise<void> {
  const response = await ApiClient.delete(`/resources/${id}`);
  
  if (!response.ok) {
    await handleResponse(response);
  }
}

export async function performResourceAction(id: string, action: 'start' | 'stop' | 'restart'): Promise<Resource> {
  const response = await ApiClient.post(`/resources/${id}/action`, { action });
  return handleResponse<Resource>(response);
}

export interface DeployContainerRequest {
  name: string;
  image: string;
  resource_group_id: string;
  description?: string;
  ports?: Array<{ container: number; host: number }>;
  environment?: Record<string, string>;
  volumes?: Array<{ host: string; container: string }>;
}

export async function deployContainer(data: DeployContainerRequest): Promise<Resource> {
  const response = await ApiClient.post('/resources/deploy', data);
  return handleResponse<Resource>(response);
}

export interface ResourceLogsResponse {
  logs: string;
}

export async function getResourceLogs(id: string): Promise<ResourceLogsResponse> {
  const response = await ApiClient.get(`/resources/${id}/logs`);
  return handleResponse<ResourceLogsResponse>(response);
}

export interface ExecCommandRequest {
  command: string;
}

export interface ExecCommandResponse {
  output: string;
}

export async function execCommand(id: string, command: string): Promise<ExecCommandResponse> {
  const response = await ApiClient.post(`/resources/${id}/exec`, { command });
  return handleResponse<ExecCommandResponse>(response);
}
