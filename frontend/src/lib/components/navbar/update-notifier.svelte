<script lang="ts">
  import { onMount } from "svelte";
  import { Bell, Download, X } from "@lucide/svelte";
  import * as Alert from "$lib/components/ui/alert";
  import { Button } from "$lib/components/ui/button";

  interface VersionInfo {
    current_version: string;
    current_commit?: string;
    latest_version: string;
    latest_commit?: string;
    update_available: boolean;
    download_url?: string;
    release_notes?: string;
    published_at?: string;
  }

  let versionInfo: VersionInfo | null = $state(null);
  let loading = $state(true);
  let dismissed = $state(false);
  let checkInterval: number;

  async function checkForUpdates() {
    try {
      const response = await fetch("/api/system/version");
      if (response.ok) {
        versionInfo = await response.json();
      }
    } catch (error) {
      console.error("Failed to check for updates:", error);
    } finally {
      loading = false;
    }
  }

  function handleDismiss() {
    dismissed = true;
  }

  function handleUpdate() {
    if (versionInfo?.download_url) {
      // Öffne Download-Link
      window.open(versionInfo.download_url, "_blank");
    }
  }

  onMount(() => {
    // Initiales Check
    checkForUpdates();

    // Prüfe alle 30 Minuten
    checkInterval = setInterval(checkForUpdates, 30 * 60 * 1000);

    return () => {
      if (checkInterval) {
        clearInterval(checkInterval);
      }
    };
  });
</script>

{#if !loading && versionInfo?.update_available && !dismissed}
  <div class="px-2 py-2">
    <Alert.Root class="border-blue-500 bg-blue-50 dark:bg-blue-950">
      <div class="flex items-start gap-2">
        <Bell class="h-4 w-4 text-blue-600 dark:text-blue-400 mt-0.5" />
        <div class="flex-1">
          <Alert.Title
            class="text-sm font-semibold text-blue-900 dark:text-blue-100"
          >
            Update verfügbar
          </Alert.Title>
          <Alert.Description
            class="text-xs text-blue-800 dark:text-blue-200 mt-1"
          >
            Version {versionInfo.latest_version} ist verfügbar
            <br />
            <span class="text-blue-600 dark:text-blue-400">
              Aktuelle Version: {versionInfo.current_version}
            </span>
          </Alert.Description>

          <div class="flex gap-2 mt-3">
            <Button
              size="sm"
              variant="default"
              class="h-7 text-xs bg-blue-600 hover:bg-blue-700"
              onclick={handleUpdate}
            >
              <Download class="h-3 w-3 mr-1" />
              Herunterladen
            </Button>
            <Button
              size="sm"
              variant="ghost"
              class="h-7 text-xs"
              onclick={handleDismiss}
            >
              <X class="h-3 w-3 mr-1" />
              Später
            </Button>
          </div>
        </div>
      </div>
    </Alert.Root>
  </div>
{/if}
