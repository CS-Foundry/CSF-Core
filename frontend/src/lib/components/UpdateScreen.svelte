<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { fade } from 'svelte/transition';
  import { Progress } from '$lib/components/ui/progress';
  import Logo from '$lib/assets/logo.svg';

  let updateStatus = $state({
    status: 'idle',
    message: 'Initializing update...',
    progress: 0,
    version: '',
    timestamp: '',
  });

  let logs: string[] = $state([]);
  let logsContainer: HTMLDivElement;
  let pollInterval: number;

  async function fetchUpdateStatus() {
    try {
      const response = await fetch('/api/updates/status');
      if (response.ok) {
        const data = await response.json();

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

        updateStatus = data;

        // If update is completed or error, stop polling
        if (data.status === 'completed') {
          setTimeout(() => {
            // Reload the page after 2 seconds
            window.location.reload();
          }, 2000);
        } else if (data.status === 'error') {
          // Stop polling on error
          if (pollInterval) {
            clearInterval(pollInterval);
          }
        }
      }
    } catch (error) {
      console.error('Failed to fetch update status:', error);
    }
  }

  onMount(() => {
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
      {getStatusIcon()} System Update in Progress
    </h1>

    <!-- Version Info -->
    {#if updateStatus.version}
      <p class="mb-8 text-center text-muted-foreground">
        Updating to version <span class="font-semibold">{updateStatus.version}</span>
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

    <!-- Status Message -->
    <div class="mb-6 rounded-lg border bg-card p-4">
      <h3 class="mb-2 text-sm font-semibold">Current Status:</h3>
      <p class="text-sm text-muted-foreground {getStatusColor()}">
        {updateStatus.message}
      </p>
    </div>

    <!-- Logs -->
    <div class="rounded-lg border bg-card">
      <div class="border-b p-3">
        <h3 class="text-sm font-semibold">Update Log</h3>
      </div>
      <div bind:this={logsContainer} class="max-h-64 overflow-y-auto p-3 font-mono text-xs">
        {#if logs.length === 0}
          <p class="text-muted-foreground">Waiting for update logs...</p>
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
      <p>⚠️ Please do not close this window or refresh the page.</p>
      <p>The system will automatically reload when the update is complete.</p>
    </div>

    <!-- Error Recovery -->
    {#if updateStatus.status === 'error'}
      <div class="mt-6 rounded-lg border border-destructive bg-destructive/10 p-4">
        <p class="mb-2 text-sm font-semibold text-destructive">Update Failed</p>
        <p class="mb-4 text-sm text-muted-foreground">
          An error occurred during the update process. The system may have been rolled back to the
          previous version.
        </p>
        <button
          onclick={() => window.location.reload()}
          class="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
        >
          Reload Application
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
