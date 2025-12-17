<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import {
    getAgentDetails,
    getAgentMetrics,
    formatBytes,
  } from "$lib/services/agents";
  import type { Agent, AgentMetrics } from "$lib/types/agent";
  import * as Card from "$lib/components/ui/card/index.js";
  import * as Chart from "$lib/components/ui/chart/index.js";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from "$lib/components/ui/table";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { PieChart, Text, AreaChart, Area, ChartClipPath } from "layerchart";
  import { scaleUtc } from "d3-scale";
  import { curveNatural } from "d3-shape";
  import ChartContainer from "$lib/components/ui/chart/chart-container.svelte";
  import { ArrowLeft, Activity, RefreshCw } from "@lucide/svelte";
  import Icon from "@iconify/svelte";
  import { cubicInOut } from "svelte/easing";

  let agent = $state<Agent | null>(null);
  let metrics = $state<AgentMetrics[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let refreshInterval: ReturnType<typeof setInterval>;

  const agentId = $derived($page.params.id);

  async function loadData() {
    if (!agentId) return;
    try {
      const [agentData, metricsData] = await Promise.all([
        getAgentDetails(agentId),
        getAgentMetrics(agentId, 100),
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

  function getOsIconName(osType: string): string {
    switch (osType?.toLowerCase()) {
      case "macos":
        return "bi:apple";
      case "linux":
        return "bi:ubuntu";
      case "windows":
        return "bi:windows";
      default:
        return "bi:pc-display";
    }
  }

  function getStatusColorClass(status: string): string {
    switch (status?.toLowerCase()) {
      case "online":
        return "bg-green-500 hover:bg-green-600 text-white";
      case "offline":
        return "bg-red-500 hover:bg-red-600 text-white";
      case "error":
      case "degraded":
        return "bg-yellow-500 hover:bg-yellow-600 text-white";
      case "stopped":
        return "bg-gray-500 hover:bg-gray-600 text-white";
      default:
        return "bg-gray-400 hover:bg-gray-500 text-white";
    }
  }

  // Chart configurations
  const cpuChartConfig = {
    usage: { label: "CPU Usage", color: "hsl(var(--chart-1))" },
  } satisfies Chart.ChartConfig;

  const memoryChartConfig = {
    usage: { label: "Memory Usage", color: "hsl(var(--chart-2))" },
  } satisfies Chart.ChartConfig;

  const diskChartConfig = {
    usage: { label: "Disk Usage", color: "hsl(var(--chart-3))" },
  } satisfies Chart.ChartConfig;

  const latest = $derived(metrics.length > 0 ? metrics[0] : null);

  const cpuChartData = $derived(
    metrics
      .slice(0, 50)
      .reverse()
      .map((m) => ({
        date: new Date(m.timestamp),
        usage: m.cpu_usage_percent,
      }))
  );

  const memoryChartData = $derived(
    metrics
      .slice(0, 50)
      .reverse()
      .map((m) => ({
        date: new Date(m.timestamp),
        usage: m.memory_usage_percent,
      }))
  );

  const diskChartData = $derived(
    metrics
      .slice(0, 50)
      .reverse()
      .map((m) => ({
        date: new Date(m.timestamp),
        usage: m.disk_usage_percent,
      }))
  );
</script>

<svelte:head>
  <title>{agent?.name || "Agent Details"} - CSF Core</title>
</svelte:head>

<div class="space-y-6">
  <!-- Back Button -->
  <Button variant="outline" onclick={() => goto("/physical-servers")}>
    <ArrowLeft class="mr-2 h-4 w-4" />
    Back to Physical Servers
  </Button>

  {#if loading && !agent}
    <div class="flex h-64 items-center justify-center">
      <RefreshCw class="h-8 w-8 animate-spin text-muted-foreground" />
    </div>
  {:else if error}
    <div
      class="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive"
    >
      {error}
    </div>
  {:else if agent}
    <!-- Agent Header Card -->
    <Card.Root>
      <Card.Header>
        <div class="flex items-start justify-between">
          <div class="flex items-center gap-4">
            <Icon
              icon={getOsIconName(agent.os_type)}
              class="h-16 w-16 text-muted-foreground"
            />
            <div>
              <Card.Title class="text-3xl">{agent.name}</Card.Title>
              <Card.Description class="mt-1 text-base">
                {agent.hostname}
              </Card.Description>
              <p class="mt-2 text-sm text-muted-foreground">
                {agent.os_type}
                {agent.os_version} • Agent v{agent.agent_version}
              </p>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onclick={loadData}
              disabled={loading}
            >
              <RefreshCw class="h-4 w-4 {loading ? 'animate-spin' : ''}" />
            </Button>
            <Badge class={getStatusColorClass(agent.status)}>
              <Activity class="mr-1 h-3 w-3" />
              {agent.status}
            </Badge>
          </div>
        </div>
      </Card.Header>
      <Card.Content>
        <div class="grid grid-cols-2 gap-4 md:grid-cols-4">
          <div>
            <p class="text-sm text-muted-foreground">Agent ID</p>
            <p class="mt-1 break-all font-mono text-xs">{agent.id}</p>
          </div>
          <div>
            <p class="text-sm text-muted-foreground">Last Heartbeat</p>
            <p class="mt-1 text-sm font-medium">
              {formatTimestamp(agent.last_heartbeat)}
            </p>
          </div>
          <div>
            <p class="text-sm text-muted-foreground">Registered</p>
            <p class="mt-1 text-sm font-medium">
              {formatTimestamp(agent.created_at)}
            </p>
          </div>
          <div>
            <p class="text-sm text-muted-foreground">Updated</p>
            <p class="mt-1 text-sm font-medium">
              {formatTimestamp(agent.updated_at)}
            </p>
          </div>
        </div>
      </Card.Content>
    </Card.Root>

    {#if latest}
      <!-- Current Metrics - 3 Radial Charts -->
      <div class="grid grid-cols-1 gap-6 md:grid-cols-3">
        <!-- CPU Usage -->
        <Card.Root class="flex flex-col">
          <Card.Header class="items-center pb-0">
            <Card.Title>CPU Usage</Card.Title>
            <Card.Description>Current utilization</Card.Description>
          </Card.Header>
          <Card.Content class="flex flex-1 items-center pb-0">
            <Chart.Container
              config={cpuChartConfig}
              class="mx-auto aspect-square max-h-[250px]"
            >
              <PieChart
                data={[
                  {
                    name: "used",
                    value: latest.cpu_usage_percent,
                    color: cpuChartConfig.usage.color,
                  },
                  {
                    name: "free",
                    value: 100 - latest.cpu_usage_percent,
                    color: "hsl(var(--muted))",
                  },
                ]}
                key="name"
                value="value"
                c="color"
                innerRadius={76}
                padding={29}
                range={[-90, 90]}
                props={{ pie: { sort: null } }}
                cornerRadius={4}
              >
                {#snippet aboveMarks()}
                  <Text
                    value={`${latest.cpu_usage_percent.toFixed(1)}%`}
                    textAnchor="middle"
                    verticalAnchor="middle"
                    class="fill-foreground text-2xl! font-bold"
                    dy={-24}
                  />
                  <Text
                    value="CPU"
                    textAnchor="middle"
                    verticalAnchor="middle"
                    class="fill-muted-foreground! text-muted-foreground"
                    dy={-4}
                  />
                {/snippet}
                {#snippet tooltip()}
                  <Chart.Tooltip hideLabel />
                {/snippet}
              </PieChart>
            </Chart.Container>
          </Card.Content>
        </Card.Root>

        <!-- Memory Usage -->
        <Card.Root class="flex flex-col">
          <Card.Header class="items-center pb-0">
            <Card.Title>Memory Usage</Card.Title>
            <Card.Description>
              {formatBytes(latest.memory_used_bytes)} / {formatBytes(
                latest.memory_total_bytes
              )}
            </Card.Description>
          </Card.Header>
          <Card.Content class="flex flex-1 items-center pb-0">
            <Chart.Container
              config={memoryChartConfig}
              class="mx-auto aspect-square max-h-[250px]"
            >
              <PieChart
                data={[
                  {
                    name: "used",
                    value: latest.memory_usage_percent,
                    color: memoryChartConfig.usage.color,
                  },
                  {
                    name: "free",
                    value: 100 - latest.memory_usage_percent,
                    color: "hsl(var(--muted))",
                  },
                ]}
                key="name"
                value="value"
                c="color"
                innerRadius={76}
                padding={29}
                range={[-90, 90]}
                props={{ pie: { sort: null } }}
                cornerRadius={4}
              >
                {#snippet aboveMarks()}
                  <Text
                    value={`${latest.memory_usage_percent.toFixed(1)}%`}
                    textAnchor="middle"
                    verticalAnchor="middle"
                    class="fill-foreground text-2xl! font-bold"
                    dy={-24}
                  />
                  <Text
                    value="Memory"
                    textAnchor="middle"
                    verticalAnchor="middle"
                    class="fill-muted-foreground! text-muted-foreground"
                    dy={-4}
                  />
                {/snippet}
                {#snippet tooltip()}
                  <Chart.Tooltip hideLabel />
                {/snippet}
              </PieChart>
            </Chart.Container>
          </Card.Content>
        </Card.Root>

        <!-- Disk Usage -->
        <Card.Root class="flex flex-col">
          <Card.Header class="items-center pb-0">
            <Card.Title>Disk Usage</Card.Title>
            <Card.Description>
              {formatBytes(latest.disk_used_bytes)} / {formatBytes(
                latest.disk_total_bytes
              )}
            </Card.Description>
          </Card.Header>
          <Card.Content class="flex flex-1 items-center pb-0">
            <Chart.Container
              config={diskChartConfig}
              class="mx-auto aspect-square max-h-[250px]"
            >
              <PieChart
                data={[
                  {
                    name: "used",
                    value: latest.disk_usage_percent,
                    color: diskChartConfig.usage.color,
                  },
                  {
                    name: "free",
                    value: 100 - latest.disk_usage_percent,
                    color: "hsl(var(--muted))",
                  },
                ]}
                key="name"
                value="value"
                c="color"
                innerRadius={76}
                padding={29}
                range={[-90, 90]}
                props={{ pie: { sort: null } }}
                cornerRadius={4}
              >
                {#snippet aboveMarks()}
                  <Text
                    value={`${latest.disk_usage_percent.toFixed(1)}%`}
                    textAnchor="middle"
                    verticalAnchor="middle"
                    class="fill-foreground text-2xl! font-bold"
                    dy={-24}
                  />
                  <Text
                    value="Disk"
                    textAnchor="middle"
                    verticalAnchor="middle"
                    class="fill-muted-foreground! text-muted-foreground"
                    dy={-4}
                  />
                {/snippet}
                {#snippet tooltip()}
                  <Chart.Tooltip hideLabel />
                {/snippet}
              </PieChart>
            </Chart.Container>
          </Card.Content>
        </Card.Root>
      </div>

      <!-- Metrics History - Area Charts and Table -->
      <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <!-- Left Column - Area Charts -->
        <div class="space-y-6">
          <!-- CPU Area Chart -->
          <Card.Root>
            <Card.Header>
              <Card.Title>CPU History</Card.Title>
              <Card.Description>Last 50 measurements</Card.Description>
            </Card.Header>
            <Card.Content>
              <ChartContainer
                config={cpuChartConfig}
                class="aspect-auto h-[200px] w-full"
              >
                <AreaChart
                  data={cpuChartData}
                  x="date"
                  xScale={scaleUtc()}
                  series={[
                    {
                      key: "usage",
                      label: "CPU %",
                      color: cpuChartConfig.usage.color,
                    },
                  ]}
                  props={{
                    area: {
                      curve: curveNatural,
                      "fill-opacity": 0.4,
                      line: { class: "stroke-2" },
                    },
                    xAxis: {
                      format: (v) =>
                        v.toLocaleTimeString("de-DE", {
                          hour: "2-digit",
                          minute: "2-digit",
                        }),
                    },
                    yAxis: { format: (v) => `${v}%` },
                  }}
                >
                  {#snippet marks({ series, getAreaProps })}
                    <defs>
                      <linearGradient id="fillCPU" x1="0" y1="0" x2="0" y2="1">
                        <stop
                          offset="5%"
                          stop-color="var(--color-usage)"
                          stop-opacity={0.8}
                        />
                        <stop
                          offset="95%"
                          stop-color="var(--color-usage)"
                          stop-opacity={0.1}
                        />
                      </linearGradient>
                    </defs>
                    <ChartClipPath
                      initialWidth={0}
                      motion={{
                        width: {
                          type: "tween",
                          duration: 1000,
                          easing: cubicInOut,
                        },
                      }}
                    >
                      {#each series as s, i (s.key)}
                        <Area {...getAreaProps(s, i)} fill="url(#fillCPU)" />
                      {/each}
                    </ChartClipPath>
                  {/snippet}
                  {#snippet tooltip()}
                    <Chart.Tooltip
                      labelFormatter={(v: Date) => {
                        return v.toLocaleString("de-DE");
                      }}
                      indicator="line"
                    />
                  {/snippet}
                </AreaChart>
              </ChartContainer>
            </Card.Content>
          </Card.Root>

          <!-- Memory Area Chart -->
          <Card.Root>
            <Card.Header>
              <Card.Title>Memory History</Card.Title>
              <Card.Description>Last 50 measurements</Card.Description>
            </Card.Header>
            <Card.Content>
              <ChartContainer
                config={memoryChartConfig}
                class="aspect-auto h-[200px] w-full"
              >
                <AreaChart
                  data={memoryChartData}
                  x="date"
                  xScale={scaleUtc()}
                  series={[
                    {
                      key: "usage",
                      label: "Memory %",
                      color: memoryChartConfig.usage.color,
                    },
                  ]}
                  props={{
                    area: {
                      curve: curveNatural,
                      "fill-opacity": 0.4,
                      line: { class: "stroke-2" },
                    },
                    xAxis: {
                      format: (v) =>
                        v.toLocaleTimeString("de-DE", {
                          hour: "2-digit",
                          minute: "2-digit",
                        }),
                    },
                    yAxis: { format: (v) => `${v}%` },
                  }}
                >
                  {#snippet marks({ series, getAreaProps })}
                    <defs>
                      <linearGradient
                        id="fillMemory"
                        x1="0"
                        y1="0"
                        x2="0"
                        y2="1"
                      >
                        <stop
                          offset="5%"
                          stop-color="var(--color-usage)"
                          stop-opacity={0.8}
                        />
                        <stop
                          offset="95%"
                          stop-color="var(--color-usage)"
                          stop-opacity={0.1}
                        />
                      </linearGradient>
                    </defs>
                    <ChartClipPath
                      initialWidth={0}
                      motion={{
                        width: {
                          type: "tween",
                          duration: 1000,
                          easing: cubicInOut,
                        },
                      }}
                    >
                      {#each series as s, i (s.key)}
                        <Area {...getAreaProps(s, i)} fill="url(#fillMemory)" />
                      {/each}
                    </ChartClipPath>
                  {/snippet}
                  {#snippet tooltip()}
                    <Chart.Tooltip
                      labelFormatter={(v: Date) => {
                        return v.toLocaleString("de-DE");
                      }}
                      indicator="line"
                    />
                  {/snippet}
                </AreaChart>
              </ChartContainer>
            </Card.Content>
          </Card.Root>

          <!-- Disk Area Chart -->
          <Card.Root>
            <Card.Header>
              <Card.Title>Disk History</Card.Title>
              <Card.Description>Last 50 measurements</Card.Description>
            </Card.Header>
            <Card.Content>
              <ChartContainer
                config={diskChartConfig}
                class="aspect-auto h-[200px] w-full"
              >
                <AreaChart
                  data={diskChartData}
                  x="date"
                  xScale={scaleUtc()}
                  series={[
                    {
                      key: "usage",
                      label: "Disk %",
                      color: diskChartConfig.usage.color,
                    },
                  ]}
                  props={{
                    area: {
                      curve: curveNatural,
                      "fill-opacity": 0.4,
                      line: { class: "stroke-2" },
                    },
                    xAxis: {
                      format: (v) =>
                        v.toLocaleTimeString("de-DE", {
                          hour: "2-digit",
                          minute: "2-digit",
                        }),
                    },
                    yAxis: { format: (v) => `${v}%` },
                  }}
                >
                  {#snippet marks({ series, getAreaProps })}
                    <defs>
                      <linearGradient id="fillDisk" x1="0" y1="0" x2="0" y2="1">
                        <stop
                          offset="5%"
                          stop-color="var(--color-usage)"
                          stop-opacity={0.8}
                        />
                        <stop
                          offset="95%"
                          stop-color="var(--color-usage)"
                          stop-opacity={0.1}
                        />
                      </linearGradient>
                    </defs>
                    <ChartClipPath
                      initialWidth={0}
                      motion={{
                        width: {
                          type: "tween",
                          duration: 1000,
                          easing: cubicInOut,
                        },
                      }}
                    >
                      {#each series as s, i (s.key)}
                        <Area {...getAreaProps(s, i)} fill="url(#fillDisk)" />
                      {/each}
                    </ChartClipPath>
                  {/snippet}
                  {#snippet tooltip()}
                    <Chart.Tooltip
                      labelFormatter={(v: Date) => {
                        return v.toLocaleString("de-DE");
                      }}
                      indicator="line"
                    />
                  {/snippet}
                </AreaChart>
              </ChartContainer>
            </Card.Content>
          </Card.Root>
        </div>

        <!-- Right Column - Data Table -->
        <Card.Root class="flex flex-col">
          <Card.Header>
            <Card.Title>Metrics Data</Card.Title>
            <Card.Description>Detailed measurements history</Card.Description>
          </Card.Header>
          <Card.Content class="flex-1">
            <div class="max-h-[660px] overflow-auto rounded-md border">
              <Table>
                <TableHeader class="sticky top-0 bg-background">
                  <TableRow>
                    <TableHead class="w-[180px]">Timestamp</TableHead>
                    <TableHead>CPU</TableHead>
                    <TableHead>Memory</TableHead>
                    <TableHead>Disk</TableHead>
                    <TableHead>Network</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {#each metrics.slice(0, 100) as metric}
                    <TableRow>
                      <TableCell class="text-xs">
                        {formatTimestamp(metric.timestamp)}
                      </TableCell>
                      <TableCell class="text-xs">
                        {metric.cpu_usage_percent.toFixed(1)}%
                      </TableCell>
                      <TableCell class="text-xs">
                        <div>{metric.memory_usage_percent.toFixed(1)}%</div>
                        <div class="text-muted-foreground">
                          {formatBytes(metric.memory_used_bytes)}
                        </div>
                      </TableCell>
                      <TableCell class="text-xs">
                        <div>{metric.disk_usage_percent.toFixed(1)}%</div>
                        <div class="text-muted-foreground">
                          {formatBytes(metric.disk_used_bytes)}
                        </div>
                      </TableCell>
                      <TableCell class="text-xs">
                        {#if metric.network_rx_bytes !== undefined && metric.network_tx_bytes !== undefined}
                          <div>↓ {formatBytes(metric.network_rx_bytes)}</div>
                          <div>↑ {formatBytes(metric.network_tx_bytes)}</div>
                        {:else}
                          -
                        {/if}
                      </TableCell>
                    </TableRow>
                  {/each}
                </TableBody>
              </Table>
            </div>
          </Card.Content>
        </Card.Root>
      </div>
    {:else}
      <Card.Root>
        <Card.Content class="flex h-64 flex-col items-center justify-center">
          <Activity class="mb-4 h-12 w-12 text-muted-foreground" />
          <Card.Title class="mb-2">No Metrics Available</Card.Title>
          <Card.Description>
            Waiting for the agent to send metrics data...
          </Card.Description>
        </Card.Content>
      </Card.Root>
    {/if}
  {/if}
</div>
