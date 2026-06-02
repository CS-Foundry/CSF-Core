<script lang="ts">
    import { auth } from '$lib/auth/store.svelte';
    import { getNodeMetricsLatest, type Node, type NodeMetricsLatest } from '$lib/api/nodes';
    import * as Sheet from '$lib/components/ui/sheet/index.js';
    import { Button } from '$lib/components/ui/button/index.js';

    interface Props {
        node: Node | null;
        open: boolean;
        onClose: () => void;
    }

    let { node, open, onClose }: Props = $props();

    let metrics = $state<NodeMetricsLatest | null>(null);
    let metricsError = $state<string | null>(null);
    let metricsLoading = $state(false);
    let activeTab = $state<'overview' | 'actions'>('overview');
    let sshCopied = $state(false);

    $effect(() => {
        if (node && open) {
            activeTab = 'overview';
            metrics = null;
            metricsError = null;
            loadMetrics(node.id);
        }
    });

    async function loadMetrics(id: string) {
        if (!auth.token) return;
        metricsLoading = true;
        try {
            metrics = await getNodeMetricsLatest(auth.token, id);
        } catch {
            metricsError = 'No metrics available';
        } finally {
            metricsLoading = false;
        }
    }

    function bytesToGb(bytes: number | null): string {
        if (bytes == null) return '-';
        return (bytes / 1_073_741_824).toFixed(1) + ' GB';
    }

    function bytesToMb(bytes: number | null): string {
        if (bytes == null) return '-';
        return (bytes / 1_048_576).toFixed(0) + ' MB/s';
    }

    function formatUptime(seconds: number | null): string {
        if (seconds == null) return '-';
        const d = Math.floor(seconds / 86400);
        const h = Math.floor((seconds % 86400) / 3600);
        const m = Math.floor((seconds % 3600) / 60);
        if (d > 0) return `${d}d ${h}h ${m}m`;
        if (h > 0) return `${h}h ${m}m`;
        return `${m}m`;
    }

    function pct(value: number | null): string {
        if (value == null) return '-';
        return value.toFixed(1) + '%';
    }

    function statusClass(status: string): string {
        switch (status.toLowerCase()) {
            case 'online': return 'text-green-500';
            case 'offline': return 'text-red-500';
            case 'degraded': return 'text-yellow-500';
            default: return 'text-muted-foreground';
        }
    }

    async function copySshCommand() {
        if (!node?.ip_address) return;
        await navigator.clipboard.writeText(`ssh root@${node.ip_address}`);
        sshCopied = true;
        setTimeout(() => { sshCopied = false; }, 2000);
    }

    function openSsh() {
        if (!node?.ip_address) return;
        window.location.href = `ssh://root@${node.ip_address}`;
    }
</script>

<Sheet.Root
    open={open}
    onOpenChange={(v) => { if (!v) onClose(); }}
