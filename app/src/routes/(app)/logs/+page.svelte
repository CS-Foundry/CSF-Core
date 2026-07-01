<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { auth } from "$lib/auth/store.svelte";
    import { listLogs, type LogEntry, type LogsFilter } from "$lib/api/logs";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import { Input } from "$lib/components/ui/input/index.js";

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
    const RECENT_LIST_LIMIT = 200;

    let entries = $state<LogEntry[]>([]);
    let loading = $state(true);
    let error = $state<string | null>(null);
    let refreshTimer: ReturnType<typeof setInterval> | null = null;

    let selectedServices = $state<Set<string>>(new Set());
    let selectedLevel = $state("");
    let selectedClassification = $state("");
    let searchText = $state("");
    let fromTime = $state("");
    let toTime = $state("");
    let agentId = $state("");
    let workloadId = $state("");
    let organizationId = $state("");

    function currentFilter(): LogsFilter {
        return {
            level: selectedLevel || undefined,
            classification: selectedClassification || undefined,
            q: searchText || undefined,
            from: fromTime ? new Date(fromTime).toISOString() : undefined,
            to: toTime ? new Date(toTime).toISOString() : undefined,
            agent_id: agentId || undefined,
            workload_id: workloadId || undefined,
            organization_id: organizationId || undefined,
            limit: RECENT_LIST_LIMIT,
        };
    }

    async function load() {
        if (!auth.token) return;
        try {
            const response = await listLogs(auth.token, currentFilter());
            entries = response.entries;
            error = null;
        } catch (e) {
            error = e instanceof Error ? e.message : "Failed to load logs";
        } finally {
            loading = false;
        }
    }

    function toggleService(service: string) {
        const next = new Set(selectedServices);
        if (next.has(service)) {
            next.delete(service);
        } else {
            next.add(service);
        }
        selectedServices = next;
    }

    function visibleServices(): string[] {
        return selectedServices.size === 0 ? SERVICES : SERVICES.filter((s) => selectedServices.has(s));
    }

    function entriesForService(service: string): LogEntry[] {
        return entries.filter((entry) => entry.service === service);
    }

    function levelClass(level: string): string {
        switch (level) {
            case "ERROR": return "text-red-500";
            case "WARN": return "text-yellow-500";
            case "DEBUG": return "text-muted-foreground";
            default: return "text-foreground";
        }
    }

    function formatTime(timestamp: string): string {
        return timestamp.slice(11, 19);
    }

    onMount(() => {
        load();
        refreshTimer = setInterval(load, REFRESH_INTERVAL_MS);
    });

    onDestroy(() => {
        if (refreshTimer) clearInterval(refreshTimer);
    });

    $effect(() => {
        selectedLevel;
        selectedClassification;
        searchText;
        fromTime;
        toTime;
        agentId;
        workloadId;
        organizationId;
        selectedServices;
        load();
    });
</script>

<header class="flex h-16 shrink-0 items-center gap-2 px-4 border-b">
    <Sidebar.Trigger class="-ms-1" />
    <span class="text-sm text-muted-foreground">/</span>
    <span class="text-sm font-medium">Logs</span>
</header>

<div class="flex h-[calc(100vh-4rem)]">
    <aside class="w-80 shrink-0 border-r flex flex-col overflow-hidden">
        <div class="p-4 flex flex-col gap-3 border-b overflow-y-auto max-h-[50%]">
            <h2 class="text-xs font-semibold uppercase text-muted-foreground">Filters</h2>

            <div class="flex flex-col gap-1">
                <span class="text-xs text-muted-foreground">Services</span>
                <div class="flex flex-wrap gap-1">
                    {#each SERVICES as service (service)}
                        <button
                            class="text-xs px-2 py-1 rounded border {selectedServices.has(service)
                                ? 'bg-primary text-primary-foreground border-primary'
                                : 'bg-background text-muted-foreground'}"
                            onclick={() => toggleService(service)}
                        >
                            {service}
                        </button>
                    {/each}
                </div>
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

        <div class="flex-1 overflow-y-auto">
            <div class="px-4 py-2 border-b sticky top-0 bg-background">
                <span class="text-xs font-semibold uppercase text-muted-foreground">Recent</span>
            </div>
            {#if loading}
                <p class="p-4 text-xs text-muted-foreground">Loading...</p>
            {:else if entries.length === 0}
                <p class="p-4 text-xs text-muted-foreground">No log entries.</p>
            {:else}
                {#each entries as entry (entry.id)}
                    <div class="px-4 py-1.5 border-b text-xs flex flex-col gap-0.5 hover:bg-muted/30">
                        <div class="flex items-center gap-2 text-muted-foreground">
                            <span>{formatTime(entry.created_at)}</span>
                            <span class="font-medium text-foreground">{entry.service}</span>
                            <span class="{levelClass(entry.level)} font-medium">{entry.level}</span>
                        </div>
                        <span class="truncate">{entry.message}</span>
                    </div>
                {/each}
            {/if}
        </div>
    </aside>

    <main class="flex-1 overflow-hidden flex flex-col">
        {#if error}
            <p class="p-4 text-sm text-destructive">{error}</p>
        {/if}
        <div class="flex-1 flex overflow-x-auto">
            {#each visibleServices() as service (service)}
                {@const serviceEntries = entriesForService(service)}
                <div class="flex-1 min-w-64 border-r flex flex-col overflow-hidden">
                    <div class="px-3 py-2 border-b bg-muted/30 sticky top-0 flex items-center justify-between">
                        <span class="text-xs font-semibold">{service}</span>
                        <span class="text-xs text-muted-foreground">{serviceEntries.length}</span>
                    </div>
                    <div class="flex-1 overflow-y-auto font-mono">
                        {#each serviceEntries as entry (entry.id)}
                            <div class="px-3 py-0.5 border-b border-muted/50 text-[11px] leading-tight flex gap-1.5 hover:bg-muted/30">
                                <span class="text-muted-foreground shrink-0">{formatTime(entry.created_at)}</span>
                                <span class="{levelClass(entry.level)} shrink-0">{entry.level}</span>
                                <span class="truncate">{entry.message}</span>
                            </div>
                        {/each}
                    </div>
                </div>
            {/each}
        </div>
    </main>
</div>
