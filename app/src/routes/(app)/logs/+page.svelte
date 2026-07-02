<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { auth } from "$lib/auth/store.svelte";
    import { listLogs, type LogEntry, type LogsFilter } from "$lib/api/logs";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import * as Sheet from "$lib/components/ui/sheet/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import RefreshCwIcon from "@lucide/svelte/icons/refresh-cw";

    const SERVICES = [
        "api-gateway",
        "registry",
        "volume-manager",
        "scheduler",
        "failover-controller",
        "sdn-controller",
    ];
    const LEVELS = ["DEBUG", "INFO", "WARN", "ERROR"];
    const CLASSIFICATIONS = ["security", "performance", "audit", "system", "network", "storage"];
    const REFRESH_INTERVAL_MS = 7000;
    const PAGE_LIMIT = 100;

    let entries = $state<LogEntry[]>([]);
    let total = $state(0);
    let hasMore = $state(false);
    let offset = $state(0);
    let loading = $state(true);
    let refreshing = $state(false);
    let error = $state<string | null>(null);
    let refreshTimer: ReturnType<typeof setInterval> | null = null;

    let selectedService = $state("");
    let selectedLevel = $state("");
    let selectedClassification = $state("");
    let searchText = $state("");
    let fromTime = $state("");
    let toTime = $state("");
    let agentId = $state("");
    let workloadId = $state("");
    let organizationId = $state("");

    let selectedEntry = $state<LogEntry | null>(null);
    let detailOpen = $state(false);

    function currentFilter(): LogsFilter {
        return {
            service: selectedService || undefined,
            level: selectedLevel || undefined,
            classification: selectedClassification || undefined,
            q: searchText || undefined,
            from: fromTime ? new Date(fromTime).toISOString() : undefined,
            to: toTime ? new Date(toTime).toISOString() : undefined,
            agent_id: agentId || undefined,
            workload_id: workloadId || undefined,
            organization_id: organizationId || undefined,
            limit: PAGE_LIMIT,
            offset,
        };
    }

    async function load() {
        if (!auth.token) return;
        try {
            const response = await listLogs(auth.token, currentFilter());
            entries = response.entries;
            total = response.total;
            hasMore = response.has_more;
            error = null;
        } catch (e) {
            error = e instanceof Error ? e.message : "Failed to load logs";
        } finally {
            loading = false;
            refreshing = false;
        }
    }

    function refreshNow() {
        refreshing = true;
        load();
    }

    function levelClass(level: string): string {
        switch (level) {
            case "ERROR": return "text-red-500";
            case "WARN": return "text-yellow-500";
            case "DEBUG": return "text-muted-foreground";
            default: return "text-foreground";
        }
    }

    function severityBars(level: string): number {
        switch (level) {
            case "ERROR": return 3;
            case "WARN": return 2;
            default: return 1;
        }
    }

    function severityBarClass(level: string): string {
        switch (level) {
            case "ERROR": return "bg-red-500";
            case "WARN": return "bg-yellow-500";
            default: return "bg-green-500";
        }
    }

    function formatDateTime(timestamp: string): string {
        return new Date(timestamp).toLocaleString();
    }

    function openDetail(entry: LogEntry) {
        selectedEntry = entry;
        detailOpen = true;
    }

    function nextPage() {
        if (!hasMore) return;
        offset += PAGE_LIMIT;
    }

    function prevPage() {
        offset = Math.max(0, offset - PAGE_LIMIT);
    }

    onMount(() => {
        load();
        refreshTimer = setInterval(load, REFRESH_INTERVAL_MS);
    });

    onDestroy(() => {
        if (refreshTimer) clearInterval(refreshTimer);
    });

    $effect(() => {
        selectedService;
        selectedLevel;
        selectedClassification;
        searchText;
        fromTime;
        toTime;
        agentId;
        workloadId;
        organizationId;
        offset;
        load();
    });

    $effect(() => {
        selectedService;
        selectedLevel;
        selectedClassification;
        searchText;
        fromTime;
        toTime;
        agentId;
        workloadId;
        organizationId;
        offset = 0;
    });
</script>

