<script lang="ts">
    import AuthRightPanel from "$lib/auth/right-panel.svelte";
    import type { Snippet } from "svelte";

    let { children }: { children: Snippet } = $props();
</script>

<div class="flex h-screen w-full">
    <div class="flex w-full lg:w-1/2 items-center justify-center p-8">
        <div class="w-full max-w-sm">
            {@render children()}
        </div>
    </div>
    <AuthRightPanel />
</div>
