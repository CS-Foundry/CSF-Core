<script lang="ts">
    import { onDestroy } from "svelte";
    import { page } from "$app/stores";
    import { goto } from "$app/navigation";
    import { auth } from "$lib/auth/store.svelte";
    import {
        listResourceGroupWorkloads,
        stopWorkload,
        restartWorkload,
        deleteWorkload,
        type Workload,
    } from "$lib/api/resource-groups";
    import { getNode } from "$lib/api/nodes";
    import VncConsole from "$lib/components/vnc-console.svelte";
    import Icon from "@iconify/svelte";
    import { Button } from "$lib/components/ui/button/index.js";
    import StatusBadge from "$lib/components/status-badge.svelte";

    const rgId: string = $page.params.id;
    const workloadId: string = $page.params.workloadId;

    let workload = $state<Workload | null>(null);
    let loading = $state(true);
    let error = $state<string | null>(null);
    let actionError = $state<string | null>(null);
    let actionBusy = $state(false);
    let nodeIp = $state<string | null>(null);

    async function load() {
        if (!auth.token) return;
        try {
            const workloads = await listResourceGroupWorkloads(auth.token, rgId);
            const found = workloads.find((w) => w.id === workloadId) ?? null;
            workload = found;
            if (found?.assigned_agent_id) {
                try {
                    const node = await getNode(auth.token, found.assigned_agent_id);
                    nodeIp = node.ip_address;
                } catch {
                    nodeIp = null;
                }
            }
        } catch (e) {
            error = e instanceof Error ? e.message : "Failed to load vm";
        } finally {
            loading = false;
        }
    }

    function fmtBytes(bytes: number): string {
        if (bytes >= 1073741824) return `${(bytes / 1073741824).toFixed(1)} GB`;
        if (bytes >= 1048576) return `${(bytes / 1048576).toFixed(0)} MB`;
        return `${bytes} B`;
    }

    async function handleStop() {
        if (!auth.token || !workload) return;
        actionBusy = true;
        actionError = null;
        try {
            await stopWorkload(auth.token, workload.id);
            await load();
        } catch (e) {
            actionError = e instanceof Error ? e.message : "Failed to stop vm";
        } finally {
            actionBusy = false;
        }
    }

    async function handleRestart() {
        if (!auth.token || !workload) return;
        actionBusy = true;
        actionError = null;
        try {
            await restartWorkload(auth.token, workload.id);
            await load();
        } catch (e) {
            actionError = e instanceof Error ? e.message : "Failed to restart vm";
        } finally {
            actionBusy = false;
        }
    }

    async function handleDelete() {
        if (!auth.token || !workload) return;
        actionBusy = true;
        actionError = null;
        try {
            await deleteWorkload(auth.token, workload.id);
            goto(`/resource-groups/${rgId}`);
        } catch (e) {
            actionError = e instanceof Error ? e.message : "Failed to delete vm";
            actionBusy = false;
        }
    }

    let loadStarted = false;

    $effect(() => {
        if (auth.token && !loadStarted) {
            loadStarted = true;
            load();
        }
    });

    let pollInterval: ReturnType<typeof setInterval> | null = null;

    $effect(() => {
        if (workload && !pollInterval) {
            pollInterval = setInterval(load, 5000);
        }
    });

    onDestroy(() => {
        if (pollInterval) clearInterval(pollInterval);
    });
</script>

<header class="flex h-16 shrink-0 items-center gap-2 px-4 border-b">
    <button
        class="flex items-center justify-center w-8 h-8 rounded-full text-muted-foreground hover:text-foreground hover:bg-muted transition-colors"
        onclick={() => goto(`/resource-groups/${rgId}`)}
        aria-label="Back"
        title="Back"
    >
        <Icon icon="mdi:arrow-left" width={18} height={18} />
    </button>
    <span class="text-sm text-muted-foreground">/</span>
    <button
        class="text-sm text-muted-foreground hover:text-foreground transition-colors"
        onclick={() => goto("/resource-groups")}
    >
        Resource Groups
    </button>
    <span class="text-sm text-muted-foreground">/</span>
    <button
        class="text-sm text-muted-foreground hover:text-foreground transition-colors"
        onclick={() => goto(`/resource-groups/${rgId}`)}
    >
        {rgId.slice(0, 8)}
    </button>
    <span class="text-sm text-muted-foreground">/</span>
    <span class="text-sm font-medium">{workload?.service_name ?? workload?.name ?? workloadId.slice(0, 8)}</span>
</header>

