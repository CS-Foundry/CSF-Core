<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { fade } from 'svelte/transition';
  import { Progress } from '$lib/components/ui/progress';
  import Logo from '/static/logos/CSF_Logo.png';

  let updateStatus = $state({
    status: 'idle',
    message: 'Initializing update...',
    progress: 0,
    version: '',
    timestamp: '',
  });

  let logs: string[] = $state([]);
  let logsContainer: HTMLDivElement;
  let pollInterval: ReturnType<typeof setInterval>;
  let connectionLost = $state(false);
  let reconnectAttempts = $state(0);
  let maxReconnectAttempts = 30; // Try for 30 seconds
  let previousStatus = $state('idle'); // Track previous status to detect transitions

  async function fetchUpdateStatus() {
    try {
      const response = await fetch('/api/updates/status');
      if (response.ok) {
        const data = await response.json();

        // Connection restored
        if (connectionLost) {
          connectionLost = false;
          reconnectAttempts = 0;
          logs = [...logs, `${new Date().toLocaleTimeString()}: ✅ Connection restored`];
        }

        // Only add to logs if message changed
        if (data.message !== updateStatus.message) {
          logs = [...logs, `${new Date().toLocaleTimeString()}: ${data.message}`];

          // Auto-scroll to bottom
          setTimeout(() => {
            if (logsContainer) {
              logsContainer.scrollTop = logsContainer.scrollHeight;
            }
          }, 10);
        }

        // Detect status transition from in_progress to idle (status file was deleted = update done)
        if (previousStatus === 'in_progress' && data.status === 'idle') {
          logs = [
            ...logs,
            `${new Date().toLocaleTimeString()}: ✅ Update completed! Status file cleaned up. Reloading...`,
          ];
          setTimeout(() => {
            window.location.reload();
          }, 2000);
          return; // Stop further processing
        }

        previousStatus = data.status;
        updateStatus = data;

        // If update is completed, reload after delay
        if (data.status === 'completed') {
          logs = [
            ...logs,
            `${new Date().toLocaleTimeString()}: ✅ Update completed successfully! Reloading...`,
          ];
          setTimeout(() => {
            window.location.reload();
          }, 2000);
        } else if (data.status === 'error') {
          // Stop polling on error
          if (pollInterval) {
            clearInterval(pollInterval);
          }
        }
      } else {
        handleConnectionError();
      }
    } catch (error) {
      handleConnectionError();
    }
  }

  function handleConnectionError() {
    if (!connectionLost) {
      connectionLost = true;
      logs = [
        ...logs,
        `${new Date().toLocaleTimeString()}: ⚠️ Backend restarting (this is normal during updates)...`,
      ];
    }

    reconnectAttempts++;

    // If backend doesn't come back after max attempts, the update might be complete
    if (reconnectAttempts >= maxReconnectAttempts) {
      logs = [
        ...logs,
        `${new Date().toLocaleTimeString()}: 🔄 Backend should be ready, reloading application...`,
      ];
      setTimeout(() => {
        window.location.reload();
      }, 2000);
    }
  }

  onMount(() => {
    // Set initial status to in_progress since we're showing the update screen
    previousStatus = 'in_progress';

    // Initial fetch
    fetchUpdateStatus();

    // Poll every second
    pollInterval = setInterval(fetchUpdateStatus, 1000);
  });

  onDestroy(() => {
    if (pollInterval) {
      clearInterval(pollInterval);
    }
  });

  function getStatusColor() {
    switch (updateStatus.status) {
      case 'completed':
        return 'text-green-500';
      case 'error':
        return 'text-red-500';
      case 'in_progress':
        return 'text-blue-500';
      default:
        return 'text-gray-500';
    }
  }

  function getStatusIcon() {
    switch (updateStatus.status) {
      case 'completed':
        return '✅';
      case 'error':
        return '❌';
      case 'in_progress':
        return '⏳';
      default:
        return '🔄';
    }
  }
</script>

