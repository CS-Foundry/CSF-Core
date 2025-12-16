<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import {
    listAgents,
    getStatusBadgeClass,
    formatBytes,
  } from "$lib/services/agents";
  import type { Agent } from "$lib/types/agent";

  let agents: Agent[] = [];
  let loading = true;
  let error: string | null = null;

  onMount(async () => {
    try {
      agents = await listAgents();
      loading = false;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load agents";
      loading = false;
    }
  });

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
</script>

<svelte:head>
  <title>Physical Servers - CSF Core</title>
</svelte:head>

<div class="container mx-auto p-6">
  <div class="flex justify-between items-center mb-6">
    <h1 class="text-3xl font-bold">Physical Servers</h1>
    <div class="text-sm text-gray-500">
      {agents.length}
      {agents.length === 1 ? "Agent" : "Agents"} registered
    </div>
  </div>

  {#if loading}
    <div class="flex justify-center items-center h-64">
      <div
        class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"
      ></div>
    </div>
  {:else if error}
    <div
      class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4"
    >
      <p class="text-red-800 dark:text-red-200">{error}</p>
    </div>
  {:else if agents.length === 0}
    <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-12 text-center">
      <div class="text-6xl mb-4">🖥️</div>
      <h2 class="text-xl font-semibold mb-2">No Agents Found</h2>
      <p class="text-gray-600 dark:text-gray-400">
        No physical servers are currently registered. Install and start an agent
        to begin monitoring.
      </p>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      {#each agents as agent}
        <button
          on:click={() => handleAgentClick(agent.id)}
          class="bg-white dark:bg-gray-800 rounded-lg shadow-md hover:shadow-lg transition-shadow p-6 text-left border border-gray-200 dark:border-gray-700 hover:border-blue-500 dark:hover:border-blue-400"
        >
          <!-- Header -->
          <div class="flex items-start justify-between mb-4">
            <div class="flex items-center gap-3">
              <div class="text-3xl">
                {getOsIcon(agent.os_type)}
              </div>
              <div>
                <h3 class="font-semibold text-lg">{agent.name}</h3>
                <p class="text-sm text-gray-500">{agent.hostname}</p>
              </div>
            </div>
            <span
              class="px-2 py-1 rounded-full text-xs font-medium {getStatusBadgeClass(
                agent.status
              )}"
            >
              {agent.status}
            </span>
          </div>

          <!-- System Info -->
          <div class="space-y-2 mb-4">
            <div class="flex justify-between text-sm">
              <span class="text-gray-600 dark:text-gray-400">OS:</span>
              <span class="font-medium">{agent.os_type} {agent.os_version}</span
              >
            </div>
            <div class="flex justify-between text-sm">
              <span class="text-gray-600 dark:text-gray-400">Version:</span>
              <span class="font-medium">{agent.agent_version}</span>
            </div>
            <div class="flex justify-between text-sm">
              <span class="text-gray-600 dark:text-gray-400">Last Seen:</span>
              <span class="font-medium"
                >{formatLastSeen(agent.last_heartbeat)}</span
              >
            </div>
          </div>

          <!-- Tags -->
          {#if agent.tags && Object.keys(agent.tags).length > 0}
            <div class="flex flex-wrap gap-2 mt-4">
              {#each Object.entries(agent.tags) as [key, value]}
                <span
                  class="px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-xs rounded"
                >
                  {key}: {value}
                </span>
              {/each}
            </div>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  button {
    cursor: pointer;
  }
</style>
