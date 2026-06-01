<script lang="ts">
    import { Button } from "$lib/components/ui/button/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import { Lock } from "@lucide/svelte";
    import { goto } from "$app/navigation";

    let password = "";
    let passwordRepeat = "";
    let loading = false;
    let error: string | null = null;

    async function handleChangePassword(event?: Event) {
        goto("/");
    }
</script>

<img
    src="/logo/logo-csfx.svg"
    alt="CSFX Logo"
    class="size-20 mb-6 rounded-md p-2 invert dark:invert-0"
/>
<h1 class="text-2xl font-font-light mb-10">Change Password</h1>

<form on:submit|preventDefault={handleChangePassword} class="flex flex-col">
    <div class="flex flex-col gap-1 mb-4">
        <Label for="password" class="text-xs font-bold mb-1">Password</Label>
        <div class="relative">
            <Lock class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-color-foreground" />
            <Input
                id="password"
                type="password"
                placeholder="••••••••"
                class="pl-9"
                bind:value={password}
            />
        </div>
    </div>

    <div class="flex flex-col gap-1 mb-4">
        <Label for="password-repeat" class="text-xs font-bold mb-1">Repeat Password</Label>
        <div class="relative">
            <Lock class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-color-foreground" />
            <Input
                id="password-repeat"
                type="password"
                placeholder="••••••••"
                class="pl-9"
                bind:value={passwordRepeat}
            />
        </div>
    </div>

    {#if error}
        <div class="mb-4 text-sm text-red-500">{error}</div>
    {/if}

    <Button class="w-full" type="submit" disabled={loading}>
        {#if loading}
            Changing Password...
        {:else}
            Change Password
        {/if}
    </Button>
</form>
