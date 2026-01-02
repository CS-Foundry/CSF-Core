<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import {
    getResource,
    updateResource,
    deleteResource,
    performResourceAction,
    getResourceLogs,
    execCommand as execCommandInResource,
  } from '$lib/services/resources';
  import type { Resource } from '$lib/types/resource';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as Tabs from '$lib/components/ui/tabs';
  import { Badge } from '$lib/components/ui/badge';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Textarea } from '$lib/components/ui/textarea';
  import {
    ArrowLeft,
    Server,
    Package,
    Play,
    Square,
    RefreshCw,
    Trash2,
    Edit,
    Save,
    X,
    Info,
    Settings,
    ScrollText,
    Terminal,
  } from '@lucide/svelte';

  let resource: Resource | null = null;
  let loading = true;
  let error = '';
  let showDeleteDialog = false;
  let showEditDialog = false;
  let actionInProgress = false;
  let containerLogs = '';
  let loadingLogs = false;
  let execCommand = '';
  let execOutput = '';
  let execRunning = false;
  let activeTab = 'info';
  let logsLoaded = false;
  let logsInterval: ReturnType<typeof setInterval> | null = null;
  let logsContainer: HTMLDivElement;

  // Edit form state
  let editName = '';
  let editDescription = '';
  let editImage = '';
  let editPorts = '';
  let editEnvironment = '';

  $: resourceId = $page.params.id;

  // Auto-load logs when switching to logs tab
  $: if (activeTab === 'logs' && resource?.container_id && !logsLoaded) {
    loadLogs();
  }

  // Auto-refresh logs every 2 seconds when in logs tab
  $: {
    if (activeTab === 'logs' && resource?.container_id) {
      if (!logsInterval) {
        logsInterval = setInterval(() => {
          loadLogs();
        }, 2000);
      }
    } else {
      if (logsInterval) {
        clearInterval(logsInterval);
        logsInterval = null;
      }
    }
  }

  onDestroy(() => {
    if (logsInterval) {
      clearInterval(logsInterval);
    }
  });

  async function loadLogs() {
    if (!resource?.container_id || loadingLogs) return;

    loadingLogs = true;
    try {
      const response = await getResourceLogs(resource.id);
      containerLogs = response.logs || 'Keine Logs verfügbar';
      logsLoaded = true;

      // Auto-scroll to bottom after logs update
      setTimeout(() => {
        if (logsContainer) {
          logsContainer.scrollTop = logsContainer.scrollHeight;
        }
      }, 100);
    } catch (e: any) {
      error = e.message || 'Fehler beim Laden der Logs';
      containerLogs = `Fehler: ${error}`;
    } finally {
      loadingLogs = false;
    }
  }

  onMount(async () => {
    await loadResource();
  });

  async function loadResource() {
    if (!resourceId) return;

    try {
      loading = true;
      error = '';
      resource = await getResource(resourceId);
    } catch (e: any) {
      error = e.message || 'Failed to load resource';
    } finally {
      loading = false;
    }
  }

  async function handleAction(action: 'start' | 'stop' | 'restart') {
    if (!resource) return;

    try {
      actionInProgress = true;
      error = '';
      resource = await performResourceAction(resource.id, action);
    } catch (e: any) {
      error = e.message || `Failed to ${action} resource`;
    } finally {
      actionInProgress = false;
    }
  }

  function openEditDialog() {
    if (!resource) return;

    editName = resource.name;
    editDescription = resource.description || '';

    if (resource.resource_type === 'docker-container' && resource.configuration) {
      editImage = resource.configuration.image || '';
      editPorts = JSON.stringify(resource.configuration.ports || [], null, 2);
      editEnvironment = JSON.stringify(resource.configuration.environment || {}, null, 2);
    }

    showEditDialog = true;
  }

  async function handleSaveEdit() {
    if (!resource) return;

    try {
      error = '';
      let configuration = resource.configuration;

      // Update configuration for docker containers
      if (resource.resource_type === 'docker-container') {
        try {
          configuration = {
            ...configuration,
            image: editImage,
            ports: editPorts ? JSON.parse(editPorts) : [],
            environment: editEnvironment ? JSON.parse(editEnvironment) : {},
          };
        } catch (e) {
          error = 'Ungültiges JSON-Format in Ports oder Environment';
          return;
        }
      }

      resource = await updateResource(resource.id, {
        name: editName,
        description: editDescription || undefined,
        configuration,
      });

      showEditDialog = false;
    } catch (e: any) {
      error = e.message || 'Failed to update resource';
    }
  }

  function getStatusBadgeVariant(status: string) {
    switch (status) {
      case 'running':
        return 'default';
      case 'stopped':
        return 'secondary';
      case 'error':
        return 'destructive';
      default:
        return 'outline';
    }
  }

  async function handleDelete() {
    if (!resource) return;

    try {
      error = '';
      await deleteResource(resource.id);
      goto('/resources');
    } catch (e: any) {
      error = e.message || 'Failed to delete resource';
      showDeleteDialog = false;
    }
  }

  function formatJson(obj: any): string {
    return JSON.stringify(obj, null, 2);
  }
