<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog/index.js";
    import UserIcon from "@lucide/svelte/icons/user";
    import SlidersHorizontalIcon from "@lucide/svelte/icons/sliders-horizontal";
    import DownloadIcon from "@lucide/svelte/icons/download";
    import ScrollTextIcon from "@lucide/svelte/icons/scroll-text";
    import AccountSettings from "./account-settings.svelte";
    import GeneralSettings from "./general-settings.svelte";
    import UpdateSettings from "./update-settings.svelte";
    import LogsSettings from "./logs-settings.svelte";
    import { settingsDialog } from "./settings-store.svelte.js";

    type Section = "account" | "general" | "update" | "logs";

    const sections: { id: Section; label: string; icon: typeof UserIcon }[] = [
        { id: "account", label: "Account", icon: UserIcon },
        { id: "general", label: "General", icon: SlidersHorizontalIcon },
        { id: "update", label: "Update", icon: DownloadIcon },
        { id: "logs", label: "Logs", icon: ScrollTextIcon },
    ];

    let activeSection = $state<Section>("account");
</script>

<Dialog.Root open={settingsDialog.open} onOpenChange={(value) => settingsDialog.set(value)}>
    <Dialog.Content class="w-[min(90vw,880px)] h-[min(85vh,640px)] p-0">
        <div class="flex h-full">
            <nav class="w-56 shrink-0 border-r bg-muted/20 flex flex-col gap-1 p-3">
                <Dialog.Title class="px-2 py-2 text-sm font-semibold">Settings</Dialog.Title>
                {#each sections as section (section.id)}
                    <button
                        onclick={() => (activeSection = section.id)}
                        class="flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-sm text-left transition-colors {activeSection === section.id
                            ? 'bg-primary/10 text-primary font-medium'
                            : 'text-muted-foreground hover:bg-muted hover:text-foreground'}"
                    >
                        <section.icon class="size-4" />
                        {section.label}
                    </button>
                {/each}
            </nav>
            <div class="flex-1 overflow-y-auto p-6">
                {#if activeSection === "account"}
                    <AccountSettings />
                {:else if activeSection === "general"}
                    <GeneralSettings />
                {:else if activeSection === "update"}
                    <UpdateSettings />
                {:else if activeSection === "logs"}
                    <LogsSettings />
                {/if}
            </div>
        </div>
    </Dialog.Content>
</Dialog.Root>
