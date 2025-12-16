export interface Agent {
  id: string;
  name: string;
  hostname: string;
  agent_version: string;
  os_type: string;
  os_version: string;
  status: 'online' | 'offline' | 'error';
  last_heartbeat: string;
  tags?: Record<string, string>;
  capabilities?: string[];
  organization_id?: string;
  created_at: string;
  updated_at: string;
}

export interface AgentMetrics {
  id: string;
  agent_id: string;
  timestamp: string;
  cpu_usage_percent: number;
  memory_usage_percent: number;
  memory_total_bytes: number;
  memory_used_bytes: number;
  disk_usage_percent: number;
  disk_total_bytes: number;
  disk_used_bytes: number;
  network_rx_bytes?: number;
  network_tx_bytes?: number;
  custom_metrics?: Record<string, any>;
}

export interface AgentWithLatestMetrics extends Agent {
  latest_metrics?: AgentMetrics;
}