<div
  class="fixed inset-0 z-[9999] flex items-center justify-center bg-background"
  transition:fade={{ duration: 300 }}
>
  <div class="w-full max-w-2xl px-8">
    <!-- Logo -->
    <div class="mb-12 flex justify-center">
      <img src={Logo} alt="CSF Logo" class="h-24 w-24" />
    </div>

    <!-- Title -->
    <h1 class="mb-2 text-center text-3xl font-bold">
      {getStatusIcon()} System-Update läuft
    </h1>

    <!-- Version Info -->
    {#if updateStatus.version}
      <p class="mb-8 text-center text-muted-foreground">
        Update auf Version <span class="font-semibold">{updateStatus.version}</span>
      </p>
    {/if}

    <!-- Progress Bar -->
    <div class="mb-8">
      <div class="mb-2 flex items-center justify-between">
        <span class="text-sm font-medium {getStatusColor()}">
          {updateStatus.message}
        </span>
        <span class="text-sm text-muted-foreground">
          {updateStatus.progress}%
        </span>
      </div>
      <Progress value={updateStatus.progress} class="h-3" />
    </div>

    <!-- Connection Status Warning -->
    {#if connectionLost}
      <div class="mb-6 rounded-lg border border-yellow-500 bg-yellow-500/10 p-4">
        <div class="flex items-center gap-2">
          <div class="h-2 w-2 animate-pulse rounded-full bg-yellow-500"></div>
          <p class="text-sm font-semibold text-yellow-600 dark:text-yellow-400">
            Backend wird neu gestartet...
          </p>
        </div>
        <p class="mt-2 text-xs text-muted-foreground">
          Dies ist normal während eines Updates. Die Verbindung wird automatisch wiederhergestellt.
        </p>
      </div>
    {/if}

    <!-- Status Message -->
    <div class="mb-6 rounded-lg border bg-card p-4">
      <h3 class="mb-2 text-sm font-semibold">Aktueller Status:</h3>
      <p class="text-sm text-muted-foreground {getStatusColor()}">
        {updateStatus.message}
      </p>
    </div>

    <!-- Logs -->
    <div class="rounded-lg border bg-card">
      <div class="border-b p-3">
        <h3 class="text-sm font-semibold">Update-Log</h3>
      </div>
      <div bind:this={logsContainer} class="max-h-64 overflow-y-auto p-3 font-mono text-xs">
        {#if logs.length === 0}
          <p class="text-muted-foreground">Warte auf Update-Logs...</p>
        {:else}
          {#each logs as log}
            <div class="mb-1 text-muted-foreground">
              {log}
            </div>
          {/each}
        {/if}
      </div>
    </div>

    <!-- Warning -->
    <div class="mt-6 text-center text-sm text-muted-foreground">
      <p>⚠️ Bitte schließen Sie dieses Fenster nicht und laden Sie die Seite nicht neu.</p>
      <p>Das System wird automatisch neu geladen, sobald das Update abgeschlossen ist.</p>
    </div>

    <!-- Error Recovery -->
    {#if updateStatus.status === 'error'}
      <div class="mt-6 rounded-lg border border-destructive bg-destructive/10 p-4">
        <p class="mb-2 text-sm font-semibold text-destructive">Update fehlgeschlagen</p>
        <p class="mb-4 text-sm text-muted-foreground">
          Während des Update-Prozesses ist ein Fehler aufgetreten. Das System wurde möglicherweise
          auf die vorherige Version zurückgesetzt.
        </p>
        <button
          onclick={() => window.location.reload()}
          class="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
        >
          Anwendung neu laden
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  /* Smooth scrolling for logs */
  div::-webkit-scrollbar {
    width: 8px;
  }

  div::-webkit-scrollbar-track {
    background: transparent;
  }

  div::-webkit-scrollbar-thumb {
    background: hsl(var(--muted));
    border-radius: 4px;
  }

  div::-webkit-scrollbar-thumb:hover {
    background: hsl(var(--muted-foreground));
  }
</style>