<header class="flex h-16 shrink-0 items-center gap-2 px-4 border-b">
    <Sidebar.Trigger class="-ms-1" />
    <span class="text-sm text-muted-foreground">/</span>
    <span class="text-sm font-medium">Logs</span>
    <Button
        variant="outline"
        size="sm"
        class="ms-auto gap-1.5"
        onclick={refreshNow}
        disabled={refreshing}
    >
        <RefreshCwIcon class="size-3.5 {refreshing ? 'animate-spin' : ''}" />
        Refresh
    </Button>
</header>

<div class="flex h-[calc(100vh-4rem)]">
    <aside class="w-72 shrink-0 border-r flex flex-col overflow-y-auto">
        <div class="p-4 flex flex-col gap-3">
            <h2 class="text-xs font-semibold uppercase text-muted-foreground">Filters</h2>

            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="service-select">Service</label>
                <select id="service-select" class="border rounded px-2 py-1.5 text-sm bg-background" bind:value={selectedService}>
                    <option value="">All services</option>
                    {#each SERVICES as service (service)}
                        <option value={service}>{service}</option>
                    {/each}
                </select>
            </div>

            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="level-select">Level</label>
                <select id="level-select" class="border rounded px-2 py-1.5 text-sm bg-background" bind:value={selectedLevel}>
                    <option value="">All levels</option>
                    {#each LEVELS as level (level)}
                        <option value={level}>{level}</option>
                    {/each}
                </select>
            </div>

            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="classification-select">Classification</label>
                <select id="classification-select" class="border rounded px-2 py-1.5 text-sm bg-background" bind:value={selectedClassification}>
                    <option value="">All classifications</option>
                    {#each CLASSIFICATIONS as classification (classification)}
                        <option value={classification}>{classification}</option>
                    {/each}
                </select>
            </div>

            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="search-input">Search</label>
                <Input id="search-input" placeholder="message contains..." bind:value={searchText} />
            </div>

            <div class="grid grid-cols-2 gap-2">
                <div class="flex flex-col gap-1">
                    <label class="text-xs text-muted-foreground" for="from-input">From</label>
                    <input id="from-input" type="datetime-local" class="border rounded px-2 py-1.5 text-xs bg-background" bind:value={fromTime} />
                </div>
                <div class="flex flex-col gap-1">
                    <label class="text-xs text-muted-foreground" for="to-input">To</label>
                    <input id="to-input" type="datetime-local" class="border rounded px-2 py-1.5 text-xs bg-background" bind:value={toTime} />
                </div>
            </div>

            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="agent-input">Agent ID</label>
                <Input id="agent-input" placeholder="uuid" bind:value={agentId} class="font-mono text-xs" />
            </div>
            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="workload-input">Workload ID</label>
                <Input id="workload-input" placeholder="uuid" bind:value={workloadId} class="font-mono text-xs" />
            </div>
            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="org-input">Organization ID</label>
                <Input id="org-input" placeholder="uuid" bind:value={organizationId} class="font-mono text-xs" />
            </div>
        </div>
    </aside>

    <main class="flex-1 overflow-hidden flex flex-col">
        {#if error}
            <p class="p-4 text-sm text-destructive">{error}</p>
        {/if}

        <div class="flex-1 overflow-y-auto">
            <table class="w-full text-xs border-collapse">
                <thead class="sticky top-0 bg-background border-b z-10">
                    <tr class="text-left text-[11px] text-muted-foreground uppercase">
                        <th class="px-2 py-1.5 font-semibold">Severity</th>
                        <th class="px-2 py-1.5 font-semibold">Service</th>
                        <th class="px-2 py-1.5 font-semibold">Level</th>
                        <th class="px-2 py-1.5 font-semibold">Classification</th>
                        <th class="px-2 py-1.5 font-semibold">Message</th>
                        <th class="px-2 py-1.5 font-semibold whitespace-nowrap">Date / Time</th>
                    </tr>
                </thead>
                <tbody>
                    {#if loading}
                        <tr><td class="px-2 py-3 text-muted-foreground" colspan="6">Loading...</td></tr>
                    {:else if entries.length === 0}
                        <tr><td class="px-2 py-3 text-muted-foreground" colspan="6">No log entries.</td></tr>
                    {:else}
                        {#each entries as entry (entry.id)}
                            <tr
                                class="border-b hover:bg-muted/30 cursor-pointer"
                                onclick={() => openDetail(entry)}
                            >
                                <td class="px-2 py-1">
                                    <div class="flex gap-0.5" title={entry.level}>
                                        {#each [1, 2, 3] as bar (bar)}
                                            <span
                                                class="h-2.5 w-1.5 rounded-sm {bar <= severityBars(entry.level) ? severityBarClass(entry.level) : 'bg-muted'}"
                                            ></span>
                                        {/each}
                                    </div>
                                </td>
                                <td class="px-2 py-1 font-medium whitespace-nowrap">{entry.service}</td>
                                <td class="px-2 py-1 {levelClass(entry.level)} font-medium whitespace-nowrap">{entry.level}</td>
                                <td class="px-2 py-1 whitespace-nowrap">{entry.classification}</td>
                                <td class="px-2 py-1 truncate max-w-0">{entry.message}</td>
                                <td class="px-2 py-1 whitespace-nowrap text-muted-foreground">{formatDateTime(entry.created_at)}</td>
                            </tr>
                        {/each}
                    {/if}
                </tbody>
            </table>
        </div>

        <div class="flex items-center justify-between border-t px-4 py-2 text-xs text-muted-foreground">
            <span>{entries.length === 0 ? 0 : offset + 1}-{offset + entries.length} of {total} logs</span>
            <div class="flex gap-2">
                <button
                    class="px-2 py-1 rounded border disabled:opacity-40"
                    onclick={prevPage}
                    disabled={offset === 0}
                >
                    Prev
                </button>
                <button
                    class="px-2 py-1 rounded border disabled:opacity-40"
                    onclick={nextPage}
                    disabled={!hasMore}
                >
                    Next
                </button>
            </div>
        </div>
    </main>
</div>

<Sheet.Root bind:open={detailOpen}>
    <Sheet.Content side="right" class="w-full sm:max-w-md overflow-y-auto">
        {#if selectedEntry}
            <Sheet.Header>
                <Sheet.Title>{formatDateTime(selectedEntry.created_at)}</Sheet.Title>
            </Sheet.Header>
            <div class="flex flex-col gap-3 px-4 pb-4 text-sm">
                <div class="flex justify-between border-b pb-2">
                    <span class="text-muted-foreground">Service</span>
                    <span class="font-medium">{selectedEntry.service}</span>
                </div>
                <div class="flex justify-between border-b pb-2">
                    <span class="text-muted-foreground">Level</span>
                    <span class="font-medium {levelClass(selectedEntry.level)}">{selectedEntry.level}</span>
                </div>
                <div class="flex justify-between border-b pb-2">
                    <span class="text-muted-foreground">Classification</span>
                    <span class="font-medium">{selectedEntry.classification}</span>
                </div>
                {#if selectedEntry.agent_id}
                    <div class="flex justify-between border-b pb-2">
                        <span class="text-muted-foreground">Agent ID</span>
                        <span class="font-mono text-xs">{selectedEntry.agent_id}</span>
                    </div>
                {/if}
                {#if selectedEntry.workload_id}
                    <div class="flex justify-between border-b pb-2">
                        <span class="text-muted-foreground">Workload ID</span>
                        <span class="font-mono text-xs">{selectedEntry.workload_id}</span>
                    </div>
                {/if}
                {#if selectedEntry.organization_id}
                    <div class="flex justify-between border-b pb-2">
                        <span class="text-muted-foreground">Organization ID</span>
                        <span class="font-mono text-xs">{selectedEntry.organization_id}</span>
                    </div>
                {/if}
                <div class="flex flex-col gap-1 pt-2">
                    <span class="text-xs font-semibold uppercase text-muted-foreground">Message</span>
                    <p class="whitespace-pre-wrap break-words">{selectedEntry.message}</p>
                </div>
            </div>
        {/if}
    </Sheet.Content>
</Sheet.Root>
