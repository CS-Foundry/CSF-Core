<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    Activity,
    Cpu,
    HardDrive,
    Network,
    Server,
    TrendingUp,
    Globe,
    AlertCircle,
    CheckCircle,
  } from '@lucide/svelte';
  import * as Card from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Progress } from '$lib/components/ui/progress';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import SystemHealthCard from '$lib/components/dashboard/SystemHealthCard.svelte';
  import ResourceDistributionCard from '$lib/components/dashboard/ResourceDistributionCard.svelte';
  import ActivityFeedCard from '$lib/components/dashboard/ActivityFeedCard.svelte';
  import TrafficMapCard from '$lib/components/dashboard/TrafficMapCard.svelte';
  import UptimeCard from '$lib/components/dashboard/UptimeCard.svelte';
  import { ApiClient } from '$lib/services/api-client';

  interface SystemMetrics {
    timestamp: string;
    cpu_usage_percent: number;
    memory_total_bytes: number;
    memory_used_bytes: number;
    memory_usage_percent: number;
    disk_total_bytes: number;
    disk_used_bytes: number;
    disk_usage_percent: number;
    network_rx_bytes: number;
    network_tx_bytes: number;
    hostname: string;
    uptime_seconds: number;
  }

  let metrics: SystemMetrics | null = $state(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let updateInterval: number | null = null;

  async function fetchMetrics() {
    try {
      const response = await ApiClient.get('/system/metrics');
      if (!response.ok) {
        throw new Error('Failed to fetch metrics');
      }
      const data = await response.json();
      metrics = data.metrics;
      error = null;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Unknown error';
      console.error('Failed to fetch system metrics:', err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    fetchMetrics();
    // Update metrics every 5 seconds
    updateInterval = window.setInterval(fetchMetrics, 5000);
  });

  onDestroy(() => {
    if (updateInterval) {
      clearInterval(updateInterval);
    }
  });

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  }

  function formatUptime(seconds: number): string {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${days}d ${hours}h ${minutes}m`;
  }

  function getStatusColor(usage: number): string {
    if (usage < 60) return 'text-green-500';
    if (usage < 80) return 'text-yellow-500';
    return 'text-red-500';
  }

  function getStatusBadge(usage: number): 'default' | 'secondary' | 'destructive' {
    if (usage < 60) return 'default';
    if (usage < 80) return 'secondary';
    return 'destructive';
  }
</script>

<div class="container mx-auto p-6 space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold tracking-tight">Dashboard</h1>
      <p class="text-muted-foreground">Echtzeit-Überwachung deiner Cloud-Infrastruktur</p>
    </div>
    {#if metrics}
      <Badge variant={getStatusBadge(metrics.cpu_usage_percent)} class="text-sm px-4 py-2">
        <Activity class="mr-2 h-4 w-4" />
        System {metrics.cpu_usage_percent < 60
          ? 'Healthy'
          : metrics.cpu_usage_percent < 80
            ? 'Warning'
            : 'Critical'}
      </Badge>
    {/if}
  </div>

  {#if error}
    <Card.Root class="border-destructive">
      <Card.Content class="pt-6">
        <div class="flex items-center gap-2 text-destructive">
          <AlertCircle class="h-5 w-5" />
          <p>{error}</p>
        </div>
      </Card.Content>
    </Card.Root>
  {/if}

  <!-- Top Row: System Health Metrics -->
  <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
    <!-- CPU Card -->
    <Card.Root>
      <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
        <Card.Title class="text-sm font-medium">CPU Usage</Card.Title>
        <Cpu class="h-4 w-4 text-muted-foreground" />
      </Card.Header>
      <Card.Content>
        {#if loading}
          <Skeleton class="h-8 w-20 mb-2" />
          <Skeleton class="h-2 w-full" />
        {:else if metrics}
          <div class="text-2xl font-bold {getStatusColor(metrics.cpu_usage_percent)}">
            {metrics.cpu_usage_percent.toFixed(1)}%
          </div>
          <Progress value={metrics.cpu_usage_percent} class="mt-2" />
          <p class="text-xs text-muted-foreground mt-2">
            {metrics.cpu_usage_percent < 60
              ? 'Normal'
              : metrics.cpu_usage_percent < 80
                ? 'Elevated'
                : 'High'}
          </p>
        {/if}
      </Card.Content>
    </Card.Root>

    <!-- Memory Card -->
    <Card.Root>
      <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
        <Card.Title class="text-sm font-medium">Memory Usage</Card.Title>
        <Server class="h-4 w-4 text-muted-foreground" />
      </Card.Header>
      <Card.Content>
        {#if loading}
          <Skeleton class="h-8 w-20 mb-2" />
          <Skeleton class="h-2 w-full" />
        {:else if metrics}
          <div class="text-2xl font-bold {getStatusColor(metrics.memory_usage_percent)}">
            {metrics.memory_usage_percent.toFixed(1)}%
          </div>
          <Progress value={metrics.memory_usage_percent} class="mt-2" />
          <p class="text-xs text-muted-foreground mt-2">
            {formatBytes(metrics.memory_used_bytes)} / {formatBytes(metrics.memory_total_bytes)}
          </p>
        {/if}
      </Card.Content>
    </Card.Root>

    <!-- Disk Card -->
    <Card.Root>
      <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
        <Card.Title class="text-sm font-medium">Storage Usage</Card.Title>
        <HardDrive class="h-4 w-4 text-muted-foreground" />
      </Card.Header>
      <Card.Content>
        {#if loading}
          <Skeleton class="h-8 w-20 mb-2" />
          <Skeleton class="h-2 w-full" />
        {:else if metrics}
          <div class="text-2xl font-bold {getStatusColor(metrics.disk_usage_percent)}">
            {metrics.disk_usage_percent.toFixed(1)}%
          </div>
          <Progress value={metrics.disk_usage_percent} class="mt-2" />
          <p class="text-xs text-muted-foreground mt-2">
            {formatBytes(metrics.disk_used_bytes)} / {formatBytes(metrics.disk_total_bytes)}
          </p>
        {/if}
      </Card.Content>
    </Card.Root>

    <!-- Network Card -->
    <Card.Root>
      <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
        <Card.Title class="text-sm font-medium">Network Traffic</Card.Title>
        <Network class="h-4 w-4 text-muted-foreground" />
      </Card.Header>
      <Card.Content>
        {#if loading}
          <Skeleton class="h-8 w-20 mb-2" />
          <Skeleton class="h-4 w-full" />
        {:else if metrics}
          <div class="space-y-1">
            <div class="flex items-center justify-between">
              <span class="text-xs text-muted-foreground">↓ RX</span>
              <span class="text-sm font-semibold">{formatBytes(metrics.network_rx_bytes)}</span>
            </div>
            <div class="flex items-center justify-between">
              <span class="text-xs text-muted-foreground">↑ TX</span>
              <span class="text-sm font-semibold">{formatBytes(metrics.network_tx_bytes)}</span>
            </div>
          </div>
        {/if}
      </Card.Content>
    </Card.Root>
  </div>

  <!-- Middle Row: System Health, Uptime, Traffic Map -->
  <div class="grid gap-6 lg:grid-cols-3">
    <!-- Detailed System Health -->
    <div class="lg:col-span-1">
      <SystemHealthCard {metrics} {loading} />
    </div>

    <!-- Uptime & Availability -->
    <div class="lg:col-span-1">
      <UptimeCard uptime={metrics?.uptime_seconds || 0} {loading} />
    </div>

    <!-- Traffic World Map -->
    <div class="lg:col-span-1">
      <TrafficMapCard {loading} />
    </div>
  </div>

  <!-- Bottom Row: Resource Distribution & Activity Feed -->
  <div class="grid gap-6 lg:grid-cols-2">
    <!-- Resource Distribution -->
    <ResourceDistributionCard {loading} />

    <!-- Activity Feed -->
    <ActivityFeedCard {loading} />
  </div>

  <!-- Footer Info -->
  {#if metrics && !loading}
    <Card.Root>
      <Card.Content class="pt-6">
        <div class="flex items-center justify-between text-sm text-muted-foreground">
          <div class="flex items-center gap-2">
            <CheckCircle class="h-4 w-4 text-green-500" />
            <span>System: {metrics.hostname}</span>
          </div>
          <div class="flex items-center gap-2">
            <TrendingUp class="h-4 w-4" />
            <span>Uptime: {formatUptime(metrics.uptime_seconds)}</span>
          </div>
          <div class="flex items-center gap-2">
            <Activity class="h-4 w-4" />
            <span>Last updated: {new Date().toLocaleTimeString()}</span>
          </div>
        </div>
      </Card.Content>
    </Card.Root>
  {/if}
</div>
