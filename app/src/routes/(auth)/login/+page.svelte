<script lang="ts">
    import { Button } from "$lib/components/ui/button/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import { Lock, User } from "@lucide/svelte";
    import { goto } from "$app/navigation";
    import { login, TwoFactorRequiredError } from "$lib/auth/api";
    import { auth } from "$lib/auth/store.svelte";

    let username = $state("");
    let password = $state("");
    let loading = $state(false);
    let error = $state<string | null>(null);

    async function handleSubmit() {
        error = null;
        loading = true;
        try {
            const response = await login(username, password);
            auth.setSession(response);
            if (response.force_password_change) {
                goto("/pw_change");
            } else {
                goto("/");
            }
        } catch (err) {
            if (err instanceof TwoFactorRequiredError) {
                goto(`/otp?username=${encodeURIComponent(err.username)}&password=${encodeURIComponent(err.password)}`);
            } else {
                error = "Invalid credentials";
            }
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
<h1 class="text-2xl font-light mb-10">Sign in</h1>

<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="flex flex-col">
    <div class="flex flex-col gap-1 mb-4">
        <Label for="username" class="text-xs font-bold mb-1">Username</Label>
        <div class="relative">
            <User class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground size-4" />
            <Input
                id="username"
                type="text"
                placeholder="admin"
                class="pl-9"
                bind:value={username}
                autocomplete="username"
            />
        </div>
    </div>

    <div class="flex flex-col gap-1 mb-6">
        <Label for="password" class="text-xs font-bold mb-1">Password</Label>
        <div class="relative">
            <Lock class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
            <Input
                id="password"
                type="password"
                placeholder="••••••••"
                class="pl-9"
                bind:value={password}
                autocomplete="current-password"
            />
        </div>
    </div>

    {#if error}
        <div class="mb-4 text-sm text-destructive">{error}</div>
    {/if}

    <Button class="w-full" type="submit" disabled={loading || !username || !password}>
        {loading ? "Signing in..." : "Sign in"}
    </Button>
</form>
