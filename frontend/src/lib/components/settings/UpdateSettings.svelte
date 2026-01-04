<script lang="ts">
  import { onMount } from 'svelte';
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import { updateStore } from '$lib/stores/updates';
  import { Download, RefreshCw, ExternalLink, CheckCircle2, Info } from '@lucide/svelte';

  let isChecking = $state(false);
  let isInstalling = $state(false);
  let message = $state('');
  let messageType: 'success' | 'error' | '' = $state('');

  onMount(() => {
    // Check for updates when component mounts
    checkForUpdates();
  });

  async function checkForUpdates() {
    isChecking = true;
    message = '';
    messageType = '';
    try {
      await updateStore.checkForUpdates();
      message = 'Update-Check abgeschlossen';
      messageType = 'success';
    } catch (error) {
      message = error instanceof Error ? error.message : 'Unbekannter Fehler';
      messageType = 'error';
    } finally {
      isChecking = false;
    }
  }

  async function installUpdate() {
    if (!$updateStore.versionInfo) return;

    isInstalling = true;
    message = '';
    messageType = '';
    try {
      const response = await updateStore.installUpdate($updateStore.versionInfo.latest_version);

      message = response.message;
      messageType = 'success';
    } catch (error) {
      message = error instanceof Error ? error.message : 'Unbekannter Fehler';
      messageType = 'error';
    } finally {
      isInstalling = false;
    }
  }

  function parseChangelog(text: string): string {
    if (!text) return '';

    return text
      .replace(/### (.*)/g, '<h3 class="text-base font-semibold mt-3 mb-2">$1</h3>')
      .replace(/## (.*)/g, '<h2 class="text-lg font-bold mt-4 mb-2">$1</h2>')
      .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
      .replace(/\* (.*)/g, '<li class="ml-4 list-disc">$1</li>')
      .replace(/\n\n/g, '<br/><br/>')
      .replace(
        /\[([^\]]+)\]\(([^)]+)\)/g,
        '<a href="$2" class="text-blue-500 hover:underline" target="_blank" rel="noopener noreferrer">$1</a>'
      );
  }
</script>

<div class="space-y-6">
  <Card.Root>
    <Card.Header>
      <Card.Title>Software-Updates</Card.Title>
      <Card.Description>Überprüfen und installieren Sie CSF-Core Updates</Card.Description>
    </Card.Header>
    <Card.Content class="space-y-6">
      <!-- Current Version -->
      <div class="flex items-center justify-between p-4 border rounded-lg">
        <div>
          <p class="text-sm font-medium">Aktuelle Version</p>
          <div class="text-2xl font-bold text-primary">
            {#if $updateStore.versionInfo}
              v{$updateStore.versionInfo.current_version}
            {:else}
              <Skeleton class="h-8 w-24" />
            {/if}
          </div>
        </div>
        <Button onclick={checkForUpdates} disabled={isChecking} variant="outline" size="sm">
          <RefreshCw class={`mr-2 h-4 w-4 ${isChecking ? 'animate-spin' : ''}`} />
          {isChecking ? 'Prüfe...' : 'Nach Updates suchen'}
        </Button>
      </div>

      <!-- Messages -->
      {#if message}
        <Alert variant={messageType === 'error' ? 'destructive' : 'default'}>
          <AlertDescription>{message}</AlertDescription>
        </Alert>
      {/if}

      <!-- Update Status -->
      {#if $updateStore.loading}
        <div class="space-y-2">
          <Skeleton class="h-4 w-full" />
          <Skeleton class="h-4 w-3/4" />
        </div>
      {:else if $updateStore.error}
        <Alert variant="destructive">
          <AlertDescription>{$updateStore.error}</AlertDescription>
        </Alert>
      {:else if $updateStore.versionInfo}
        {#if $updateStore.versionInfo.update_available}
          <Alert>
            <Info class="h-4 w-4" />
            <AlertDescription>
              <div class="flex items-center justify-between">
                <div>
                  <p class="font-semibold mb-1">Neue Version verfügbar!</p>
                  <p class="text-sm">
                    Version <Badge variant="secondary"
                      >v{$updateStore.versionInfo.latest_version}</Badge
                    > ist jetzt verfügbar.
                  </p>
                </div>
                <Button onclick={installUpdate} disabled={isInstalling} size="sm">
                  {#if isInstalling}
                    <RefreshCw class="mr-2 h-4 w-4 animate-spin" />
                    Installiere...
                  {:else}
                    <Download class="mr-2 h-4 w-4" />
                    Jetzt installieren
                  {/if}
                </Button>
              </div>
            </AlertDescription>
          </Alert>

          <!-- Changelog -->
          {#if $updateStore.versionInfo.changelog}
            <Card.Root>
              <Card.Header>
                <Card.Title class="text-base">Was ist neu?</Card.Title>
              </Card.Header>
              <Card.Content>
                <div class="prose prose-sm dark:prose-invert max-w-none max-h-96 overflow-y-auto">
                  {@html parseChangelog($updateStore.versionInfo.changelog)}
                </div>
              </Card.Content>
              <Card.Footer>
                <Button
                  variant="outline"
                  href={$updateStore.versionInfo.release_url}
                  target="_blank"
                  size="sm"
                >
                  <ExternalLink class="mr-2 h-4 w-4" />
                  Vollständige Release-Notes auf GitHub
                </Button>
              </Card.Footer>
            </Card.Root>
          {/if}
        {:else}
          <Alert>
            <CheckCircle2 class="h-4 w-4" />
            <AlertDescription>
              <p class="font-semibold">Sie verwenden die neueste Version!</p>
              <p class="text-sm mt-1">
                CSF-Core ist auf dem neuesten Stand (v{$updateStore.versionInfo.current_version}).
              </p>
            </AlertDescription>
          </Alert>
        {/if}

        <!-- Last Checked -->
        {#if $updateStore.lastChecked}
          <p class="text-xs text-muted-foreground text-right">
            Zuletzt geprüft: {new Intl.DateTimeFormat('de-DE', {
              dateStyle: 'short',
              timeStyle: 'short',
            }).format($updateStore.lastChecked)}
          </p>
        {/if}
      {/if}

      <!-- Update Information -->
      <div class="pt-4 border-t">
        <h4 class="text-sm font-semibold mb-2">Automatische Update-Prüfung</h4>
        <p class="text-sm text-muted-foreground mb-3">
          CSF-Core prüft automatisch stündlich auf neue Updates. Updates werden nur angezeigt, wenn
          sie verfügbar sind.
        </p>
        <Alert>
          <Info class="h-4 w-4" />
          <AlertDescription class="text-xs">
            <strong>Hinweis:</strong> Bei der Installation eines Updates wird die Anwendung neu gestartet.
            Stellen Sie sicher, dass alle Änderungen gespeichert sind.
          </AlertDescription>
        </Alert>
      </div>
    </Card.Content>
  </Card.Root>
</div>
