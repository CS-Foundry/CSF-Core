<script lang="ts">
    import { Button } from "$lib/components/ui/button/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import { Lock } from "@lucide/svelte";
    import { goto } from "$app/navigation";
    import { changePassword } from "$lib/auth/api";
    import { auth } from "$lib/auth/store.svelte";

    let newPassword = $state("");
    let confirmPassword = $state("");
    let loading = $state(false);
    let error = $state<string | null>(null);

    const mismatch = $derived(confirmPassword.length > 0 && newPassword !== confirmPassword);
    const canSubmit = $derived(newPassword.length >= 8 && newPassword === confirmPassword && !loading);

    async function handleSubmit() {
        if (!canSubmit || !auth.token) return;
        error = null;
        loading = true;
        try {
            await changePassword(auth.token, "", newPassword);
            goto("/");
        } catch {
            error = "Failed to change password";
        } finally {
            loading = false;
        }
    }
</script>

<img
    src="/logo/logo-csfx.svg"
    alt="CSFX Logo"
    class="size-20 mb-6 rounded-md p-2 invert dark:invert-0"
/>
<h1 class="text-2xl font-light mb-2">Change Password</h1>
<p class="text-sm text-muted-foreground mb-10">A new password is required before continuing</p>

<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="flex flex-col">
    <div class="flex flex-col gap-1 mb-4">
        <Label for="new-password" class="text-xs font-bold mb-1">New Password</Label>
        <div class="relative">
            <Lock class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
            <Input
                id="new-password"
                type="password"
                placeholder="••••••••"
                class="pl-9"
                bind:value={newPassword}
                autocomplete="new-password"
            />
        </div>
    </div>

    <div class="flex flex-col gap-1 mb-6">
        <Label for="confirm-password" class="text-xs font-bold mb-1">Confirm Password</Label>
        <div class="relative">
            <Lock class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
            <Input
                id="confirm-password"
                type="password"
                placeholder="••••••••"
                class="pl-9 {mismatch ? 'border-destructive' : ''}"
                bind:value={confirmPassword}
                autocomplete="new-password"
            />
        </div>
        {#if mismatch}
            <p class="text-xs text-destructive mt-1">Passwords do not match</p>
        {/if}
    </div>

    {#if error}
        <div class="mb-4 text-sm text-destructive">{error}</div>
    {/if}

    <Button class="w-full" type="submit" disabled={!canSubmit}>
        {loading ? "Changing Password..." : "Change Password"}
    </Button>
</form>
