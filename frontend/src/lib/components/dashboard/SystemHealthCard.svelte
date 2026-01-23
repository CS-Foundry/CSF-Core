<script lang="ts">
  import { Server, Cpu, HardDrive, Zap } from '@lucide/svelte';
  import * as Card from '$lib/components/ui/card';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import { Progress } from '$lib/components/ui/progress';

  interface Props {
    metrics: any;
    loading: boolean;
  }

  let { metrics, loading }: Props = $props();

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  }
</script>

<Card.Root>
  <Card.Header>
    <Card.Title class="flex items-center gap-2">
      <Zap class="h-5 w-5" />
      System Health
    </Card.Title>
    <Card.Description>Detaillierte Hardware-Metriken</Card.Description>
  </Card.Header>
  <Card.Content class="space-y-4">
    {#if loading}
      <div class="space-y-4">
        <Skeleton class="h-16 w-full" />
        <Skeleton class="h-16 w-full" />
        <Skeleton class="h-16 w-full" />
      </div>
    {:else if metrics}
      <!-- CPU Details -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <Cpu class="h-4 w-4 text-muted-foreground" />
            <span class="text-sm font-medium">CPU</span>
          </div>
          <span class="text-sm font-bold">{metrics.cpu_usage_percent.toFixed(1)}%</span>
        </div>
        <Progress value={metrics.cpu_usage_percent} />
        <p class="text-xs text-muted-foreground">
          {metrics.cpu_usage_percent < 60
            ? '✓ Normal load'
            : metrics.cpu_usage_percent < 80
              ? '⚠ Elevated load'
              : '⚠ High load'}
        </p>
      </div>

      <!-- Memory Details -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <Server class="h-4 w-4 text-muted-foreground" />
            <span class="text-sm font-medium">Memory</span>
          </div>
          <span class="text-sm font-bold">{metrics.memory_usage_percent.toFixed(1)}%</span>
        </div>
        <Progress value={metrics.memory_usage_percent} />
        <p class="text-xs text-muted-foreground">
          {formatBytes(metrics.memory_used_bytes)} von {formatBytes(metrics.memory_total_bytes)}
        </p>
      </div>

      <!-- Disk Details -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <HardDrive class="h-4 w-4 text-muted-foreground" />
            <span class="text-sm font-medium">Storage</span>
          </div>
          <span class="text-sm font-bold">{metrics.disk_usage_percent.toFixed(1)}%</span>
        </div>
        <Progress value={metrics.disk_usage_percent} />
        <p class="text-xs text-muted-foreground">
          {formatBytes(metrics.disk_used_bytes)} von {formatBytes(metrics.disk_total_bytes)}
        </p>
      </div>
    {/if}
  </Card.Content>
</Card.Root>
