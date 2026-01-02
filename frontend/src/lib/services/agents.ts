import type { Agent, AgentMetrics } from '$lib/types/agent';
import { ApiClient } from './api-client';

export async function listAgents(): Promise<Agent[]> {
  const response = await ApiClient.get('/agents');

  if (!response.ok) {
    throw new Error(`Failed to fetch agents: ${response.statusText}`);
  }

  return response.json();
}

export async function getAgentDetails(agentId: string): Promise<Agent> {
  const response = await ApiClient.get(`/agents/${agentId}`);

  if (!response.ok) {
    throw new Error(`Failed to fetch agent details: ${response.statusText}`);
  }

  return response.json();
}

export async function getAgentMetrics(
  agentId: string,
  limit: number = 100
): Promise<AgentMetrics[]> {
  const response = await ApiClient.get(`/agents/${agentId}/metrics?limit=${limit}`);

  if (!response.ok) {
    throw new Error(`Failed to fetch agent metrics: ${response.statusText}`);
  }

  return response.json();
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

export function getStatusColor(status: string): string {
  switch (status) {
    case 'online':
      return 'text-green-500';
    case 'offline':
      return 'text-gray-500';
    case 'error':
      return 'text-red-500';
    default:
      return 'text-gray-500';
  }
}

export function getStatusBadgeClass(status: string): string {
  switch (status) {
    case 'online':
      return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
    case 'offline':
      return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200';
    case 'error':
      return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
    default:
      return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200';
  }
}
