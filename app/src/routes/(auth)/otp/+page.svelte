<script lang="ts">
    import { Button } from "$lib/components/ui/button/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import * as InputOTP from "$lib/components/ui/input-otp/index.js";
    import { login } from "$lib/auth/api";
    import { auth } from "$lib/auth/store.svelte";

    const username = $derived($page.url.searchParams.get("username") ?? "");
    const password = $derived($page.url.searchParams.get("password") ?? "");

    let otpValue = $state("");
    let loading = $state(false);
    let error = $state<string | null>(null);

    async function handleVerify() {
        if (otpValue.length !== 6) return;
        error = null;
        loading = true;
        try {
            const response = await login(username, password, otpValue);
            auth.setSession(response);
            if (response.force_password_change) {
                goto("/pw_change");
            } else {
                goto("/");
            }
        } catch {
            error = "Invalid code";
            otpValue = "";
        } finally {
            loading = false;
        }
    }

    $effect(() => {
        if (otpValue.length === 6) handleVerify();
    });
</script>

<img
    src="/logo/logo-csfx.svg"
    alt="CSFX Logo"
    class="size-20 mb-6 rounded-md p-2 invert dark:invert-0"
/>
<h1 class="text-2xl font-light mb-2">2FA Verification</h1>
<p class="text-sm text-muted-foreground mb-10">Enter the 6-digit code from your authenticator app</p>

<form onsubmit={(e) => { e.preventDefault(); handleVerify(); }} class="flex flex-col">
    <div class="flex flex-col gap-1 mb-20">
        <Label class="text-xs font-bold mb-1">TOTP</Label>
        <InputOTP.Root maxlength={6} class="w-full" bind:value={otpValue}>
            {#snippet children({ cells })}
                <InputOTP.Group class="flex-1">
                    {#each cells.slice(0, 3) as cell (cell)}
                        <InputOTP.Slot {cell} class="flex-1 w-full" />
                    {/each}
                </InputOTP.Group>
                <InputOTP.Separator />
                <InputOTP.Group class="flex-1">
                    {#each cells.slice(3, 6) as cell (cell)}
                        <InputOTP.Slot {cell} class="flex-1 w-full" />
                    {/each}
                </InputOTP.Group>
            {/snippet}
        </InputOTP.Root>
    </div>

    {#if error}
        <div class="mb-4 text-sm text-destructive">{error}</div>
    {/if}

    <Button class="w-full" type="submit" disabled={loading || otpValue.length !== 6}>
        {loading ? "Verifying..." : "Verify"}
    </Button>
</form>