<div class="flex flex-col gap-6 p-6">
    {#if loading}
        <p class="text-sm text-muted-foreground">Loading...</p>
    {:else if error}
        <p class="text-sm text-destructive">{error}</p>
    {:else if !workload}
        <p class="text-sm text-muted-foreground">VM not found.</p>
    {:else}
        <div class="flex items-start justify-between gap-4">
            <div class="flex items-center gap-3">
                <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-lg border bg-muted/40">
                    <Icon icon="mdi:monitor" width={24} height={24} />
                </div>
                <div>
                    <div class="flex items-center gap-3 flex-wrap">
                        <h1 class="text-xl font-semibold tracking-tight">{workload.service_name ?? workload.name}</h1>
                        <StatusBadge status={workload.status} />
                        {#if workload.restart_count > 0}
                            <span
                                class="text-xs px-1.5 py-0.5 rounded-full font-medium bg-amber-500/10 text-amber-600"
                                title="Restarted {workload.restart_count} time{workload.restart_count === 1 ? '' : 's'}"
                            >
                                ↻ {workload.restart_count}
                            </span>
                        {/if}
                    </div>
                    <p class="text-xs text-muted-foreground font-mono mt-0.5">
                        {workload.cpu_millicores}m vCPU · {fmtBytes(workload.memory_bytes)} RAM · {fmtBytes(workload.disk_bytes)} disk
                        {#if workload.assigned_agent_id}
                            · node {nodeIp ?? workload.assigned_agent_id.slice(0, 8)}
                        {/if}
                    </p>
                </div>
            </div>
            <div class="flex items-center gap-2 shrink-0">
                <Button size="sm" variant="outline" onclick={handleRestart} disabled={actionBusy}>
                    <Icon icon="mdi:restart" width={16} height={16} class="mr-1.5" />
                    Restart
                </Button>
                <Button
                    size="sm"
                    variant="outline"
                    onclick={handleStop}
                    disabled={actionBusy || workload.desired_state === "stopped"}
                >
                    <Icon icon="mdi:stop-circle-outline" width={16} height={16} class="mr-1.5" />
                    Stop
                </Button>
                <Button size="sm" variant="destructive" onclick={handleDelete} disabled={actionBusy}>
                    <Icon icon="mdi:trash-can-outline" width={16} height={16} class="mr-1.5" />
                    Delete
                </Button>
            </div>
        </div>

        {#if actionError}
            <p class="text-xs text-destructive">{actionError}</p>
        {/if}

        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
            <div class="border rounded-lg p-4">
                <p class="text-xs text-muted-foreground">CPU Usage</p>
                <p class="text-2xl font-semibold mt-1">
                    {workload.cpu_usage_percent !== null ? `${workload.cpu_usage_percent.toFixed(1)}%` : "-"}
                </p>
                <p class="text-xs text-muted-foreground mt-0.5">{workload.cpu_millicores}m requested</p>
            </div>
            <div class="border rounded-lg p-4">
                <p class="text-xs text-muted-foreground">Memory Usage</p>
                <p class="text-2xl font-semibold mt-1">
                    {workload.memory_usage_bytes !== null ? fmtBytes(workload.memory_usage_bytes) : "-"}
                </p>
                <p class="text-xs text-muted-foreground mt-0.5">{fmtBytes(workload.memory_bytes)} requested</p>
            </div>
            <div class="border rounded-lg p-4">
                <p class="text-xs text-muted-foreground">Network RX</p>
                <p class="text-2xl font-semibold mt-1">
                    {workload.network_rx_bytes !== null ? fmtBytes(workload.network_rx_bytes) : "-"}
                </p>
            </div>
            <div class="border rounded-lg p-4">
                <p class="text-xs text-muted-foreground">Network TX</p>
                <p class="text-2xl font-semibold mt-1">
                    {workload.network_tx_bytes !== null ? fmtBytes(workload.network_tx_bytes) : "-"}
                </p>
            </div>
        </div>
        <p class="text-xs text-muted-foreground -mt-3">
            {workload.stats_updated_at ? `Last updated ${new Date(workload.stats_updated_at).toLocaleTimeString()}` : "No stats reported yet."}
        </p>

        <div class="flex flex-col gap-2">
            <div class="flex items-center gap-2">
                <Icon icon="mdi:monitor-dashboard" width={16} height={16} class="text-muted-foreground" />
                <p class="text-sm font-medium">Console</p>
            </div>
            <div class="border rounded-lg overflow-hidden h-[36rem]">
                {#if workload.status !== "running" || !auth.token}
                    <div class="flex items-center justify-center h-full">
                        <p class="text-sm text-muted-foreground">Console is only available while the vm is running.</p>
                    </div>
                {:else}
                    {#key workload.id}
                        <VncConsole token={auth.token} workloadId={workload.id} />
                    {/key}
                {/if}
            </div>
        </div>
    {/if}
</div>
