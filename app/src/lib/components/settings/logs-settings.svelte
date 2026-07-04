<script lang="ts">
    import { onMount } from "svelte";
    import * as Card from "$lib/components/ui/card/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { auth } from "$lib/auth/store.svelte";
    import { getLogsRetention, updateLogsRetention } from "$lib/api/logs";

    const MIN_RETENTION_DAYS = 1;
    const MAX_RETENTION_DAYS = 365;

    let retentionDays = $state<number | null>(null);
    let inputValue = $state("");
    let loading = $state(true);
    let saving = $state(false);
    let error = $state<string | null>(null);
    let saved = $state(false);

    async function load() {
        if (!auth.token) return;
        loading = true;
        try {
            const response = await getLogsRetention(auth.token);
            retentionDays = response.retention_days;
            inputValue = String(response.retention_days);
            error = null;
        } catch (e) {
            error = e instanceof Error ? e.message : "failed to load retention setting";
        } finally {
            loading = false;
        }
    }

    async function save() {
        if (!auth.token) return;
        const days = Number(inputValue);
        if (!Number.isInteger(days) || days < MIN_RETENTION_DAYS || days > MAX_RETENTION_DAYS) {
            error = `retention days must be between ${MIN_RETENTION_DAYS} and ${MAX_RETENTION_DAYS}`;
            return;
        }
        saving = true;
        saved = false;
        try {
            const response = await updateLogsRetention(auth.token, days);
            retentionDays = response.retention_days;
            error = null;
            saved = true;
        } catch (e) {
            error = e instanceof Error ? e.message : "failed to save retention setting";
        } finally {
            saving = false;
        }
    }

    onMount(load);
</script>

{#if error}
    <div class="mb-4 px-3 py-2 rounded-md bg-destructive/10 border border-destructive/20 text-sm text-destructive">
        {error}
    </div>
{/if}

<div class="flex flex-col gap-6 max-w-2xl">
    <Card.Root>
        <Card.Header>
            <Card.Title>Log retention</Card.Title>
            <Card.Description>
                Logs older than this many days are deleted automatically every hour.
            </Card.Description>
        </Card.Header>
        <Card.Content>
            {#if loading}
                <div class="h-9 w-32 rounded bg-muted animate-pulse"></div>
            {:else}
                <div class="flex items-end gap-3">
                    <div class="flex flex-col gap-1">
                        <label class="text-xs text-muted-foreground" for="retention-input">Retention (days)</label>
                        <Input
                            id="retention-input"
                            type="number"
                            min={MIN_RETENTION_DAYS}
                            max={MAX_RETENTION_DAYS}
                            bind:value={inputValue}
                            class="w-32"
                        />
                    </div>
                    <Button onclick={save} disabled={saving || inputValue === String(retentionDays)} size="sm">
                        {saving ? "Saving..." : "Save"}
                    </Button>
                    {#if saved && !saving}
                        <span class="text-xs text-green-500">Saved</span>
                    {/if}
                </div>
            {/if}
        </Card.Content>
    </Card.Root>
</div>
