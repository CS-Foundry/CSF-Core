<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import {
    getResource,
    updateResource,
    deleteResource,
  } from "$lib/services/resources";
  import type { Resource } from "$lib/types/resource";
  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Badge } from "$lib/components/ui/badge";
  import {
    ArrowLeft,
    Server,
    Package,
    Play,
    Square,
    RefreshCw,
    Trash2,
    Edit,
  } from "@lucide/svelte";

  let resource: Resource | null = null;
  let loading = true;
  let error = "";
  let showDeleteDialog = false;

  $: resourceId = $page.params.id;

  onMount(async () => {
    await loadResource();
  });

  async function loadResource() {
    try {
      loading = true;
      error = "";
      resource = await getResource(resourceId);
    } catch (e: any) {
      error = e.message || "Failed to load resource";
    } finally {
      loading = false;
    }
  }

  function getStatusBadgeVariant(status: string) {
    switch (status) {
      case "running":
        return "default";
      case "stopped":
        return "secondary";
      case "error":
        return "destructive";
      default:
        return "outline";
    }
  }

  async function handleDelete() {
    if (!resource) return;

    try {
      error = "";
      await deleteResource(resource.id);
      goto("/resources");
    } catch (e: any) {
      error = e.message || "Failed to delete resource";
      showDeleteDialog = false;
    }
  }

  function formatJson(obj: any): string {
    return JSON.stringify(obj, null, 2);
  }
</script>

<div class="container mx-auto p-6 max-w-5xl">
  <div class="mb-6">
    <Button variant="ghost" onclick={() => goto("/resources")}>
      <ArrowLeft class="h-4 w-4 mr-2" />
      Zurück zu Ressourcen
    </Button>
  </div>

  {#if error}
    <div
      class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded mb-4"
    >
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
              {#if resource.resource_type === "docker-stack"}
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
                variant="destructive"
                onclick={() => (showDeleteDialog = true)}
              >
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
                {new Date(resource.created_at).toLocaleString("de-DE")}
              </p>
            </div>
          </div>
        </Card.Content>
      </Card.Root>

      <!-- Configuration -->
      {#if resource.configuration}
        <Card.Root>
          <Card.Header>
            <Card.Title>Konfiguration</Card.Title>
          </Card.Header>
          <Card.Content>
            {#if resource.resource_type === "docker-stack" && resource.configuration.services}
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
                              {port.host || "auto"}:{port.container}
                            </Badge>
                          {/each}
                        </div>
                      </div>
                    {/if}

                    {#if service.environment}
                      <div class="mt-2">
                        <p class="text-sm text-gray-500 mb-1">
                          Environment Variables:
                        </p>
                        <div class="bg-gray-50 rounded p-2 text-xs">
                          {#each Object.entries(service.environment) as [key, value]}
                            <div class="font-mono">
                              <span class="text-purple-600">{key}</span> =
                              <span class="text-gray-700">{value}</span>
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
                            <Badge variant="secondary" class="text-xs"
                              >{dep}</Badge
                            >
                          {/each}
                        </div>
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            {:else if resource.resource_type === "docker-container"}
              <div class="space-y-3">
                <div>
                  <p class="text-sm text-gray-500">Image</p>
                  <code class="bg-gray-100 px-2 py-1 rounded"
                    >{resource.configuration.image}</code
                  >
                </div>

                {#if resource.configuration.ports}
                  <div>
                    <p class="text-sm text-gray-500 mb-1">Ports</p>
                    <div class="flex flex-wrap gap-2">
                      {#each resource.configuration.ports as port}
                        <Badge variant="outline">
                          {port.host || "auto"}:{port.container}
                        </Badge>
                      {/each}
                    </div>
                  </div>
                {/if}

                {#if resource.configuration.environment}
                  <div>
                    <p class="text-sm text-gray-500 mb-1">
                      Environment Variables
                    </p>
                    <div class="bg-gray-50 rounded p-2 text-xs font-mono">
                      {#each Object.entries(resource.configuration.environment) as [key, value]}
                        <div>
                          <span class="text-purple-600">{key}</span> =
                          <span>{value}</span>
                        </div>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>
            {:else}
              <pre
                class="bg-gray-50 p-4 rounded text-xs overflow-auto">{formatJson(
                  resource.configuration
                )}</pre>
            {/if}
          </Card.Content>
        </Card.Root>
      {/if}

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
                <dd class="font-mono text-xs mt-1">{resource.container_id}</dd>
              </div>
            {/if}
            {#if resource.stack_name}
              <div>
                <dt class="text-sm text-gray-500">Stack Name</dt>
                <dd class="font-mono text-xs mt-1">{resource.stack_name}</dd>
              </div>
            {/if}
            <div>
              <dt class="text-sm text-gray-500">Zuletzt aktualisiert</dt>
              <dd class="font-medium mt-1">
                {new Date(resource.updated_at).toLocaleString("de-DE")}
              </dd>
            </div>
          </dl>
        </Card.Content>
      </Card.Root>
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
        Möchten Sie die Ressource "{resource?.name}" wirklich löschen? Diese
        Aktion kann nicht rückgängig gemacht werden.
      </Dialog.Description>
    </Dialog.Header>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (showDeleteDialog = false)}>
        Abbrechen
      </Button>
      <Button variant="destructive" onclick={handleDelete}>Löschen</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
