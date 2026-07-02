<script lang="ts">
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import UpdateSettings from "$lib/components/settings/update-settings.svelte";
    import LogsSettings from "$lib/components/settings/logs-settings.svelte";

    type Tab = "general" | "update" | "logs";
    let activeTab = $state<Tab>("general");
</script>

<header class="flex h-16 shrink-0 items-center gap-2 px-4 border-b">
    <Sidebar.Trigger class="-ms-1" />
    <span class="text-sm font-medium text-muted-foreground">Admin / Settings</span>
</header>

<div class="flex flex-1 flex-col gap-6 p-6">
    <h1 class="text-2xl font-bold">Settings</h1>

    <div class="flex gap-1 border-b">
        <button
            onclick={() => (activeTab = "general")}
            class="px-4 py-2 text-sm font-medium transition-colors border-b-2 -mb-px {activeTab === 'general'
                ? 'border-foreground text-foreground'
                : 'border-transparent text-muted-foreground hover:text-foreground'}"
        >
            General
        </button>
        <button
            onclick={() => (activeTab = "update")}
            class="px-4 py-2 text-sm font-medium transition-colors border-b-2 -mb-px {activeTab === 'update'
                ? 'border-foreground text-foreground'
                : 'border-transparent text-muted-foreground hover:text-foreground'}"
        >
            Update
        </button>
        <button
            onclick={() => (activeTab = "logs")}
            class="px-4 py-2 text-sm font-medium transition-colors border-b-2 -mb-px {activeTab === 'logs'
                ? 'border-foreground text-foreground'
                : 'border-transparent text-muted-foreground hover:text-foreground'}"
        >
            Logs
        </button>
    </div>

    <div class="mt-2">
        {#if activeTab === "general"}
            <p class="text-sm text-muted-foreground">No general settings configured.</p>
        {:else if activeTab === "update"}
            <UpdateSettings />
        {:else if activeTab === "logs"}
            <LogsSettings />
        {/if}
    </div>
</div>
