<script lang="ts">
    import { Button } from "$lib/components/ui/button/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import Checkbox from "$lib/components/ui/checkbox/checkbox.svelte";
    import { Lock, Mail } from "@lucide/svelte";
    import AuthRightPanel from "$lib/auth/right-panel.svelte";
    import { goto } from "$app/navigation";

    let email = "";
    let password = "";
    let remember = false;
    let loading = false;
    let error: string | null = null;

    async function handleSignIn(event?: Event) {
        goto("/");
    }
</script>

<div class="flex h-screen w-full">
    <div class="flex w-full lg:w-1/2 items-center justify-center p-8">
        <div class="w-full max-w-sm">
            <img
                src="/logo/logo-csfx.svg"
                alt="CSFX Logo"
                class="size-20 mb-6 rounded-md p-2 invert dark:invert-0"
            />
            <h1 class="text-2xl font-font-light mb-10">Sign in</h1>

            <form on:submit|preventDefault={handleSignIn} class="flex flex-col">
                <div class="flex flex-col gap-1 mb-4">
                    <Label for="email" class="text-xs font-bold mb-1"
                        >Email Address</Label
                    >
                    <div class="relative">
                        <Mail
                            class="absolute left-3 top-1/2 -translate-y-1/2 text-color-foreground size-4"
                        />
                        <Input
                            id="email"
                            type="email"
                            placeholder="you@example.com"
                            class="pl-9 pt-2 pb-2"
                            bind:value={email}
                        />
                    </div>
                </div>

                <div class="flex flex-col gap-1 mb-4">
                    <Label for="password" class="text-xs font-bold mb-1"
                        >Password</Label
                    >
                    <div class="relative">
                        <Lock
                            class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-color-foreground"
                        />
                        <Input
                            id="password"
                            type="password"
                            placeholder="••••••••"
                            class="pl-9"
                            bind:value={password}
                        />
                    </div>
                </div>

                <div class="flex items-center gap-2 mb-6">
                    <Checkbox id="remember" bind:checked={remember} />
                    <Label for="remember" class="cursor-pointer font-normal"
                        >Remember me</Label
                    >
                </div>

                {#if error}
                    <div class="mb-4 text-sm text-red-500">{error}</div>
                {/if}

                <Button class="w-full" type="submit" disabled={loading}>
                    {#if loading}
                        Signing in...
                    {:else}
                        Sign in
                    {/if}
                </Button>
            </form>
        </div>
    </div>
    <AuthRightPanel />
</div>
