<script lang="ts">
    import { onMount } from "svelte";
    import { auth } from "$lib/auth/store.svelte";
    import { listNodes, getClusterStats, type Node, type ClusterStats } from "$lib/api/nodes";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import NodeDetailSheet from "$lib/components/nodes/NodeDetailSheet.svelte";

    let nodes = $state<Node[]>([]);
    let stats = $state<ClusterStats | null>(null);
    let loading = $state(true);
    let error = $state<string | null>(null);
    let selectedNode = $state<Node | null>(null);
    let sheetOpen = $state(false);

    onMount(async () => {
        if (!auth.token) return;
        try {
            [nodes, stats] = await Promise.all([
                listNodes(auth.token),
                getClusterStats(auth.token),
            ]);
        } catch (e) {
            error = e instanceof Error ? e.message : "Failed to load nodes";
        } finally {
            loading = false;
        }
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
        return (bytes / 1_073_741_824).toFixed(1) + "G";
    }

    function statusClass(status: string): string {
        switch (status.toLowerCase()) {
            case "online": return "text-green-500";
            case "offline": return "text-red-500";
            case "degraded": return "text-yellow-500";
            default: return "text-muted-foreground";
        }
    }
</script>

<header class="flex h-16 shrink-0 items-center gap-2 px-4 border-b">
    <Sidebar.Trigger class="-ms-1" />
    <span class="text-sm text-muted-foreground">/</span>
    <span class="text-sm font-medium">Nodes</span>
</header>

<div class="flex flex-col gap-6 p-6">
    {#if stats}
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
            <div class="border rounded-lg p-4">
                <p class="text-xs text-muted-foreground mb-1">Nodes</p>
                <p class="text-2xl font-semibold">{stats.online_count} <span class="text-muted-foreground text-base font-normal">/ {stats.node_count}</span></p>
            </div>
            <div class="border rounded-lg p-4">
                <p class="text-xs text-muted-foreground mb-1">CPU Cores</p>
                <p class="text-2xl font-semibold">{stats.total_cpu_cores}</p>
            </div>
            <div class="border rounded-lg p-4">
                <p class="text-xs text-muted-foreground mb-1">Memory</p>
                <p class="text-2xl font-semibold">{bytesToGb(stats.used_memory_bytes)} <span class="text-muted-foreground text-base font-normal">/ {bytesToGb(stats.total_memory_bytes)}</span></p>
            </div>
            <div class="border rounded-lg p-4">
                <p class="text-xs text-muted-foreground mb-1">Disk</p>
                <p class="text-2xl font-semibold">{bytesToGb(stats.used_disk_bytes)} <span class="text-muted-foreground text-base font-normal">/ {bytesToGb(stats.total_disk_bytes)}</span></p>
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
