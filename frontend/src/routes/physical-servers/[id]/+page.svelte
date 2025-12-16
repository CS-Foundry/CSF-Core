<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import {
    getAgentDetails,
    getAgentMetrics,
    formatBytes,
    getStatusBadgeClass,
  } from "$lib/services/agents";
  import type { Agent, AgentMetrics } from "$lib/types/agent";

  let agent: Agent | null = null;
  let metrics: AgentMetrics[] = [];
  let loading = true;
  let error: string | null = null;
  let refreshInterval: number;

  $: agentId = $page.params.id;

  async function loadData() {
    try {
      const [agentData, metricsData] = await Promise.all([
        getAgentDetails(agentId),
        getAgentMetrics(agentId, 50),
      ]);
      agent = agentData;
      metrics = metricsData;
      loading = false;
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load agent data";
      loading = false;
    }
  }

  onMount(() => {
    loadData();
    // Auto-refresh every 10 seconds
    refreshInterval = setInterval(loadData, 10000);
  });

  onDestroy(() => {
    if (refreshInterval) {
      clearInterval(refreshInterval);
    }
  });

  function formatTimestamp(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleString("de-DE", {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  }

  function getMetricColor(percent: number): string {
    if (percent >= 90) return "bg-red-500";
    if (percent >= 75) return "bg-yellow-500";
    return "bg-green-500";
  }

  function getOsIcon(osType: string): string {
    switch (osType?.toLowerCase()) {
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
  <title>{agent?.name || "Agent Details"} - CSF Core</title>
</svelte:head>

<div class="container mx-auto p-6">
  <button
    on:click={() => goto("/physical-servers")}
    class="mb-6 text-blue-500 hover:text-blue-600 flex items-center gap-2"
  >
    ← Back to Physical Servers
  </button>

  {#if loading && !agent}
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
  {:else if agent}
    <!-- Agent Header -->
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 mb-6">
      <div class="flex items-start justify-between">
        <div class="flex items-center gap-4">
          <div class="text-5xl">{getOsIcon(agent.os_type)}</div>
          <div>
            <h1 class="text-3xl font-bold mb-1">{agent.name}</h1>
            <p class="text-gray-600 dark:text-gray-400">{agent.hostname}</p>
            <p class="text-sm text-gray-500 mt-1">
              {agent.os_type}
              {agent.os_version} • Agent v{agent.agent_version}
            </p>
          </div>
        </div>
        <span
          class="px-3 py-1 rounded-full text-sm font-medium {getStatusBadgeClass(
            agent.status
          )}"
        >
          {agent.status}
        </span>
      </div>

      <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mt-6">
        <div>
          <p class="text-sm text-gray-600 dark:text-gray-400">Agent ID</p>
          <p class="font-mono text-xs mt-1 break-all">{agent.id}</p>
        </div>
        <div>
          <p class="text-sm text-gray-600 dark:text-gray-400">Last Heartbeat</p>
          <p class="text-sm font-medium mt-1">
            {formatTimestamp(agent.last_heartbeat)}
          </p>
        </div>
        <div>
          <p class="text-sm text-gray-600 dark:text-gray-400">Registered</p>
          <p class="text-sm font-medium mt-1">
            {formatTimestamp(agent.created_at)}
          </p>
        </div>
        <div>
          <p class="text-sm text-gray-600 dark:text-gray-400">Updated</p>
          <p class="text-sm font-medium mt-1">
            {formatTimestamp(agent.updated_at)}
          </p>
        </div>
      </div>
    </div>

    <!-- Latest Metrics -->
    {#if metrics.length > 0}
      {@const latest = metrics[0]}
      <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
        <!-- CPU Usage -->
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6">
          <h3 class="text-lg font-semibold mb-4">CPU Usage</h3>
          <div class="relative pt-1">
            <div class="flex mb-2 items-center justify-between">
              <div>
                <span class="text-3xl font-bold"
                  >{latest.cpu_usage_percent.toFixed(1)}%</span
                >
              </div>
            </div>
            <div
              class="overflow-hidden h-4 text-xs flex rounded bg-gray-200 dark:bg-gray-700"
            >
              <div
                style="width: {latest.cpu_usage_percent}%"
                class="{getMetricColor(
                  latest.cpu_usage_percent
                )} transition-all duration-500"
              ></div>
            </div>
          </div>
        </div>

        <!-- Memory Usage -->
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6">
          <h3 class="text-lg font-semibold mb-4">Memory Usage</h3>
          <div class="relative pt-1">
            <div class="flex mb-2 items-center justify-between">
              <div>
                <span class="text-3xl font-bold"
                  >{latest.memory_usage_percent.toFixed(1)}%</span
                >
              </div>
            </div>
            <div
              class="overflow-hidden h-4 text-xs flex rounded bg-gray-200 dark:bg-gray-700"
            >
              <div
                style="width: {latest.memory_usage_percent}%"
                class="{getMetricColor(
                  latest.memory_usage_percent
                )} transition-all duration-500"
              ></div>
            </div>
            <div class="flex justify-between text-xs text-gray-500 mt-2">
              <span>{formatBytes(latest.memory_used_bytes)}</span>
              <span>{formatBytes(latest.memory_total_bytes)}</span>
            </div>
          </div>
        </div>

        <!-- Disk Usage -->
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6">
          <h3 class="text-lg font-semibold mb-4">Disk Usage</h3>
          <div class="relative pt-1">
            <div class="flex mb-2 items-center justify-between">
              <div>
                <span class="text-3xl font-bold"
                  >{latest.disk_usage_percent.toFixed(1)}%</span
                >
              </div>
            </div>
            <div
              class="overflow-hidden h-4 text-xs flex rounded bg-gray-200 dark:bg-gray-700"
            >
              <div
                style="width: {latest.disk_usage_percent}%"
                class="{getMetricColor(
                  latest.disk_usage_percent
                )} transition-all duration-500"
              ></div>
            </div>
            <div class="flex justify-between text-xs text-gray-500 mt-2">
              <span>{formatBytes(latest.disk_used_bytes)}</span>
              <span>{formatBytes(latest.disk_total_bytes)}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Metrics History -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6">
        <h3 class="text-lg font-semibold mb-4">Metrics History</h3>
        <div class="overflow-x-auto">
          <table
            class="min-w-full divide-y divide-gray-200 dark:divide-gray-700"
          >
            <thead>
              <tr>
                <th
                  class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  Timestamp
                </th>
                <th
                  class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  CPU
                </th>
                <th
                  class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  Memory
                </th>
                <th
                  class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  Disk
                </th>
                <th
                  class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  Network RX/TX
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
              {#each metrics.slice(0, 20) as metric}
                <tr class="hover:bg-gray-50 dark:hover:bg-gray-700">
                  <td class="px-4 py-3 whitespace-nowrap text-sm">
                    {formatTimestamp(metric.timestamp)}
                  </td>
                  <td class="px-4 py-3 whitespace-nowrap text-sm">
                    {metric.cpu_usage_percent.toFixed(1)}%
                  </td>
                  <td class="px-4 py-3 whitespace-nowrap text-sm">
                    {metric.memory_usage_percent.toFixed(1)}%
                    <span class="text-xs text-gray-500">
                      ({formatBytes(metric.memory_used_bytes)})
                    </span>
                  </td>
                  <td class="px-4 py-3 whitespace-nowrap text-sm">
                    {metric.disk_usage_percent.toFixed(1)}%
                    <span class="text-xs text-gray-500">
                      ({formatBytes(metric.disk_used_bytes)})
                    </span>
                  </td>
                  <td class="px-4 py-3 whitespace-nowrap text-sm">
                    {#if metric.network_rx_bytes !== undefined && metric.network_tx_bytes !== undefined}
                      ↓ {formatBytes(metric.network_rx_bytes)} / ↑ {formatBytes(
                        metric.network_tx_bytes
                      )}
                    {:else}
                      -
                    {/if}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    {:else}
      <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-12 text-center">
        <div class="text-6xl mb-4">📊</div>
        <h2 class="text-xl font-semibold mb-2">No Metrics Available</h2>
        <p class="text-gray-600 dark:text-gray-400">
          Waiting for the agent to send metrics data...
        </p>
      </div>
    {/if}
  {/if}
</div>
