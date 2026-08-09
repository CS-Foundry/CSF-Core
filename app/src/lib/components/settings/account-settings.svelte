<script lang="ts">
    import { Button } from "$lib/components/ui/button/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import { Switch } from "$lib/components/ui/switch/index.js";
    import * as InputOTP from "$lib/components/ui/input-otp/index.js";
    import Spinner from "$lib/components/ui/spinner/spinner.svelte";
    import { auth } from "$lib/auth/store.svelte";
    import { changeEmail, changePassword, setup2FA, enable2FA, disable2FA } from "$lib/auth/api";
    import { toast } from "svelte-sonner";

    let email = $state(auth.user?.email ?? "");
    let emailSaving = $state(false);

    let oldPassword = $state("");
    let newPassword = $state("");
    let confirmPassword = $state("");
    let passwordSaving = $state(false);

    let twoFactorStep = $state<"idle" | "enrolling" | "disabling">("idle");
    let qrCode = $state("");
    let otpValue = $state("");
    let twoFactorBusy = $state(false);

    const emailChanged = $derived(email.trim() !== "" && email !== (auth.user?.email ?? ""));
    const passwordValid = $derived(newPassword.length >= 8 && newPassword === confirmPassword);

    async function saveEmail() {
        if (!auth.token || !emailChanged) return;
        emailSaving = true;
        try {
            await changeEmail(auth.token, email.trim());
            auth.setUser({ ...auth.user!, email: email.trim() });
            toast.success("Email updated");
        } catch {
            toast.error("Failed to update email");
        } finally {
            emailSaving = false;
        }
    }

    async function savePassword() {
        if (!auth.token || !passwordValid) return;
        passwordSaving = true;
        try {
            await changePassword(auth.token, oldPassword, newPassword);
            oldPassword = "";
            newPassword = "";
            confirmPassword = "";
            toast.success("Password updated");
        } catch {
            toast.error("Failed to update password", { description: "Check your current password" });
        } finally {
            passwordSaving = false;
        }
    }

    async function toggleTwoFactor(next: boolean) {
        if (!auth.token) return;
        if (next) {
            twoFactorBusy = true;
            try {
                const response = await setup2FA(auth.token);
                qrCode = response.qr_code;
                twoFactorStep = "enrolling";
            } catch {
                toast.error("Failed to start 2FA setup");
            } finally {
                twoFactorBusy = false;
            }
        } else {
            twoFactorStep = "disabling";
            otpValue = "";
        }
    }

    async function confirmEnable() {
        if (!auth.token || otpValue.length !== 6) return;
        twoFactorBusy = true;
        try {
            await enable2FA(auth.token, otpValue);
            auth.setUser({ ...auth.user!, two_factor_enabled: true });
            twoFactorStep = "idle";
            otpValue = "";
            toast.success("Two-factor authentication enabled");
        } catch {
            toast.error("Invalid code");
            otpValue = "";
        } finally {
            twoFactorBusy = false;
        }
    }

    async function confirmDisable() {
        if (!auth.token || otpValue.length !== 6) return;
        twoFactorBusy = true;
        try {
            await disable2FA(auth.token, otpValue);
            auth.setUser({ ...auth.user!, two_factor_enabled: false });
            twoFactorStep = "idle";
            otpValue = "";
            toast.success("Two-factor authentication disabled");
        } catch {
            toast.error("Invalid code");
            otpValue = "";
        } finally {
            twoFactorBusy = false;
        }
    }

    function cancelTwoFactorStep() {
        twoFactorStep = "idle";
        otpValue = "";
        qrCode = "";
    }
</script>

<div class="flex flex-col gap-8 max-w-xl">
    <section class="flex flex-col gap-4">
        <h3 class="text-sm font-semibold">Profile</h3>
        <div class="flex flex-col gap-1.5">
            <Label for="settings-username" class="text-xs text-muted-foreground">Username</Label>
            <Input id="settings-username" value={auth.user?.username ?? ""} disabled />
        </div>
        <div class="flex flex-col gap-1.5">
            <Label for="settings-email" class="text-xs text-muted-foreground">Email</Label>
            <div class="flex gap-2">
                <Input id="settings-email" type="email" bind:value={email} placeholder="you@example.com" />
                <Button onclick={saveEmail} disabled={!emailChanged || emailSaving} size="sm">
                    {#if emailSaving}<Spinner />{:else}Save{/if}
                </Button>
            </div>
        </div>
    </section>

    <section class="flex flex-col gap-4 pt-6 border-t">
        <h3 class="text-sm font-semibold">Password</h3>
        <div class="flex flex-col gap-1.5">
            <Label for="settings-old-password" class="text-xs text-muted-foreground">Current password</Label>
            <Input id="settings-old-password" type="password" bind:value={oldPassword} autocomplete="current-password" />
        </div>
        <div class="flex flex-col gap-1.5">
            <Label for="settings-new-password" class="text-xs text-muted-foreground">New password</Label>
            <Input id="settings-new-password" type="password" bind:value={newPassword} autocomplete="new-password" />
        </div>
        <div class="flex flex-col gap-1.5">
            <Label for="settings-confirm-password" class="text-xs text-muted-foreground">Confirm new password</Label>
            <Input id="settings-confirm-password" type="password" bind:value={confirmPassword} autocomplete="new-password" />
        </div>
        <Button onclick={savePassword} disabled={!passwordValid || passwordSaving} size="sm" class="w-fit">
            {#if passwordSaving}<Spinner />{:else}Update password{/if}
        </Button>
    </section>

    <section class="flex flex-col gap-4 pt-6 border-t">
        <div class="flex items-center justify-between">
            <div>
                <h3 class="text-sm font-semibold">Two-factor authentication</h3>
                <p class="text-xs text-muted-foreground mt-0.5">Require an authenticator code at login</p>
            </div>
            <Switch
                checked={auth.user?.two_factor_enabled ?? false}
                disabled={twoFactorBusy || twoFactorStep !== "idle"}
                onCheckedChange={toggleTwoFactor}
            />
        </div>

        {#if twoFactorStep === "enrolling"}
            <div class="flex flex-col gap-4 p-4 rounded-lg border bg-muted/30">
                <p class="text-xs text-muted-foreground">Scan with your authenticator app, then enter the code</p>
                {#if qrCode}
                    <img
                        src="data:image/png;base64,{qrCode}"
                        alt="2FA QR code"
                        class="size-40 rounded-md self-center bg-white p-2"
                    />
                {/if}
                <InputOTP.Root maxlength={6} bind:value={otpValue}>
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
                <div class="flex gap-2">
                    <Button onclick={confirmEnable} disabled={otpValue.length !== 6 || twoFactorBusy} size="sm">
                        {#if twoFactorBusy}<Spinner />{:else}Confirm{/if}
                    </Button>
                    <Button onclick={cancelTwoFactorStep} variant="outline" size="sm">Cancel</Button>
                </div>
            </div>
        {:else if twoFactorStep === "disabling"}
            <div class="flex flex-col gap-4 p-4 rounded-lg border bg-muted/30">
                <p class="text-xs text-muted-foreground">Enter your current code to disable 2FA</p>
                <InputOTP.Root maxlength={6} bind:value={otpValue}>
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
                <div class="flex gap-2">
                    <Button onclick={confirmDisable} disabled={otpValue.length !== 6 || twoFactorBusy} variant="destructive" size="sm">
                        {#if twoFactorBusy}<Spinner />{:else}Disable{/if}
                    </Button>
                    <Button onclick={cancelTwoFactorStep} variant="outline" size="sm">Cancel</Button>
                </div>
            </div>
        {/if}
    </section>
</div>
