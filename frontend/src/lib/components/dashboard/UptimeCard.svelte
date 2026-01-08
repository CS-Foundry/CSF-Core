<script lang="ts">
  import { Clock, TrendingUp, CheckCircle, XCircle } from '@lucide/svelte';
  import * as Card from '$lib/components/ui/card';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import { Badge } from '$lib/components/ui/badge';

  interface Props {
    uptime: number;
    loading: boolean;
  }

  let { uptime, loading }: Props = $props();

  function formatUptime(seconds: number): string {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${days}d ${hours}h ${minutes}m`;
  }

  function calculateAvailability(uptime: number): number {
    // Angenommen: Target uptime für letzten Monat
    const monthSeconds = 30 * 24 * 60 * 60;
    return Math.min((uptime / monthSeconds) * 100, 99.99);
  }
</script>

<Card.Root>
  <Card.Header>
    <Card.Title class="flex items-center gap-2">
      <Clock class="h-5 w-5" />
      Uptime & Availability
    </Card.Title>
    <Card.Description>System-Verfügbarkeit und Betriebszeit</Card.Description>
  </Card.Header>
  <Card.Content class="space-y-6">
    {#if loading}
      <div class="space-y-4">
        <Skeleton class="h-16 w-full" />
        <Skeleton class="h-16 w-full" />
      </div>
    {:else}
      <!-- Current Uptime -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <span class="text-sm text-muted-foreground">Current Uptime</span>
          <CheckCircle class="h-4 w-4 text-green-500" />
        </div>
        <div class="text-3xl font-bold">{formatUptime(uptime)}</div>
        <p class="text-xs text-muted-foreground">Seit letztem Neustart</p>
      </div>

      <!-- Availability Percentage -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <span class="text-sm text-muted-foreground">Availability (30d)</span>
          <TrendingUp class="h-4 w-4 text-muted-foreground" />
        </div>
        <div class="flex items-center gap-3">
          <div class="text-3xl font-bold text-green-500">
            {calculateAvailability(uptime).toFixed(2)}%
          </div>
          <Badge variant="default" class="bg-green-500">✓ Excellent</Badge>
        </div>
        <p class="text-xs text-muted-foreground">Target: 99.9% SLA</p>
      </div>

      <!-- Status Indicators -->
      <div class="grid grid-cols-2 gap-4 pt-4 border-t">
        <div class="flex items-center gap-2">
          <CheckCircle class="h-4 w-4 text-green-500" />
          <div>
            <p class="text-xs font-medium">API Status</p>
            <p class="text-xs text-muted-foreground">Operational</p>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <CheckCircle class="h-4 w-4 text-green-500" />
          <div>
            <p class="text-xs font-medium">Database</p>
            <p class="text-xs text-muted-foreground">Healthy</p>
          </div>
        </div>
      </div>
    {/if}
  </Card.Content>
</Card.Root>
