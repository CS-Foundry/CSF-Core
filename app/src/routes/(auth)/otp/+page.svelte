<script lang="ts">
    import { Button } from "$lib/components/ui/button/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import { goto } from "$app/navigation";
    import * as InputOTP from "$lib/components/ui/input-otp/index.js";

    let loading = false;
    let error: string | null = null;

    async function handleVerify(event?: Event) {
        goto("/");
    }
</script>

<img
    src="/logo/logo-csfx.svg"
    alt="CSFX Logo"
    class="size-20 mb-6 rounded-md p-2 invert dark:invert-0"
/>
<h1 class="text-2xl font-font-light mb-10">2FA Verification</h1>

<form on:submit|preventDefault={handleVerify} class="flex flex-col">
    <div class="flex flex-col gap-1 mb-20">
        <Label class="text-xs font-bold mb-1">TOTP</Label>
        <InputOTP.Root maxlength={6} class="w-full">
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
        <div class="mb-4 text-sm text-red-500">{error}</div>
    {/if}

    <Button class="w-full" type="submit" disabled={loading}>
        {#if loading}
            Verifying...
        {:else}
            Verify
        {/if}
    </Button>
</form>
