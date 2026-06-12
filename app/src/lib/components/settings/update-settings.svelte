<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import * as Card from "$lib/components/ui/card/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import { auth } from "$lib/auth/store.svelte";
    import {
        getUpdateStatus,
        getReleases,
        triggerUpdate,
        pauseUpdate,
        resumeUpdate,
        type UpdateStatus,
        type ReleasesResponse,
    } from "$lib/api/system";

    let status = $state<UpdateStatus | null>(null);
    let releases = $state<ReleasesResponse | null>(null);
    let statusLoading = $state(true);
    let releasesLoading = $state(true);
    let error = $state<string | null>(null);
    let actionLoading = $state(false);
    let includePre = $state(false);
    let pollInterval: ReturnType<typeof setInterval> | null = null;

    async function fetchStatus() {
        if (!auth.token) return;
        try {
            status = await getUpdateStatus(auth.token);
            error = null;
        } catch (e) {
            error = e instanceof Error ? e.message : "failed to fetch update status";
        } finally {
            statusLoading = false;
        }
    }

    async function fetchReleases() {
        if (!auth.token) return;
        releasesLoading = true;
        try {
            releases = await getReleases(auth.token, includePre);
        } catch (e) {
            error = e instanceof Error ? e.message : "failed to fetch releases";
        } finally {
            releasesLoading = false;
        }
    }

    onMount(() => {
        fetchStatus();
        fetchReleases();
        pollInterval = setInterval(fetchStatus, 5000);
    });

    onDestroy(() => {
        if (pollInterval) clearInterval(pollInterval);
    });

    async function handleTrigger(version: string) {
        if (!auth.token) return;
        actionLoading = true;
        try {
            await triggerUpdate(auth.token, version);
            await fetchStatus();
        } catch (e) {
            error = e instanceof Error ? e.message : "trigger update failed";
        } finally {
            actionLoading = false;
        }
    }

    async function handlePause() {
        if (!auth.token) return;
        actionLoading = true;
        try {
            await pauseUpdate(auth.token);
            await fetchStatus();
        } catch (e) {
            error = e instanceof Error ? e.message : "pause failed";
        } finally {
            actionLoading = false;
        }
    }

    async function handleResume() {
        if (!auth.token) return;
        actionLoading = true;
        try {
            await resumeUpdate(auth.token);
            await fetchStatus();
        } catch (e) {
            error = e instanceof Error ? e.message : "resume failed";
        } finally {
            actionLoading = false;
        }
    }

    function buildStatusLabel(s: string | null): string {
        if (!s) return "idle";
        return s.replace(/_/g, " ").toLowerCase();
    }

    function buildStatusColor(s: string | null): string {
        if (!s) return "text-muted-foreground";
        const lower = s.toLowerCase();
        if (lower.includes("success") || lower.includes("done")) return "text-green-500";
        if (lower.includes("fail") || lower.includes("error")) return "text-red-500";
        if (lower.includes("running") || lower.includes("pending")) return "text-yellow-500";
        return "text-muted-foreground";
    }

    function isActive(s: string | null): boolean {
        if (!s) return false;
        const lower = s.toLowerCase();
        return lower.includes("running") || lower.includes("pending") || lower.includes("building");
    }
</script>

