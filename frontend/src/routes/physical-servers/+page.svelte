<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import {
    listAgents,
    getStatusBadgeClass,
    formatBytes,
  } from "$lib/services/agents";
  import type { Agent } from "$lib/types/agent";
  import { Button } from "$lib/components/ui/button";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from "$lib/components/ui/table";
  import { Badge } from "$lib/components/ui/badge";
  import { Server, Eye, RefreshCw, Activity } from "@lucide/svelte";

  let agents = $state<Agent[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function loadAgents() {
    loading = true;
    error = null;
    try {
      agents = await listAgents();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load agents";
    } finally {
      loading = false;
    }
  }

  function handleAgentClick(agentId: string) {
    goto(`/physical-servers/${agentId}`);
  }

  function formatLastSeen(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffMins < 1440) return `${Math.floor(diffMins / 60)}h ago`;
    return `${Math.floor(diffMins / 1440)}d ago`;
  }

  function getOsIcon(osType: string): string {
    switch (osType.toLowerCase()) {
      case "macos":
        return "🍎";
      case "linux":
        return "🐧";
      case "windows":
        return "🪟";
      default:
        return "💻";
    }
  }

  function getStatusVariant(
    status: string
  ): "default" | "secondary" | "destructive" | "outline" {
    switch (status.toLowerCase()) {
      case "online":
        return "default";
      case "offline":
        return "destructive";
      case "degraded":
        return "secondary";
      default:
        return "outline";
    }
  }

  onMount(() => {
    loadAgents();
  });
</script>

<svelte:head>
  <title>Physical Servers - CSF Core</title>
</svelte:head>

<div class="mb-6 flex items-center justify-between">
  <div>
    <h2 class="text-2xl font-bold">Physical Servers</h2>
    <p class="text-muted-foreground">
      Monitor and manage your physical server infrastructure
    </p>
  </div>
  <div class="flex items-center gap-2">
    <div class="text-sm text-muted-foreground">
      {agents.length}
      {agents.length === 1 ? "Server" : "Servers"} registered
    </div>
    <Button variant="outline" size="sm" onclick={loadAgents} disabled={loading}>
      <RefreshCw class="h-4 w-4 {loading ? 'animate-spin' : ''}" />
    </Button>
  </div>
</div>

{#if error}
  <div
    class="mb-4 rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive"
  >
    {error}
  </div>
{/if}

{#if loading && agents.length === 0}
  <div class="flex h-64 items-center justify-center">
    <div class="flex items-center gap-2">
      <RefreshCw class="h-6 w-6 animate-spin" />
      <span class="text-muted-foreground">Loading servers...</span>
    </div>
  </div>
{:else if agents.length === 0}
  <div
    class="flex h-64 flex-col items-center justify-center rounded-lg border border-dashed"
  >
    <Server class="mb-4 h-12 w-12 text-muted-foreground" />
    <h3 class="mb-2 text-xl font-semibold">No Agents Found</h3>
    <p class="text-muted-foreground">
      No physical servers are currently registered. Install and start an agent
      to begin monitoring.
    </p>
  </div>
{:else}
  <div class="rounded-md border">
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Server</TableHead>
          <TableHead>Status</TableHead>
          <TableHead>Operating System</TableHead>
          <TableHead>Agent Version</TableHead>
          <TableHead>Last Seen</TableHead>
          <TableHead>Tags</TableHead>
          <TableHead class="text-right">Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {#each agents as agent (agent.id)}
          <TableRow class="cursor-pointer hover:bg-muted/50">
            <TableCell class="font-medium">
              <div class="flex items-center gap-3">
                <div class="text-2xl">
                  {getOsIcon(agent.os_type)}
                </div>
                <div>
                  <div class="font-semibold">{agent.name}</div>
                  <div class="text-sm text-muted-foreground">
                    {agent.hostname}
                  </div>
                </div>
              </div>
            </TableCell>
            <TableCell>
              <Badge variant={getStatusVariant(agent.status)}>
                <Activity class="mr-1 h-3 w-3" />
                {agent.status}
              </Badge>
            </TableCell>
            <TableCell>
              <div class="text-sm">
                <div>{agent.os_type}</div>
                <div class="text-muted-foreground">{agent.os_version}</div>
              </div>
            </TableCell>
            <TableCell>
              <code class="rounded bg-muted px-2 py-1 text-xs font-mono">
                {agent.agent_version}
              </code>
            </TableCell>
            <TableCell>
              <div class="text-sm">
                {formatLastSeen(agent.last_heartbeat)}
              </div>
              <div class="text-xs text-muted-foreground">
                {new Date(agent.last_heartbeat).toLocaleString()}
              </div>
            </TableCell>
            <TableCell>
              {#if agent.tags && Object.keys(agent.tags).length > 0}
                <div class="flex flex-wrap gap-1">
                  {#each Object.entries(agent.tags).slice(0, 2) as [key, value]}
                    <Badge variant="outline" class="text-xs">
                      {key}: {value}
                    </Badge>
                  {/each}
                  {#if Object.keys(agent.tags).length > 2}
                    <Badge variant="outline" class="text-xs">
                      +{Object.keys(agent.tags).length - 2}
                    </Badge>
                  {/if}
                </div>
              {:else}
                <span class="text-muted-foreground">-</span>
              {/if}
            </TableCell>
            <TableCell class="text-right">
              <Button
                variant="ghost"
                size="sm"
                onclick={() => handleAgentClick(agent.id)}
              >
                <Eye class="h-4 w-4" />
              </Button>
            </TableCell>
          </TableRow>
        {/each}
      </TableBody>
    </Table>
  </div>
{/if}
