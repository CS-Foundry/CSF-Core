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
</script>

{#if error}
    <p class="text-sm text-destructive mb-4">{error}</p>
{/if}

<div class="flex flex-col gap-6 max-w-xl">
    <Card.Root size="sm">
        <Card.Header>
            <Card.Title>Current version</Card.Title>
        </Card.Header>
        <Card.Content class="flex flex-col gap-2">
            {#if statusLoading}
                <p class="text-sm text-muted-foreground">Loading...</p>
            {:else if status}
                <div class="flex justify-between text-sm">
                    <span class="text-muted-foreground">Installed</span>
                    <span class="font-mono">{status.current_version}</span>
                </div>
                {#if status.desired_version && status.desired_version !== status.current_version}
                    <div class="flex justify-between text-sm">
                        <span class="text-muted-foreground">Scheduled</span>
                        <span class="font-mono text-yellow-500">{status.desired_version}</span>
                    </div>
                {/if}
                {#if status.build_status}
                    <div class="flex justify-between text-sm">
                        <span class="text-muted-foreground">Build status</span>
                        <span class="font-mono">{buildStatusLabel(status.build_status)}</span>
                    </div>
                {/if}
                {#if status.last_result}
                    <div class="flex justify-between text-sm">
                        <span class="text-muted-foreground">Last result</span>
                        <span class="font-mono">{buildStatusLabel(status.last_result)}</span>
                    </div>
                {/if}
                {#if status.paused}
                    <div class="flex justify-between text-sm">
                        <span class="text-muted-foreground">Updates</span>
                        <span class="text-yellow-500">paused</span>
                    </div>
                {/if}
            {/if}
        </Card.Content>
        {#if status}
            <Card.Footer class="flex gap-2">
                {#if !status.paused}
                    <Button onclick={handlePause} disabled={actionLoading} variant="outline" size="sm">
                        Pause updates
                    </Button>
                {:else}
                    <Button onclick={handleResume} disabled={actionLoading} variant="outline" size="sm">
                        Resume updates
                    </Button>
                {/if}
            </Card.Footer>
        {/if}
    </Card.Root>

    <Card.Root size="sm">
        <Card.Header>
            <Card.Title>Available releases</Card.Title>
            <Card.Action>
                <label class="flex items-center gap-2 text-xs text-muted-foreground cursor-pointer select-none">
                    <input
                        type="checkbox"
                        bind:checked={includePre}
                        onchange={fetchReleases}
                        class="size-3"
                    />
                    Pre-releases
                </label>
            </Card.Action>
        </Card.Header>
        <Card.Content>
            {#if releasesLoading}
                <p class="text-sm text-muted-foreground">Loading...</p>
            {:else if releases && releases.releases.length > 0}
                <div class="flex flex-col divide-y divide-border">
                    {#each releases.releases as release (release.tag)}
                        <div class="flex items-center justify-between py-2 gap-4">
                            <div class="flex flex-col gap-0.5 min-w-0">
                                <div class="flex items-center gap-2">
                                    <span class="font-mono text-sm {release.is_newer ? 'text-yellow-500' : release.is_current ? 'text-green-500' : 'text-muted-foreground'}">
                                        {release.version}
                                    </span>
                                    {#if release.is_current}
                                        <span class="text-xs text-green-500 font-medium">current</span>
                                    {:else if release.is_newer}
                                        <span class="text-xs text-yellow-500 font-medium">newer</span>
                                    {/if}
                                    {#if release.prerelease}
                                        <span class="text-xs text-muted-foreground">pre-release</span>
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
                                >
                                    {release.is_newer ? "Update" : "Downgrade"}
                                </Button>
                            {/if}
                        </div>
                    {/each}
                </div>
            {:else}
                <p class="text-sm text-muted-foreground">No releases found.</p>
            {/if}
        </Card.Content>
    </Card.Root>
</div>