</script>

<div class="container mx-auto p-6 max-w-5xl">
  <div class="mb-6">
    <Button variant="ghost" onclick={() => goto('/resources')}>
      <ArrowLeft class="h-4 w-4 mr-2" />
      Zurück zu Ressourcen
    </Button>
  </div>

  {#if error}
    <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded mb-4">
      {error}
    </div>
  {/if}

  {#if loading}
    <Card.Root>
      <Card.Content class="py-12 text-center">
        <p class="text-gray-500">Laden...</p>
      </Card.Content>
    </Card.Root>
  {:else if resource}
    <div class="space-y-6">
      <!-- Header -->
      <Card.Root>
        <Card.Header>
          <div class="flex items-start justify-between">
            <div class="flex items-center gap-4">
              {#if resource.resource_type === 'docker-stack'}
                <Package class="h-12 w-12 text-blue-600" />
              {:else}
                <Server class="h-12 w-12 text-blue-600" />
              {/if}
              <div>
                <Card.Title class="text-3xl">{resource.name}</Card.Title>
                <div class="flex items-center gap-2 mt-2">
                  <Badge variant="outline">{resource.resource_type}</Badge>
                  <Badge variant={getStatusBadgeVariant(resource.status)}>
                    {resource.status}
                  </Badge>
                </div>
              </div>
            </div>
            <div class="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onclick={() => handleAction('start')}
                disabled={actionInProgress || resource.status === 'running'}
              >
                <Play class="h-4 w-4 mr-2" />
                Starten
              </Button>
              <Button
                variant="outline"
                size="sm"
                onclick={() => handleAction('stop')}
                disabled={actionInProgress || resource.status === 'stopped'}
              >
                <Square class="h-4 w-4 mr-2" />
                Stoppen
              </Button>
              <Button
                variant="outline"
                size="sm"
                onclick={() => handleAction('restart')}
                disabled={actionInProgress}
              >
                <RefreshCw class="h-4 w-4 mr-2" />
                Neustarten
              </Button>
              <Button variant="outline" size="sm" onclick={openEditDialog}>
                <Edit class="h-4 w-4 mr-2" />
                Bearbeiten
              </Button>
              <Button variant="destructive" size="sm" onclick={() => (showDeleteDialog = true)}>
                <Trash2 class="h-4 w-4 mr-2" />
                Löschen
              </Button>
            </div>
          </div>
        </Card.Header>
        <Card.Content>
          {#if resource.description}
            <p class="text-gray-600">{resource.description}</p>
          {/if}
          <div class="grid grid-cols-2 gap-4 mt-4">
            <div>
              <p class="text-sm text-gray-500">Resource Group</p>
              <a
                href="/resource-groups/{resource.resource_group_id}"
                class="text-blue-600 hover:underline font-medium"
              >
                {resource.resource_group_name}
              </a>
            </div>
            <div>
              <p class="text-sm text-gray-500">Erstellt am</p>
              <p class="font-medium">
                {new Date(resource.created_at).toLocaleString('de-DE')}
              </p>
            </div>
          </div>
        </Card.Content>
      </Card.Root>

      <!-- Tabs Section -->
      <Tabs.Root bind:value={activeTab} class="w-full">
        <Tabs.List class="grid w-full grid-cols-4">
          <Tabs.Trigger value="info">
            <Info class="h-4 w-4 mr-2" />
            Informationen
          </Tabs.Trigger>
          <Tabs.Trigger value="config">
            <Settings class="h-4 w-4 mr-2" />
            Konfiguration
          </Tabs.Trigger>
          <Tabs.Trigger value="logs">
            <ScrollText class="h-4 w-4 mr-2" />
            Logs
          </Tabs.Trigger>
          <Tabs.Trigger value="exec">
            <Terminal class="h-4 w-4 mr-2" />
            Exec
          </Tabs.Trigger>
        </Tabs.List>

        <!-- Info Tab -->
        <Tabs.Content value="info" class="space-y-4 mt-4">
          <!-- Tags -->
          {#if resource.tags}
            <Card.Root>
              <Card.Header>
                <Card.Title>Tags</Card.Title>
              </Card.Header>
              <Card.Content>
                <div class="flex flex-wrap gap-2">
                  {#each Object.entries(resource.tags) as [key, value]}
                    <Badge variant="outline">
                      <span class="text-gray-500">{key}:</span>
                      {value}
                    </Badge>
                  {/each}
                </div>
              </Card.Content>
            </Card.Root>
          {/if}

          <!-- Technical Details -->
          <Card.Root>
            <Card.Header>
              <Card.Title>Technische Details</Card.Title>
            </Card.Header>
            <Card.Content>
              <dl class="grid grid-cols-2 gap-4">
                <div>
                  <dt class="text-sm text-gray-500">Resource ID</dt>
                  <dd class="font-mono text-xs mt-1">{resource.id}</dd>
                </div>
                {#if resource.container_id}
                  <div>
                    <dt class="text-sm text-gray-500">Container ID</dt>
                    <dd class="font-mono text-xs mt-1">
                      {resource.container_id}
                    </dd>
                  </div>
                {/if}
                {#if resource.stack_name}
                  <div>
                    <dt class="text-sm text-gray-500">Stack Name</dt>
                    <dd class="font-mono text-xs mt-1">
                      {resource.stack_name}
                    </dd>
                  </div>
                {/if}
                <div>
                  <dt class="text-sm text-gray-500">Zuletzt aktualisiert</dt>
                  <dd class="font-medium mt-1">
                    {new Date(resource.updated_at).toLocaleString('de-DE')}
                  </dd>
                </div>
              </dl>
            </Card.Content>
          </Card.Root>
        </Tabs.Content>

        <!-- Configuration Tab -->
        <Tabs.Content value="config" class="mt-4">
          <Card.Root>
            <Card.Header>
              <Card.Title>Konfiguration</Card.Title>
            </Card.Header>
            <Card.Content>
              {#if resource.configuration}
                {#if resource.resource_type === 'docker-stack' && resource.configuration.services}
                  <div class="space-y-4">
                    <h3 class="font-semibold">
                      Services ({resource.configuration.services.length})
                    </h3>
                    {#each resource.configuration.services as service}
                      <div class="border rounded-lg p-4">
                        <div class="flex items-center justify-between mb-2">
                          <h4 class="font-medium text-lg">{service.name}</h4>
                          <Badge variant="secondary">{service.image}</Badge>
                        </div>

                        {#if service.ports && service.ports.length > 0}
                          <div class="mt-2">
                            <p class="text-sm text-gray-500 mb-1">Ports:</p>
                            <div class="flex flex-wrap gap-2">
                              {#each service.ports as port}
                                <Badge variant="outline">
                                  {port.host || 'auto'}:{port.container}
                                </Badge>
                              {/each}
                            </div>
                          </div>
                        {/if}

                        {#if service.environment}
                          <div class="mt-2">
                            <p class="text-sm text-gray-500 mb-1">Environment Variables:</p>
                            <div class="space-y-1">
                              {#each Object.entries(service.environment) as [key, value]}
                                <div class="font-mono text-xs">
                                  <span class="text-purple-600 font-semibold">{key}</span>
                                  <span class="text-gray-500"> = </span>
                                  <span class="text-gray-900">{value}</span>
                                </div>
                              {/each}
                            </div>
                          </div>
                        {/if}

                        {#if service.volumes && service.volumes.length > 0}
                          <div class="mt-2">
                            <p class="text-sm text-gray-500 mb-1">Volumes:</p>
                            <div class="space-y-1">
                              {#each service.volumes as volume}
                                <Badge variant="outline" class="font-mono text-xs">
                                  {volume.host} → {volume.container}
                                </Badge>
                              {/each}
                            </div>
                          </div>
                        {/if}

                        {#if service.depends_on && service.depends_on.length > 0}
                          <div class="mt-2">
                            <p class="text-sm text-gray-500 mb-1">Depends On:</p>
                            <div class="flex flex-wrap gap-1">
                              {#each service.depends_on as dep}
                                <Badge variant="secondary" class="text-xs">{dep}</Badge>
                              {/each}
                            </div>
                          </div>
                        {/if}
                      </div>
                    {/each}
                  </div>
                {:else if resource.resource_type === 'docker-container'}
                  <div class="space-y-4">
                    <div>
                      <p class="text-sm font-medium text-gray-700 mb-2">Image</p>
                      <p class="font-mono text-sm">
                        {resource.configuration.image}
                      </p>
                    </div>

                    {#if resource.configuration.ports && resource.configuration.ports.length > 0}
                      <div>
                        <p class="text-sm font-medium text-gray-700 mb-2">Ports</p>
                        <div class="flex flex-wrap gap-2">
                          {#each resource.configuration.ports as port}
                            <Badge variant="outline">
                              {port.host || 'auto'}:{port.container}
                            </Badge>
                          {/each}
                        </div>
                      </div>
                    {/if}

                    {#if resource.configuration.environment && Object.keys(resource.configuration.environment).length > 0}
                      <div>
                        <p class="text-sm font-medium text-gray-700 mb-2">Environment Variables</p>
                        <div class="space-y-1">
                          {#each Object.entries(resource.configuration.environment) as [key, value]}
                            <div class="font-mono text-sm">
                              <span class="text-purple-600 font-semibold">{key}</span>
                              <span class="text-gray-500"> = </span>
                              <span class="text-gray-900">{value}</span>
                            </div>
                          {/each}
                        </div>
                      </div>
                    {/if}

                    {#if resource.configuration.volumes && resource.configuration.volumes.length > 0}
                      <div>
                        <p class="text-sm font-medium text-gray-700 mb-2">Volumes</p>
                        <div class="space-y-1">
                          {#each resource.configuration.volumes as volume}
                            <div class="font-mono text-sm">
                              <span>{volume.host}</span>
                              <span class="text-gray-500"> → </span>
                              <span>{volume.container}</span>
                            </div>
                          {/each}
                        </div>
                      </div>
                    {/if}
                  </div>
                {:else}
                  <pre class="border rounded p-4 text-xs overflow-auto font-mono">{formatJson(
                      resource.configuration
                    )}</pre>
                {/if}
              {:else}
                <p class="text-sm text-gray-500">Keine Konfiguration vorhanden</p>
              {/if}
            </Card.Content>
          </Card.Root>
        </Tabs.Content>

        <!-- Logs Tab -->
        <Tabs.Content value="logs" class="mt-4">
          <Card.Root>
            <Card.Header>
              <div class="flex items-center justify-between">
                <Card.Title>Container Logs</Card.Title>
                <Button
                  variant="outline"
                  size="sm"
                  onclick={async () => {
                    if (!resource?.container_id) return;
                    logsLoaded = false;
                    await loadLogs();
                  }}
                  disabled={loadingLogs || !resource?.container_id}
                >
                  <RefreshCw class="h-4 w-4 mr-2 {loadingLogs ? 'animate-spin' : ''}" />
                  {loadingLogs ? 'Laden...' : 'Aktualisieren'}
                </Button>
              </div>
            </Card.Header>
            <Card.Content>
              {#if !resource?.container_id}
                <p class="text-sm text-gray-500">
                  Keine Container ID vorhanden. Logs sind nur für laufende Container verfügbar.
                </p>
              {:else}
                <div
                  bind:this={logsContainer}
                  class="bg-black text-green-400 p-4 rounded font-mono text-xs max-h-96 whitespace-pre-wrap overflow-y-auto scrollbar-hide"
                  style="scroll-behavior: smooth;"
                >
                  {containerLogs || 'Logs werden automatisch geladen...'}
                </div>
              {/if}
            </Card.Content>
          </Card.Root>
        </Tabs.Content>

        <!-- Exec Tab -->
        <Tabs.Content value="exec" class="mt-4">
          <Card.Root>
            <Card.Header>
              <Card.Title>Container Exec</Card.Title>
              <Card.Description>
                Führe Befehle im Container aus. Beispiel: ls -la, ps aux, env
              </Card.Description>
            </Card.Header>
            <Card.Content class="space-y-4">
              {#if !resource?.container_id}
                <p class="text-sm text-gray-500">
                  Keine Container ID vorhanden. Exec ist nur für laufende Container verfügbar.
                </p>
              {:else}
                <div class="space-y-2">
                  <Label for="exec-command">Befehl</Label>
                  <div class="flex gap-2">
                    <Input
                      id="exec-command"
                      bind:value={execCommand}
                      placeholder="z.B. ls -la"
                      class="font-mono"
                      disabled={execRunning}
                      onkeydown={async (e) => {
                        if (e.key === 'Enter' && execCommand && !execRunning && resource) {
                          e.preventDefault();
                          execRunning = true;
                          try {
                            const response = await execCommandInResource(resource.id, execCommand);
                            execOutput = `$ ${execCommand}\n${response.output || '(kein Output)'}`;
                          } catch (err: any) {
                            execOutput = `$ ${execCommand}\nFehler: ${err.message || 'Unbekannter Fehler'}`;
                          } finally {
                            execRunning = false;
                          }
                        }
                      }}
                    />
                    <Button
                      onclick={async () => {
                        if (!execCommand || execRunning || !resource) return;
                        execRunning = true;
                        try {
                          const response = await execCommandInResource(resource.id, execCommand);
                          execOutput = `$ ${execCommand}\n${response.output || '(kein Output)'}`;
                        } catch (err: any) {
                          execOutput = `$ ${execCommand}\nFehler: ${err.message || 'Unbekannter Fehler'}`;
                        } finally {
                          execRunning = false;
                        }
                      }}
                      disabled={!execCommand || execRunning}
                    >
                      <Terminal class="h-4 w-4 mr-2" />
                      {execRunning ? 'Ausführen...' : 'Ausführen'}
                    </Button>
                  </div>
                </div>

                {#if execOutput}
                  <div>
                    <Label>Output</Label>
                    <div
                      class="bg-black text-green-400 p-4 rounded font-mono text-xs overflow-auto max-h-96 mt-2 whitespace-pre-wrap"
                    >
                      {execOutput}
                    </div>
                  </div>
                {/if}

                <div class="bg-blue-50 border border-blue-200 rounded p-3">
                  <p class="text-sm text-blue-800">
                    <strong>Tipp:</strong> Verwende einfache Shell-Befehle wie
                    <code class="bg-blue-100 px-1 rounded">ls</code>,
                    <code class="bg-blue-100 px-1 rounded">pwd</code>,
                    <code class="bg-blue-100 px-1 rounded">env</code> oder
                    <code class="bg-blue-100 px-1 rounded">ps aux</code>.
                  </p>
                </div>
              {/if}
            </Card.Content>
          </Card.Root>
        </Tabs.Content>
      </Tabs.Root>
    </div>
  {:else}
    <Card.Root>
      <Card.Content class="py-12 text-center">
        <p class="text-gray-500">Ressource nicht gefunden</p>
      </Card.Content>
    </Card.Root>
  {/if}
</div>

<!-- Delete Confirmation Dialog -->
<Dialog.Root bind:open={showDeleteDialog}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Ressource löschen</Dialog.Title>
      <Dialog.Description>
        Möchten Sie die Ressource "{resource?.name}" wirklich löschen? Diese Aktion kann nicht
        rückgängig gemacht werden.
      </Dialog.Description>
    </Dialog.Header>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (showDeleteDialog = false)}>Abbrechen</Button>
      <Button variant="destructive" onclick={handleDelete}>Löschen</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<!-- Edit Resource Dialog -->
<Dialog.Root bind:open={showEditDialog}>
  <Dialog.Content class="max-w-2xl max-h-[90vh] overflow-y-auto">
    <Dialog.Header>
      <Dialog.Title>Ressource bearbeiten</Dialog.Title>
      <Dialog.Description>
        Ändern Sie die Konfiguration der Ressource "{resource?.name}".
      </Dialog.Description>
    </Dialog.Header>

    <div class="space-y-4 py-4">
      <div class="space-y-2">
        <Label for="edit-name">Name</Label>
        <Input id="edit-name" bind:value={editName} placeholder="Ressource Name" />
      </div>

      <div class="space-y-2">
        <Label for="edit-description">Beschreibung</Label>
        <textarea
          id="edit-description"
          bind:value={editDescription}
          placeholder="Optional: Beschreibung der Ressource"
          rows={3}
          class="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
        ></textarea>
      </div>

      {#if resource?.resource_type === 'docker-container'}
        <div class="space-y-2">
          <Label for="edit-image">Docker Image</Label>
          <Input id="edit-image" bind:value={editImage} placeholder="z.B. nginx:latest" />
        </div>

        <div class="space-y-2">
          <Label for="edit-ports">Ports (JSON)</Label>
          <textarea
            id="edit-ports"
            bind:value={editPorts}
            placeholder={'[{"container": 80, "host": 8080}]'}
            rows={4}
            class="flex min-h-[100px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
          ></textarea>
          <p class="text-xs text-gray-500">
            Format: [{'{'}container: 80, host: 8080{'}'}]
          </p>
        </div>

        <div class="space-y-2">
          <Label for="edit-environment">Environment Variables (JSON)</Label>
          <textarea
            id="edit-environment"
            bind:value={editEnvironment}
            placeholder={'{"KEY": "value"}'}
            rows={6}
            class="flex min-h-[150px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
          ></textarea>
          <p class="text-xs text-gray-500">
            Format: {'{'}KEY: "value"{'}'}
          </p>
        </div>
      {/if}
    </div>

    <Dialog.Footer>
      <Button variant="outline" onclick={() => (showEditDialog = false)}>
        <X class="h-4 w-4 mr-2" />
        Abbrechen
      </Button>
      <Button onclick={handleSaveEdit}>
        <Save class="h-4 w-4 mr-2" />
        Speichern
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
