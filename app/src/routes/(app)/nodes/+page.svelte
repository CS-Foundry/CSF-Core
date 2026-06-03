<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { auth } from "$lib/auth/store.svelte";
    import { listNodes, getClusterStats, type Node, type ClusterStats } from "$lib/api/nodes";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import NodeDetailSheet from "$lib/components/nodes/NodeDetailSheet.svelte";

    let nodes = $state<Node[]>([]);
    let stats = $state<ClusterStats | null>(null);
    let loading = $state(true);
    let error = $state<string | null>(null);
    let selectedNode = $state<Node | null>(null);
    let sheetOpen = $state(false);

    let onlineHistory = $state<number[]>([]);
    let cpuHistory = $state<number[]>([]);
    let memHistory = $state<number[]>([]);

    let pollInterval: ReturnType<typeof setInterval> | null = null;
    let refreshing = $state(false);
    let addNodePulsing = $state(false);

    async function fetchStats() {
        if (!auth.token) return;
        try {
            const s = await getClusterStats(auth.token);
            stats = s;

            const onlinePct = s.node_count > 0 ? (s.online_count / s.node_count) * 100 : 0;
            onlineHistory = [...onlineHistory.slice(-23), onlinePct];

            const cpuPct = s.avg_cpu_usage_percent ?? 0;
            cpuHistory = [...cpuHistory.slice(-23), cpuPct];

            const memPct = s.total_memory_bytes > 0
                ? (s.used_memory_bytes / s.total_memory_bytes) * 100
                : 0;
            memHistory = [...memHistory.slice(-23), memPct];
        } catch {
            // non-fatal poll failure
        }
    }

    async function refresh() {
        if (!auth.token || refreshing) return;
        refreshing = true;
        try {
            [nodes] = await Promise.all([
                listNodes(auth.token),
                fetchStats(),
            ]);
        } catch (e) {
            error = e instanceof Error ? e.message : "Failed to refresh";
        } finally {
            refreshing = false;
        }
    }

    onMount(async () => {
        if (!auth.token) return;
        try {
            [nodes] = await Promise.all([
                listNodes(auth.token),
                fetchStats(),
            ]);
        } catch (e) {
            error = e instanceof Error ? e.message : "Failed to load nodes";
        } finally {
            loading = false;
        }
        pollInterval = setInterval(fetchStats, 30_000);
    });

    onDestroy(() => {
        if (pollInterval) clearInterval(pollInterval);
    });

    function openNode(node: Node) {
        selectedNode = node;
        sheetOpen = true;
    }

    function closeSheet() {
        sheetOpen = false;
        selectedNode = null;
    }

    function bytesToGb(bytes: number | null): string {
        if (bytes == null) return "-";
        return (bytes / 1_073_741_824).toFixed(1) + " GB";
    }

    function statusClass(status: string): string {
        switch (status.toLowerCase()) {
            case "online": return "text-green-500";
            case "offline": return "text-red-500";
            case "degraded": return "text-yellow-500";
            default: return "text-muted-foreground";
        }
    }

    function sparklinePath(values: number[], width: number, height: number): string {
        if (values.length < 2) return "";
        const max = Math.max(...values, 1);
        const step = width / (values.length - 1);
        return values
            .map((v, i) => {
                const x = i * step;
                const y = height - (v / max) * height;
                return `${i === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`;
            })
            .join(" ");
    }

    function memPct(): number {
        if (!stats || stats.total_memory_bytes === 0) return 0;
        return (stats.used_memory_bytes / stats.total_memory_bytes) * 100;
    }

    function cpuPct(): number {
        return stats?.avg_cpu_usage_percent ?? 0;
    }
</script>

<header class="flex h-16 shrink-0 items-center gap-2 px-4 border-b">
    <Sidebar.Trigger class="-ms-1" />
    <span class="text-sm text-muted-foreground">/</span>
    <span class="text-sm font-medium">Nodes</span>
</header>

