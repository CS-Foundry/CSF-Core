<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { ArrowUpCircle } from "@lucide/svelte";
    import { auth } from "$lib/auth/store.svelte";
    import { getReleases } from "$lib/api/system";

    let latestStable = $state<string | null>(null);

    onMount(async () => {
        if (!auth.token) return;
        try {
            const data = await getReleases(auth.token);
            if (data.update_available) {
                latestStable = data.latest_stable;
            }
        } catch {
            // non-critical
        }
    });
</script>

{#if latestStable}
    <button
        onclick={() => goto("/admin/settings")}
        class="mx-2 mb-2 flex items-start gap-3 rounded-lg border border-yellow-500/30 bg-yellow-500/10 p-3 text-left transition-colors hover:bg-yellow-500/20 group-data-[collapsible=icon]:hidden cursor-pointer w-[calc(100%-1rem)]"
    >
        <ArrowUpCircle class="mt-0.5 size-4 shrink-0 text-yellow-500" />
        <div class="flex flex-col gap-0.5 min-w-0">
            <span class="text-xs font-medium text-yellow-500">Update available</span>
            <span class="truncate text-xs text-muted-foreground">{latestStable}</span>
        </div>
    </button>
{/if}
