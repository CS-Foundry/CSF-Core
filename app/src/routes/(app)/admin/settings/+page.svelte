<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { settingsDialog } from "$lib/components/settings/settings-store.svelte.js";

    onMount(() => {
        settingsDialog.show();
    });

    $effect(() => {
        if (!settingsDialog.open) goto("/");
    });
</script>
