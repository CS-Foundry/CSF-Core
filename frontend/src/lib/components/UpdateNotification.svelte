<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Download, RefreshCw, ExternalLink, X } from '@lucide/svelte';
  import { updateStore, updateAvailable } from '$lib/stores/updates';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import * as Alert from '$lib/components/ui/alert';

  let showChangelog = $state(false);
  let changelog = $state('');
  let isInstalling = $state(false);
  let message = $state('');
  let messageType: 'success' | 'error' | '' = $state('');

  onMount(() => {
    // Start automatic update checks
    updateStore.startAutoCheck();
  });

  onDestroy(() => {
    // Stop automatic update checks
    updateStore.stopAutoCheck();
  });

  async function handleInstallUpdate() {
    const versionInfo = $updateStore.versionInfo;
    if (!versionInfo) return;

    try {
      isInstalling = true;
      const response = await updateStore.installUpdate(versionInfo.latest_version);

      message = response.message;
      messageType = 'success';

      // Close dialog after 2 seconds
      setTimeout(() => {
        showChangelog = false;
      }, 2000);
    } catch (error) {
      message = error instanceof Error ? error.message : 'Unbekannter Fehler';
      messageType = 'error';
    } finally {
      isInstalling = false;
    }
  }

  function openChangelog() {
    if ($updateStore.versionInfo?.changelog) {
      changelog = $updateStore.versionInfo.changelog;
      showChangelog = true;
      message = '';
      messageType = '';
    }
  }

  // Parse markdown-style changelog to HTML
  function parseChangelog(text: string): string {
    if (!text) return '';

    return text
      .replace(/### (.*)/g, '<h3 class="text-lg font-semibold mt-4 mb-2">$1</h3>')
      .replace(/## (.*)/g, '<h2 class="text-xl font-bold mt-4 mb-2">$1</h2>')
      .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
      .replace(/\* (.*)/g, '<li class="ml-4">$1</li>')
      .replace(/\n\n/g, '<br/><br/>')
      .replace(
        /\[([^\]]+)\]\(([^)]+)\)/g,
        '<a href="$2" class="text-blue-500 hover:underline" target="_blank">$1</a>'
      );
  }
</script>

{#if $updateAvailable && $updateStore.versionInfo}
  <div class="p-2">
    <Card.Root class="border-primary/50 bg-primary/5">
      <Card.Header class="pb-3">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <Download class="h-4 w-4 text-primary" />
            <Card.Title class="text-sm">Update verfügbar</Card.Title>
          </div>
          <Badge variant="secondary" class="text-xs">
            v{$updateStore.versionInfo.latest_version}
          </Badge>
        </div>
      </Card.Header>
      <Card.Content class="pb-3">
        <p class="text-xs text-muted-foreground mb-3">
          Eine neue Version ist verfügbar. Aktuelle Version: v{$updateStore.versionInfo
            .current_version}
        </p>
        <div class="flex flex-col gap-2">
          <Button size="sm" onclick={openChangelog} class="w-full">
            <RefreshCw class="mr-2 h-3 w-3" />
            Changelog anzeigen & Installieren
          </Button>
        </div>
      </Card.Content>
    </Card.Root>
  </div>
{/if}

<Dialog.Root bind:open={showChangelog}>
  <Dialog.Content class="max-w-2xl max-h-[80vh] overflow-hidden flex flex-col">
    <Dialog.Header>
      <Dialog.Title>Update auf v{$updateStore.versionInfo?.latest_version}</Dialog.Title>
      <Dialog.Description>Neue Features und Verbesserungen in dieser Version</Dialog.Description>
    </Dialog.Header>

    <div class="flex-1 overflow-y-auto pr-2">
      {#if message}
        <Alert.Root variant={messageType === 'error' ? 'destructive' : 'default'} class="mb-4">
          <Alert.AlertDescription>{message}</Alert.AlertDescription>
        </Alert.Root>
      {/if}

      <div class="prose prose-sm dark:prose-invert max-w-none">
        {@html parseChangelog(changelog)}
      </div>
    </div>

    <Dialog.Footer class="flex-shrink-0 gap-2">
      <Button variant="outline" onclick={() => (showChangelog = false)} disabled={isInstalling}>
        Abbrechen
      </Button>
      {#if $updateStore.versionInfo}
        <Button
          variant="outline"
          href={$updateStore.versionInfo.release_url}
          target="_blank"
          disabled={isInstalling}
        >
          <ExternalLink class="mr-2 h-4 w-4" />
          Auf GitHub ansehen
        </Button>
      {/if}
      <Button onclick={handleInstallUpdate} disabled={isInstalling}>
        {#if isInstalling}
          <RefreshCw class="mr-2 h-4 w-4 animate-spin" />
          Installiere...
        {:else}
          <Download class="mr-2 h-4 w-4" />
          Jetzt installieren
        {/if}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
