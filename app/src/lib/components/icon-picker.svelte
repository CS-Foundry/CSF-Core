<script lang="ts">
    import Icon from "@iconify/svelte";

    let { icon = $bindable(), color = $bindable() }: { icon: string; color: string } = $props();

    const SUGGESTED_ICONS = [
        "mdi:cube-outline",
        "mdi:server",
        "mdi:database",
        "mdi:rocket-launch-outline",
        "mdi:shield-check-outline",
        "mdi:layers-outline",
        "mdi:cloud-outline",
        "mdi:flask-outline",
        "mdi:cog-outline",
        "mdi:web",
        "mdi:folder-outline",
        "mdi:fire",
    ];

    const SUGGESTED_COLORS = [
        "#6366f1",
        "#22c55e",
        "#ef4444",
        "#f59e0b",
        "#06b6d4",
        "#a855f7",
        "#ec4899",
        "#64748b",
    ];
</script>

<div class="flex items-center gap-3">
    <div
        class="flex h-12 w-12 shrink-0 items-center justify-center rounded-lg border"
        style="background-color: {color}20; color: {color};"
    >
        <Icon icon={icon || "mdi:cube-outline"} width={24} height={24} />
    </div>
    <div class="flex flex-col gap-1 flex-1">
        <label class="text-xs text-muted-foreground" for="icon-name">Icon name</label>
        <input
            id="icon-name"
            class="border rounded px-3 py-1.5 text-sm bg-background font-mono"
            placeholder="mdi:cube-outline"
            bind:value={icon}
        />
    </div>
</div>
<div class="flex flex-wrap gap-1.5">
    {#each SUGGESTED_ICONS as suggestion (suggestion)}
        <button
            type="button"
            class="flex items-center justify-center w-8 h-8 rounded border transition-colors {icon === suggestion ? 'border-foreground' : 'hover:bg-muted'}"
            onclick={() => (icon = suggestion)}
            aria-label={suggestion}
            title={suggestion}
        >
            <Icon icon={suggestion} width={16} height={16} />
        </button>
    {/each}
</div>
<div class="flex flex-col gap-1">
    <label class="text-xs text-muted-foreground" for="rg-color">Color</label>
    <div class="flex items-center gap-2">
        <input id="rg-color" type="color" class="w-9 h-8 rounded border bg-background cursor-pointer" bind:value={color} />
        <div class="flex gap-1.5">
            {#each SUGGESTED_COLORS as suggestion (suggestion)}
                <button
                    type="button"
                    class="w-6 h-6 rounded-full border transition-transform {color === suggestion ? 'ring-2 ring-offset-2 ring-offset-background ring-foreground' : ''}"
                    style="background-color: {suggestion};"
                    onclick={() => (color = suggestion)}
                    aria-label={suggestion}
                    title={suggestion}
                ></button>
            {/each}
        </div>
    </div>
</div>