<div class="flex flex-col gap-6 p-6">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-xl font-semibold tracking-tight">Nodes</h1>
            <p class="text-sm text-muted-foreground mt-0.5">Manage and monitor your cluster nodes</p>
        </div>
        <div class="flex items-center gap-2">
            <Button variant="outline" size="sm">
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"/>
                </svg>
                Filter
            </Button>
            <Button
                variant="outline"
                size="sm"
                onclick={refresh}
                disabled={refreshing}
                class={refreshing ? "opacity-60" : ""}
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class={refreshing ? "animate-spin" : ""}
                >
                    <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
                    <path d="M21 3v5h-5"/>
                    <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/>
                    <path d="M8 16H3v5"/>
                </svg>
                Refresh
            </Button>
            <Button
                size="sm"
                class="relative overflow-hidden"
                onclick={() => { addNodePulsing = true; setTimeout(() => { addNodePulsing = false; }, 600); }}
            >
                {#if addNodePulsing}
                    <span class="absolute inset-0 rounded-md animate-ping bg-primary/30 pointer-events-none"></span>
                {/if}
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="12" y1="5" x2="12" y2="19"/>
                    <line x1="5" y1="12" x2="19" y2="12"/>
                </svg>
                Add Node
            </Button>
        </div>
    </div>

    {#if stats}
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
            <div class="border rounded-lg p-4">
                <p class="text-xs text-muted-foreground mb-1">Total Nodes</p>
                <p class="text-2xl font-semibold">{stats.node_count}</p>
            </div>

            <div class="border rounded-lg p-4 flex flex-col gap-2">
                <p class="text-xs text-muted-foreground">Healthy</p>
                <div class="flex items-end justify-between gap-2">
                    <div class="flex items-center gap-2">
                        <span class="inline-block w-2 h-2 rounded-full bg-green-500 shrink-0"></span>
                        <p class="text-2xl font-semibold">{stats.online_count}</p>
                    </div>
                    {#if onlineHistory.length >= 2}
                        <svg width="72" height="32" class="text-green-500 shrink-0">
                            <path
                                d={sparklinePath(onlineHistory, 72, 28)}
                                fill="none"
                                stroke="currentColor"
                                stroke-width="1.5"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            />
                        </svg>
                    {/if}
                </div>
            </div>

            <div class="border rounded-lg p-4 flex flex-col gap-2">
                <p class="text-xs text-muted-foreground">vCPU Capacity</p>
                <div class="flex items-end justify-between gap-2">
                    <div>
                        <p class="text-2xl font-semibold">{stats.total_cpu_cores}</p>
                        <p class="text-xs text-muted-foreground mt-0.5">{cpuPct().toFixed(1)}% used</p>
                    </div>
                    {#if cpuHistory.length >= 2}
                        <svg width="72" height="32" class="text-primary shrink-0">
                            <path
                                d={sparklinePath(cpuHistory, 72, 28)}
                                fill="none"
                                stroke="currentColor"
                                stroke-width="1.5"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            />
                        </svg>
                    {/if}
                </div>
            </div>

            <div class="border rounded-lg p-4 flex flex-col gap-2">
                <p class="text-xs text-muted-foreground">Memory</p>
                <div class="flex items-end justify-between gap-2">
                    <div>
                        <p class="text-2xl font-semibold">{bytesToGb(stats.used_memory_bytes)}</p>
                        <p class="text-xs text-muted-foreground mt-0.5">/ {bytesToGb(stats.total_memory_bytes)}</p>
                    </div>
                    {#if memHistory.length >= 2}
                        <svg width="72" height="32" class="text-primary shrink-0">
                            <path
                                d={sparklinePath(memHistory, 72, 28)}
                                fill="none"
                                stroke="currentColor"
                                stroke-width="1.5"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            />
                        </svg>
                    {/if}
                </div>
            </div>
        </div>
    {/if}

    <div class="border rounded-lg overflow-hidden">
        <table class="w-full text-sm">
            <thead class="bg-muted/50">
                <tr>
                    <th class="text-left px-4 py-3 font-medium text-muted-foreground">ID</th>
                    <th class="text-left px-4 py-3 font-medium text-muted-foreground">Hostname</th>
                    <th class="text-left px-4 py-3 font-medium text-muted-foreground">IP</th>
                    <th class="text-left px-4 py-3 font-medium text-muted-foreground">OS</th>
                    <th class="text-left px-4 py-3 font-medium text-muted-foreground">Arch</th>
                    <th class="text-left px-4 py-3 font-medium text-muted-foreground">Version</th>
                    <th class="text-left px-4 py-3 font-medium text-muted-foreground">Status</th>
                    <th class="text-left px-4 py-3 font-medium text-muted-foreground">Heartbeat</th>
                </tr>
            </thead>
            <tbody>
                {#if loading}
                    <tr>
                        <td colspan="8" class="px-4 py-8 text-center text-muted-foreground">Loading...</td>
                    </tr>
                {:else if error}
                    <tr>
                        <td colspan="8" class="px-4 py-8 text-center text-destructive">{error}</td>
                    </tr>
                {:else if nodes.length === 0}
                    <tr>
                        <td colspan="8" class="px-4 py-8 text-center text-muted-foreground">No nodes registered</td>
                    </tr>
                {:else}
                    {#each nodes as node (node.id)}
                        <tr
                            class="border-t hover:bg-muted/30 transition-colors cursor-pointer"
                            onclick={() => openNode(node)}
                        >
                            <td class="px-4 py-3 font-mono text-xs text-muted-foreground">{node.id.slice(0, 8)}</td>
                            <td class="px-4 py-3 font-medium">{node.hostname}</td>
                            <td class="px-4 py-3 text-muted-foreground">{node.ip_address ?? "-"}</td>
                            <td class="px-4 py-3 text-muted-foreground">{node.os_type} {node.os_version}</td>
                            <td class="px-4 py-3 text-muted-foreground">{node.architecture}</td>
                            <td class="px-4 py-3 text-muted-foreground">{node.agent_version}</td>
                            <td class="px-4 py-3">
                                <span class="font-medium {statusClass(node.status)}">{node.status}</span>
                            </td>
                            <td class="px-4 py-3 text-muted-foreground text-xs">
                                {node.last_heartbeat ? node.last_heartbeat.slice(0, 16) : "never"}
                            </td>
                        </tr>
                    {/each}
                {/if}
            </tbody>
        </table>
    </div>
</div>

<NodeDetailSheet node={selectedNode} open={sheetOpen} onClose={closeSheet} />
