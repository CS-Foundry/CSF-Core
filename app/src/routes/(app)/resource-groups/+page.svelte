<script lang="ts">
    import { goto } from "$app/navigation";
    import { auth } from "$lib/auth/store.svelte";
    import {
        listResourceGroups,
        createResourceGroup,
        deleteResourceGroup,
        type ResourceGroup,
    } from "$lib/api/resource-groups";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import { Button } from "$lib/components/ui/button/index.js";

    let groups = $state<ResourceGroup[]>([]);
    let loading = $state(true);
    let error = $state<string | null>(null);
    let creating = $state(false);
    let createDialog = $state<HTMLDialogElement | null>(null);

    let newName = $state("");
    let newCidr = $state("10.100.0.0/24");
    let newDescription = $state("");
    let createError = $state<string | null>(null);

    async function load() {
        if (!auth.token) return;
        try {
            groups = await listResourceGroups(auth.token);
        } catch (e) {
            error = e instanceof Error ? e.message : "Failed to load resource groups";
        } finally {
            loading = false;
        }
    }

    async function handleCreate() {
        if (!auth.token || !newName || !newCidr) return;
        creating = true;
        createError = null;
        try {
            const created = await createResourceGroup(auth.token, {
                name: newName,
                description: newDescription || undefined,
                internal_cidr: newCidr,
            });
            groups = [...groups, created];
            createDialog?.close();
            newName = "";
            newCidr = "10.100.0.0/24";
            newDescription = "";
        } catch (e) {
            createError = e instanceof Error ? e.message : "Failed to create resource group";
        } finally {
            creating = false;
        }
    }

    async function handleDelete(id: string) {
        if (!auth.token) return;
        try {
            await deleteResourceGroup(auth.token, id);
            groups = groups.filter((g) => g.id !== id);
        } catch (e) {
            error = e instanceof Error ? e.message : "Failed to delete resource group";
        }
    }

    function statusClass(status: string): string {
        switch (status.toLowerCase()) {
            case "active": return "text-green-500";
            case "deleting": return "text-red-500";
            case "suspended": return "text-yellow-500";
            default: return "text-muted-foreground";
        }
    }

    let loadStarted = false;

    $effect(() => {
        if (auth.token && !loadStarted) {
            loadStarted = true;
            load();
        }
    });
</script>

<dialog
    bind:this={createDialog}
    class="fixed inset-0 z-50 m-auto w-full max-w-md rounded-xl border bg-background shadow-xl p-0 backdrop:bg-black/40"
    onclose={() => { createError = null; }}
>
    <div class="flex flex-col gap-5 p-6">
        <div class="flex items-center justify-between">
            <h2 class="text-base font-semibold">New Resource Group</h2>
            <button
                class="text-muted-foreground hover:text-foreground"
                onclick={() => createDialog?.close()}
                aria-label="Close"
            >
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
        </div>
        <div class="flex flex-col gap-3">
            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="rg-name">Name</label>
                <input
                    id="rg-name"
                    class="border rounded px-3 py-1.5 text-sm bg-background"
                    placeholder="production"
                    bind:value={newName}
                />
            </div>
            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="rg-cidr">Internal CIDR</label>
                <input
                    id="rg-cidr"
                    class="border rounded px-3 py-1.5 text-sm bg-background font-mono"
                    placeholder="10.100.0.0/24"
                    bind:value={newCidr}
                />
            </div>
            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="rg-desc">Description</label>
                <input
                    id="rg-desc"
                    class="border rounded px-3 py-1.5 text-sm bg-background"
                    placeholder="Optional"
                    bind:value={newDescription}
                />
            </div>
        </div>
        {#if createError}
            <p class="text-xs text-destructive">{createError}</p>
        {/if}
        <div class="flex gap-2 justify-end">
            <Button size="sm" variant="outline" onclick={() => createDialog?.close()}>Cancel</Button>
            <Button size="sm" onclick={handleCreate} disabled={creating || !newName || !newCidr}>
                {creating ? "Creating..." : "Create"}
            </Button>
        </div>
    </div>
</dialog>

<header class="flex h-16 shrink-0 items-center gap-2 px-4 border-b">
    <Sidebar.Trigger class="-ms-1" />
    <span class="text-sm text-muted-foreground">/</span>
    <span class="text-sm font-medium">Resource Groups</span>
</header>

<div class="flex flex-col gap-6 p-6">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-xl font-semibold tracking-tight">Resource Groups</h1>
            <p class="text-sm text-muted-foreground mt-0.5">
                Isolated namespaces with dedicated internal networks
            </p>
        </div>
        <Button size="sm" onclick={() => createDialog?.showModal()}>
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="12" y1="5" x2="12" y2="19"/>
                <line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
            New Resource Group
        </Button>
    </div>

    {#if error}
        <p class="text-sm text-destructive">{error}</p>
    {/if}

    <div class="border rounded-lg overflow-hidden">
        <table class="w-full text-sm">
            <thead class="bg-muted/50">
                <tr>
                    <th class="text-left px-4 py-3 font-medium text-muted-foreground">ID</th>
                    <th class="text-left px-4 py-3 font-medium text-muted-foreground">Name</th>
                    <th class="text-left px-4 py-3 font-medium text-muted-foreground">CIDR</th>
                    <th class="text-left px-4 py-3 font-medium text-muted-foreground">Description</th>
                    <th class="text-left px-4 py-3 font-medium text-muted-foreground">Status</th>
                    <th class="text-left px-4 py-3 font-medium text-muted-foreground">Created</th>
                    <th class="px-4 py-3"></th>
                </tr>
            </thead>
            <tbody>
                {#if loading}
                    <tr>
                        <td colspan="7" class="px-4 py-8 text-center text-muted-foreground">Loading...</td>
                    </tr>
                {:else if groups.length === 0}
                    <tr>
                        <td colspan="7" class="px-4 py-8 text-center text-muted-foreground">
                            No resource groups. Create one to get started.
                        </td>
                    </tr>
                {:else}
                    {#each groups as group (group.id)}
                        <tr
                            class="border-t hover:bg-muted/30 transition-colors cursor-pointer"
                            onclick={() => goto(`/resource-groups/${group.id}`)}
                        >
                            <td class="px-4 py-3 font-mono text-xs text-muted-foreground">{group.id.slice(0, 8)}</td>
                            <td class="px-4 py-3 font-medium">{group.name}</td>
                            <td class="px-4 py-3 font-mono text-xs">{group.internal_cidr}</td>
                            <td class="px-4 py-3 text-muted-foreground">{group.description ?? "-"}</td>
                            <td class="px-4 py-3">
                                <span class="font-medium {statusClass(group.status)}">{group.status}</span>
                            </td>
                            <td class="px-4 py-3 text-xs text-muted-foreground">{group.created_at.slice(0, 10)}</td>
                            <td class="px-4 py-3 text-right">
                                <Button
                                    size="sm"
                                    variant="ghost"
                                    class="text-destructive hover:text-destructive"
                                    onclick={(e) => { e.stopPropagation(); handleDelete(group.id); }}
                                >
                                    Delete
                                </Button>
                            </td>
                        </tr>
                    {/each}
                {/if}
            </tbody>
        </table>
    </div>
</div>
