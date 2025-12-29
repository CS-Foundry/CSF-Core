<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import {
    listResources,
    deleteResource,
    deployContainer,
    type DeployContainerRequest,
  } from "$lib/services/resources";
  import { listResourceGroups } from "$lib/services/resource-groups";
  import type { Resource } from "$lib/types/resource";
  import type { ResourceGroup } from "$lib/types/resource-group";
  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import * as Table from "$lib/components/ui/table";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Badge } from "$lib/components/ui/badge";
  import DeployDockerContainerDialog from "$lib/components/DeployDockerContainerDialog.svelte";
  import {
    Plus,
    Server,
    Package,
    Trash2,
    Play,
    Square,
    RefreshCw,
    Rocket,
  } from "@lucide/svelte";

  let resources: Resource[] = [];
  let resourceGroups: ResourceGroup[] = [];
  let loading = true;
  let error = "";

  // Delete confirmation dialog
  let showDeleteDialog = false;
  let resourceToDelete: Resource | null = null;

  // Deploy dialog
  let showDeployDialog = false;

  onMount(async () => {
    await Promise.all([loadResources(), loadResourceGroups()]);
  });

  async function loadResources() {
    try {
      loading = true;
      error = "";
      resources = await listResources();
    } catch (e: any) {
      error = e.message || "Failed to load resources";
    } finally {
      loading = false;
    }
  }

  async function loadResourceGroups() {
    try {
      resourceGroups = await listResourceGroups();
    } catch (e: any) {
      console.error("Failed to load resource groups:", e);
    }
  }

  async function handleDeploy(event: CustomEvent<DeployContainerRequest>) {
    try {
      error = "";
      await deployContainer(event.detail);
      showDeployDialog = false;
      await loadResources();
    } catch (e: any) {
      error = e.message || "Failed to deploy container";
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

  function getResourceIcon(resourceType: string) {
    if (resourceType === "docker-stack") {
      return Package;
    }
    return Server;
  }

  function openDeleteDialog(resource: Resource) {
    resourceToDelete = resource;
    showDeleteDialog = true;
  }

  async function handleDelete() {
    if (!resourceToDelete) return;

    try {
      error = "";
      await deleteResource(resourceToDelete.id);
      showDeleteDialog = false;
      resourceToDelete = null;
      await loadResources();
    } catch (e: any) {
      error = e.message || "Failed to delete resource";
    }
  }

  $: dockerContainers = resources.filter(
    (r) => r.resource_type === "docker-container"
  );
  $: dockerStacks = resources.filter((r) => r.resource_type === "docker-stack");
</script>

<div class="container mx-auto p-6">
  <div class="flex items-center justify-between mb-6">
    <div>
      <h1 class="text-3xl font-bold flex items-center gap-2">
        <Server class="h-8 w-8" />
        Ressourcen
      </h1>
      <p class="text-muted-foreground mt-2">
        Verwalte deine Docker Container und Stacks
      </p>
    </div>
    <div class="flex gap-2">
      <Button variant="outline" onclick={loadResources}>
        <RefreshCw class="h-4 w-4 mr-2" />
        Aktualisieren
      </Button>
      <Button variant="default" onclick={() => (showDeployDialog = true)}>
        <Rocket class="h-4 w-4 mr-2" />
        Container bereitstellen
      </Button>
      <Button onclick={() => goto("/marketplace")}>
        <Plus class="h-4 w-4 mr-2" />
        Aus Marketplace
      </Button>
    </div>
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
  {:else if resources.length === 0}
    <Card.Root>
      <Card.Content class="py-12 text-center">
        <Server class="h-16 w-16 mx-auto mb-4 text-gray-300" />
        <h3 class="text-lg font-semibold mb-2">Keine Ressourcen vorhanden</h3>
        <p class="text-gray-500 mb-4">
          Beginne damit, Docker Container oder Stacks aus dem Marketplace
          hinzuzufügen
        </p>
        <Button onclick={() => goto("/marketplace")}>
          <Plus class="h-4 w-4 mr-2" />
          Zum Marketplace
        </Button>
      </Card.Content>
    </Card.Root>
  {:else}
    <div class="space-y-6">
      <!-- Docker Stacks -->
      {#if dockerStacks.length > 0}
        <div>
          <h2 class="text-2xl font-semibold mb-4 flex items-center gap-2">
            <Package class="h-6 w-6" />
            Docker Stacks ({dockerStacks.length})
          </h2>
          <Card.Root>
            <Table.Root>
              <Table.Header>
                <Table.Row>
                  <Table.Head>Name</Table.Head>
                  <Table.Head>Resource Group</Table.Head>
                  <Table.Head>Services</Table.Head>
                  <Table.Head>Status</Table.Head>
                  <Table.Head>Erstellt</Table.Head>
                  <Table.Head class="text-right">Aktionen</Table.Head>
                </Table.Row>
              </Table.Header>
              <Table.Body>
                {#each dockerStacks as resource}
                  <Table.Row>
                    <Table.Cell class="font-medium">
                      <a
                        href="/resources/{resource.id}"
                        class="hover:underline"
                      >
                        {resource.name}
                      </a>
                      {#if resource.description}
                        <p class="text-xs text-gray-500 mt-1">
                          {resource.description}
                        </p>
                      {/if}
                    </Table.Cell>
                    <Table.Cell>
                      <a
                        href="/resource-groups/{resource.resource_group_id}"
                        class="hover:underline text-blue-600"
                      >
                        {resource.resource_group_name}
                      </a>
                    </Table.Cell>
                    <Table.Cell>
                      {#if resource.configuration?.services}
                        {resource.configuration.services.length} Services
                      {:else}
                        -
                      {/if}
                    </Table.Cell>
                    <Table.Cell>
                      <Badge variant={getStatusBadgeVariant(resource.status)}>
                        {resource.status}
                      </Badge>
                    </Table.Cell>
                    <Table.Cell>
                      {new Date(resource.created_at).toLocaleDateString(
                        "de-DE"
                      )}
                    </Table.Cell>
                    <Table.Cell class="text-right">
                      <div class="flex justify-end gap-2">
                        <Button
                          size="sm"
                          variant="outline"
                          onclick={() => goto(`/resources/${resource.id}`)}
                        >
                          Details
                        </Button>
                        <Button
                          size="sm"
                          variant="destructive"
                          onclick={() => openDeleteDialog(resource)}
                        >
                          <Trash2 class="h-4 w-4" />
                        </Button>
                      </div>
                    </Table.Cell>
                  </Table.Row>
                {/each}
              </Table.Body>
            </Table.Root>
          </Card.Root>
        </div>
      {/if}

      <!-- Docker Container -->
      {#if dockerContainers.length > 0}
        <div>
          <h2 class="text-2xl font-semibold mb-4 flex items-center gap-2">
            <Server class="h-6 w-6" />
            Docker Container ({dockerContainers.length})
          </h2>
          <Card.Root>
            <Table.Root>
              <Table.Header>
                <Table.Row>
                  <Table.Head>Name</Table.Head>
                  <Table.Head>Resource Group</Table.Head>
                  <Table.Head>Image</Table.Head>
                  <Table.Head>Status</Table.Head>
                  <Table.Head>Erstellt</Table.Head>
                  <Table.Head class="text-right">Aktionen</Table.Head>
                </Table.Row>
              </Table.Header>
              <Table.Body>
                {#each dockerContainers as resource}
                  <Table.Row>
                    <Table.Cell class="font-medium">
                      <a
                        href="/resources/{resource.id}"
                        class="hover:underline"
                      >
                        {resource.name}
                      </a>
                      {#if resource.description}
                        <p class="text-xs text-gray-500 mt-1">
                          {resource.description}
                        </p>
                      {/if}
                    </Table.Cell>
                    <Table.Cell>
                      <a
                        href="/resource-groups/{resource.resource_group_id}"
                        class="hover:underline text-blue-600"
                      >
                        {resource.resource_group_name}
                      </a>
                    </Table.Cell>
                    <Table.Cell>
                      {#if resource.configuration?.image}
                        <code class="text-xs bg-gray-100 px-2 py-1 rounded">
                          {resource.configuration.image}
                        </code>
                      {:else}
                        -
                      {/if}
                    </Table.Cell>
                    <Table.Cell>
                      <Badge variant={getStatusBadgeVariant(resource.status)}>
                        {resource.status}
                      </Badge>
                    </Table.Cell>
                    <Table.Cell>
                      {new Date(resource.created_at).toLocaleDateString(
                        "de-DE"
                      )}
                    </Table.Cell>
                    <Table.Cell class="text-right">
                      <div class="flex justify-end gap-2">
                        <Button
                          size="sm"
                          variant="outline"
                          onclick={() => goto(`/resources/${resource.id}`)}
                        >
                          Details
                        </Button>
                        <Button
                          size="sm"
                          variant="destructive"
                          onclick={() => openDeleteDialog(resource)}
                        >
                          <Trash2 class="h-4 w-4" />
                        </Button>
                      </div>
                    </Table.Cell>
                  </Table.Row>
                {/each}
              </Table.Body>
            </Table.Root>
          </Card.Root>
        </div>
      {/if}
    </div>
  {/if}
</div>

<!-- Delete Confirmation Dialog -->
<Dialog.Root bind:open={showDeleteDialog}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Ressource löschen</Dialog.Title>
      <Dialog.Description>
        Möchten Sie die Ressource "{resourceToDelete?.name}" wirklich löschen?
        Diese Aktion kann nicht rückgängig gemacht werden.
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

<!-- Deploy Container Dialog -->
<DeployDockerContainerDialog
  bind:open={showDeployDialog}
  resourceGroups={resourceGroups.map((rg) => ({ id: rg.id, name: rg.name }))}
  on:deploy={handleDeploy}
  on:cancel={() => (showDeployDialog = false)}
/>