{#if error}
    <div class="mb-4 px-3 py-2 rounded-md bg-destructive/10 border border-destructive/20 text-sm text-destructive">
        {error}
    </div>
{/if}

<div class="flex flex-col gap-6 max-w-2xl">
    <Card.Root>
        <Card.Header>
            <div class="flex items-start justify-between gap-4">
                <div>
                    <Card.Title>System version</Card.Title>
                    <Card.Description>Current installation and update state</Card.Description>
                </div>
                {#if status}
                    <span class="text-xs font-medium px-2 py-1 rounded-full border {status.paused ? 'border-yellow-500/30 text-yellow-500 bg-yellow-500/10' : 'border-green-500/30 text-green-500 bg-green-500/10'}">
                        {status.paused ? "paused" : "active"}
                    </span>
                {/if}
            </div>
        </Card.Header>
        <Card.Content>
            {#if statusLoading}
                <div class="flex flex-col gap-3">
                    {#each [1, 2] as _}
                        <div class="h-4 rounded bg-muted animate-pulse"></div>
                    {/each}
                </div>
            {:else if status}
                <div class="grid grid-cols-2 gap-px bg-border rounded-lg overflow-hidden border">
                    <div class="bg-background p-4">
                        <p class="text-xs text-muted-foreground mb-1">Installed</p>
                        <p class="font-mono text-sm font-medium">{status.current_version}</p>
                    </div>

                    {#if status.desired_version && status.desired_version !== status.current_version}
                        <div class="bg-background p-4">
                            <p class="text-xs text-muted-foreground mb-1">Scheduled</p>
                            <p class="font-mono text-sm font-medium text-yellow-500">{status.desired_version}</p>
                        </div>
                    {:else}
                        <div class="bg-background p-4">
                            <p class="text-xs text-muted-foreground mb-1">Target</p>
                            <p class="text-sm text-muted-foreground">up to date</p>
                        </div>
                    {/if}

                    {#if status.build_status}
                        <div class="bg-background p-4 col-span-2">
                            <div class="flex items-center justify-between">
                                <p class="text-xs text-muted-foreground">Build status</p>
                                {#if isActive(status.build_status)}
                                    <span class="flex items-center gap-1.5 text-xs text-yellow-500">
                                        <span class="inline-block w-1.5 h-1.5 rounded-full bg-yellow-500 animate-pulse"></span>
                                        running
                                    </span>
                                {/if}
                            </div>
                            <p class="font-mono text-sm mt-1 {buildStatusColor(status.build_status)}">
                                {buildStatusLabel(status.build_status)}
                            </p>
                        </div>
                    {/if}

                    {#if status.last_result}
                        <div class="bg-background p-4 {status.build_status ? '' : 'col-span-2'}">
                            <p class="text-xs text-muted-foreground mb-1">Last result</p>
                            <p class="font-mono text-sm {buildStatusColor(status.last_result)}">
                                {buildStatusLabel(status.last_result)}
                            </p>
                        </div>
                    {/if}
                </div>
            {/if}
        </Card.Content>
        {#if status}
            <Card.Footer class="border-t pt-4 flex items-center justify-between">
                <p class="text-xs text-muted-foreground">
                    {status.paused ? "Updates are paused. No automatic upgrades will run." : "Updates are enabled. The system checks for new releases automatically."}
                </p>
                {#if !status.paused}
                    <Button onclick={handlePause} disabled={actionLoading} variant="outline" size="sm">
                        Pause updates
                    </Button>
                {:else}
                    <Button onclick={handleResume} disabled={actionLoading} size="sm">
                        Resume updates
                    </Button>
                {/if}
            </Card.Footer>
        {/if}
    </Card.Root>

    <Card.Root>
        <Card.Header>
            <div class="flex items-center justify-between">
                <div>
                    <Card.Title>Available releases</Card.Title>
                    <Card.Description>Select a version to update or downgrade</Card.Description>
                </div>
                <label class="flex items-center gap-2 text-xs text-muted-foreground cursor-pointer select-none">
                    <div class="relative inline-flex">
                        <input
                            type="checkbox"
                            bind:checked={includePre}
                            onchange={fetchReleases}
                            class="peer sr-only"
                            id="include-pre"
                        />
                        <div class="w-8 h-4 rounded-full bg-muted border peer-checked:bg-primary transition-colors"></div>
                        <div class="absolute left-0.5 top-0.5 w-3 h-3 rounded-full bg-white shadow-sm transition-transform peer-checked:translate-x-4"></div>
                    </div>
                    Pre-releases
                </label>
            </div>
        </Card.Header>
        <Card.Content class="p-0">
            {#if releasesLoading}
                <div class="flex flex-col gap-0">
                    {#each [1, 2, 3] as _}
                        <div class="px-6 py-4 border-b last:border-b-0 flex items-center justify-between">
                            <div class="flex flex-col gap-1.5">
                                <div class="h-4 w-24 rounded bg-muted animate-pulse"></div>
                                <div class="h-3 w-48 rounded bg-muted animate-pulse"></div>
                            </div>
                            <div class="h-7 w-20 rounded bg-muted animate-pulse"></div>
                        </div>
                    {/each}
                </div>
            {:else if releases && releases.releases.length > 0}
                <div class="flex flex-col">
                    {#each releases.releases as release, i (release.tag)}
                        <div class="flex items-center justify-between px-6 py-4 gap-6 {i < releases.releases.length - 1 ? 'border-b' : ''} {release.is_current ? 'bg-muted/30' : 'hover:bg-muted/20'} transition-colors">
                            <div class="flex flex-col gap-1 min-w-0">
                                <div class="flex items-center gap-2 flex-wrap">
                                    <span class="font-mono text-sm font-medium {release.is_newer ? 'text-foreground' : release.is_current ? 'text-foreground' : 'text-muted-foreground'}">
                                        {release.version}
                                    </span>
                                    {#if release.is_current}
                                        <span class="inline-flex items-center text-[10px] font-medium px-1.5 py-0.5 rounded-full bg-green-500/15 text-green-500 border border-green-500/20">
                                            installed
                                        </span>
                                    {:else if release.is_newer}
                                        <span class="inline-flex items-center text-[10px] font-medium px-1.5 py-0.5 rounded-full bg-yellow-500/15 text-yellow-500 border border-yellow-500/20">
                                            newer
                                        </span>
                                    {/if}
                                    {#if release.prerelease}
                                        <span class="inline-flex items-center text-[10px] px-1.5 py-0.5 rounded-full bg-muted text-muted-foreground border border-border">
                                            pre-release
                                        </span>
                                    {/if}
                                </div>
                                {#if release.name}
                                    <span class="text-xs text-muted-foreground truncate">{release.name}</span>
                                {/if}
                            </div>
                            {#if !release.is_current}
                                <Button
                                    onclick={() => handleTrigger(release.version)}
                                    disabled={actionLoading}
                                    variant={release.is_newer ? "default" : "outline"}
                                    size="sm"
                                    class="shrink-0"
                                >
                                    {release.is_newer ? "Update" : "Downgrade"}
                                </Button>
                            {:else}
                                <span class="text-xs text-muted-foreground shrink-0">current</span>
                            {/if}
                        </div>
                    {/each}
                </div>
            {:else}
                <div class="px-6 py-12 text-center">
                    <p class="text-sm text-muted-foreground">No releases found</p>
                </div>
            {/if}
        </Card.Content>
    </Card.Root>
</div>
