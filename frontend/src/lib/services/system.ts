import { ApiClient } from './api-client';

export interface LocalSystemInfo {
  hostname: string;
  os_name: string;
  os_version: string;
  kernel_version: string;
  uptime_seconds: number;
  cpu_model: string;
  cpu_cores: number;
  cpu_threads: number;
}

export interface LocalSystemMetrics {
  timestamp: string;
  cpu_model: string;
  cpu_cores: number;
  cpu_threads: number;
  cpu_usage_percent: number;
  memory_total_bytes: number;
  memory_used_bytes: number;
  memory_usage_percent: number;
  disk_total_bytes: number;
  disk_used_bytes: number;
  disk_usage_percent: number;
  network_rx_bytes: number;
  network_tx_bytes: number;
  os_name: string;
  os_version: string;
  kernel_version: string;
  hostname: string;
  uptime_seconds: number;
}

export async function getLocalSystemInfo(): Promise<LocalSystemInfo> {
  const response = await ApiClient.get('/system/info');

  if (!response.ok) {
    throw new Error(`Failed to fetch system info: ${response.statusText}`);
  }

  return response.json();
}

export async function getLocalSystemMetrics(): Promise<LocalSystemMetrics> {
  const response = await ApiClient.get('/system/metrics');

  if (!response.ok) {
    throw new Error(`Failed to fetch system metrics: ${response.statusText}`);
  }

  const data = await response.json();
  return data.metrics;
}

export function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  const parts = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0) parts.push(`${hours}h`);
  if (minutes > 0) parts.push(`${minutes}m`);

  return parts.join(' ') || '0m';
}