>
    <Sheet.Content side="right" class="w-[480px] sm:max-w-[480px] flex flex-col p-0 gap-0">
        {#if node}
            <Sheet.Header class="px-6 pt-6 pb-4 border-b shrink-0">
                <div class="flex items-center gap-3 pr-8">
                    <div class="flex flex-col gap-0.5">
                        <Sheet.Title class="text-base font-semibold leading-tight">{node.hostname}</Sheet.Title>
                        <span class="text-xs text-muted-foreground font-mono">{node.ip_address ?? 'no ip'}</span>
                    </div>
                    <span class="ml-auto text-sm font-medium {statusClass(node.status)}">{node.status}</span>
                </div>

                <div class="flex gap-1 mt-3">
                    <button
                        class="px-3 py-1.5 text-xs rounded-md transition-colors {activeTab === 'overview' ? 'bg-muted font-medium' : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'}"
                        onclick={() => activeTab = 'overview'}
                    >
                        Overview
                    </button>
                    <button
                        class="px-3 py-1.5 text-xs rounded-md transition-colors {activeTab === 'actions' ? 'bg-muted font-medium' : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'}"
                        onclick={() => activeTab = 'actions'}
                    >
                        Actions
                    </button>
                </div>
            </Sheet.Header>

            <div class="flex-1 overflow-y-auto px-6 py-4">
                {#if activeTab === 'overview'}
                    <div class="flex flex-col gap-5">
                        <section>
                            <p class="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-3">Node Info</p>
                            <div class="grid grid-cols-2 gap-y-2 text-sm">
                                <span class="text-muted-foreground">ID</span>
                                <span class="font-mono text-xs truncate">{node.id}</span>
                                <span class="text-muted-foreground">OS</span>
                                <span>{node.os_type} {node.os_version}</span>
                                <span class="text-muted-foreground">Arch</span>
                                <span>{node.architecture}</span>
                                <span class="text-muted-foreground">Agent</span>
                                <span>{node.agent_version}</span>
                                <span class="text-muted-foreground">Registered</span>
                                <span class="text-xs">{node.registered_at.slice(0, 16)}</span>
                                <span class="text-muted-foreground">Last heartbeat</span>
                                <span class="text-xs">{node.last_heartbeat ? node.last_heartbeat.slice(0, 16) : 'never'}</span>
                            </div>
                        </section>

                        <div class="border-t"></div>

                        <section>
                            <p class="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-3">Live Metrics</p>

                            {#if metricsLoading}
                                <p class="text-sm text-muted-foreground">Loading...</p>
                            {:else if metricsError}
                                <p class="text-sm text-muted-foreground">{metricsError}</p>
                            {:else if metrics}
                                <div class="flex flex-col gap-4">
                                    {#if metrics.uptime_seconds != null}
                                        <div class="flex flex-col gap-1">
                                            <div class="flex justify-between text-xs">
                                                <span class="text-muted-foreground">Uptime</span>
                                                <span>{formatUptime(metrics.uptime_seconds)}</span>
                                            </div>
                                        </div>
                                    {/if}

                                    <div class="flex flex-col gap-1">
                                        <div class="flex justify-between text-xs">
                                            <span class="text-muted-foreground">CPU{metrics.cpu_model ? ` — ${metrics.cpu_model}` : ''}</span>
                                            <span>{pct(metrics.cpu_usage_percent)}</span>
                                        </div>
                                        {#if metrics.cpu_usage_percent != null}
                                            <div class="h-1.5 bg-muted rounded-full overflow-hidden">
                                                <div
                                                    class="h-full rounded-full transition-all {metrics.cpu_usage_percent > 80 ? 'bg-red-500' : metrics.cpu_usage_percent > 60 ? 'bg-yellow-500' : 'bg-primary'}"
                                                    style="width: {Math.min(metrics.cpu_usage_percent, 100)}%"
                                                ></div>
                                            </div>
                                        {/if}
                                        {#if metrics.cpu_cores != null}
                                            <span class="text-xs text-muted-foreground">{metrics.cpu_cores} cores / {metrics.cpu_threads ?? '?'} threads</span>
                                        {/if}
                                    </div>

                                    <div class="flex flex-col gap-1">
                                        <div class="flex justify-between text-xs">
                                            <span class="text-muted-foreground">Memory</span>
                                            <span>{bytesToGb(metrics.memory_used_bytes)} / {bytesToGb(metrics.memory_total_bytes)}</span>
                                        </div>
                                        {#if metrics.memory_usage_percent != null}
                                            <div class="h-1.5 bg-muted rounded-full overflow-hidden">
                                                <div
                                                    class="h-full rounded-full transition-all {metrics.memory_usage_percent > 80 ? 'bg-red-500' : metrics.memory_usage_percent > 60 ? 'bg-yellow-500' : 'bg-primary'}"
                                                    style="width: {Math.min(metrics.memory_usage_percent, 100)}%"
                                                ></div>
                                            </div>
                                        {/if}
                                    </div>

                                    <div class="flex flex-col gap-1">
                                        <div class="flex justify-between text-xs">
                                            <span class="text-muted-foreground">Disk</span>
                                            <span>{bytesToGb(metrics.disk_used_bytes)} / {bytesToGb(metrics.disk_total_bytes)}</span>
                                        </div>
                                        {#if metrics.disk_usage_percent != null}
                                            <div class="h-1.5 bg-muted rounded-full overflow-hidden">
                                                <div
                                                    class="h-full rounded-full transition-all {metrics.disk_usage_percent > 80 ? 'bg-red-500' : metrics.disk_usage_percent > 60 ? 'bg-yellow-500' : 'bg-primary'}"
                                                    style="width: {Math.min(metrics.disk_usage_percent, 100)}%"
                                                ></div>
                                            </div>
                                        {/if}
                                    </div>

                                    {#if metrics.network_rx_bytes != null || metrics.network_tx_bytes != null}
                                        <div class="flex flex-col gap-1">
                                            <span class="text-xs text-muted-foreground">Network</span>
                                            <div class="flex gap-4 text-xs">
                                                <span>RX {bytesToMb(metrics.network_rx_bytes)}</span>
                                                <span>TX {bytesToMb(metrics.network_tx_bytes)}</span>
                                            </div>
                                        </div>
                                    {/if}

                                    {#if metrics.kernel_version}
                                        <div class="flex justify-between text-xs">
                                            <span class="text-muted-foreground">Kernel</span>
                                            <span class="font-mono text-xs">{metrics.kernel_version}</span>
                                        </div>
                                    {/if}
                                </div>
                            {/if}
                        </section>
                    </div>

                {:else if activeTab === 'actions'}
                    <div class="flex flex-col gap-6">
                        <section>
                            <p class="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-3">SSH Access</p>
                            <div class="flex flex-col gap-2">
                                {#if node.ip_address}
                                    <div class="flex items-center gap-2 px-3 py-2 bg-muted rounded-md font-mono text-xs">
                                        <span class="flex-1 truncate">ssh root@{node.ip_address}</span>
                                    </div>
                                    <div class="flex gap-2">
                                        <Button variant="outline" size="sm" class="flex-1" onclick={copySshCommand}>
                                            {sshCopied ? 'Copied' : 'Copy command'}
                                        </Button>
                                        <Button variant="outline" size="sm" class="flex-1" onclick={openSsh}>
                                            Open terminal
                                        </Button>
                                    </div>
                                {:else}
                                    <p class="text-sm text-muted-foreground">No IP address available</p>
                                {/if}
                            </div>
                        </section>

                        <div class="border-t"></div>

                        <section>
                            <p class="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-3">Power Management</p>
                            <div class="flex flex-col gap-2">
                                <Button variant="outline" size="sm" class="justify-start" disabled>
                                    Reboot
                                </Button>
                                <Button variant="outline" size="sm" class="justify-start text-red-500 hover:text-red-500" disabled>
                                    Power off
                                </Button>
                                <Button variant="outline" size="sm" class="justify-start text-green-600 hover:text-green-600" disabled>
                                    Power on (WoL)
                                </Button>
                                <p class="text-xs text-muted-foreground mt-1">Power management requires agent support</p>
                            </div>
                        </section>
                    </div>
                {/if}
            </div>
        {/if}
    </Sheet.Content>
</Sheet.Root>
