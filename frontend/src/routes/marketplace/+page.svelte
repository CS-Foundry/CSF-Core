<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { listResourceGroups } from "$lib/services/resource-groups";
  import {
    listTemplates,
    listPopularTemplates,
    installTemplate,
    seedMarketplace,
  } from "$lib/services/marketplace";
  import type { ResourceGroup } from "$lib/types/resource-group";
  import type { MarketplaceTemplate } from "$lib/services/marketplace";
  import { createResource } from "$lib/services/resources";
  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Badge } from "$lib/components/ui/badge";
  import * as Select from "$lib/components/ui/select";
  import { Search, Star, Layers, Container } from "@lucide/svelte";
  import DeployDockerContainerDialog from "$lib/components/DeployDockerContainerDialog.svelte";

  let resourceGroups: ResourceGroup[] = [];
  let templates: MarketplaceTemplate[] = [];
  let popularTemplates: MarketplaceTemplate[] = [];
  let loading = true;
  let error = "";
  let searchQuery = "";

  // Dialog state für Docker Stacks
  let showStackDialog = false;
  let selectedStackTemplate: MarketplaceTemplate | null = null;
  let deployName = "";
  let deployResourceGroupId = "";
  let deploying = false;

  // Dialog state für Docker Container
  let showContainerDialog = false;
  let selectedContainerTemplate: MarketplaceTemplate | null = null;

  // Get resource group ID from URL if present
  $: preselectedResourceGroupId = $page.url.searchParams.get("resourceGroupId");

  onMount(async () => {
    await Promise.all([
      loadResourceGroups(),
      loadTemplates(),
      loadPopularTemplates(),
    ]);

    // If a resource group is preselected, set it
    if (preselectedResourceGroupId) {
      deployResourceGroupId = preselectedResourceGroupId;
    }
  });

  async function loadResourceGroups() {
    try {
      error = "";
      const response = await listResourceGroups();
      resourceGroups = response;
    } catch (e: any) {
      error = e.message || "Failed to load resource groups";
    }
  }

  async function loadTemplates() {
    try {
      loading = true;
      error = "";
      templates = await listTemplates();
    } catch (e: any) {
      error = e.message || "Failed to load templates";
      // If no templates exist, seed them
      if (e.message.includes("not found") || templates.length === 0) {
        await handleSeedMarketplace();
      }
    } finally {
      loading = false;
    }
  }

  async function loadPopularTemplates() {
    try {
      popularTemplates = await listPopularTemplates();
    } catch (e: any) {
      console.error("Failed to load popular templates:", e);
    }
  }

  async function handleSeedMarketplace() {
    try {
      await seedMarketplace();
      await loadTemplates();
      await loadPopularTemplates();
    } catch (e: any) {
      console.error("Failed to seed marketplace:", e);
    }
  }

  function openStackDialog(template: MarketplaceTemplate) {
    selectedStackTemplate = template;
    deployName = `${template.name.toLowerCase().replace(/\s+/g, "-")}-${Date.now()}`;
    showStackDialog = true;
  }

  function openContainerDialog(template: MarketplaceTemplate) {
    selectedContainerTemplate = template;
    showContainerDialog = true;
  }

  async function handleDeployStack() {
    if (!selectedStackTemplate || !deployName || !deployResourceGroupId) {
      error = "Bitte füllen Sie alle Felder aus";
      return;
    }

    try {
      deploying = true;
      error = "";

      await installTemplate({
        template_id: selectedStackTemplate.template_id,
        name: deployName,
        resource_group_id: deployResourceGroupId,
      });

      showStackDialog = false;
      goto(`/resources`);
    } catch (e: any) {
      error = e.message || "Failed to deploy resource";
    } finally {
      deploying = false;
    }
  }

  async function handleDeployContainer(event: CustomEvent) {
    const config = event.detail;

    try {
      deploying = true;
      error = "";

      await createResource({
        name: config.name,
        description: config.description,
        resource_type: "docker-container",
        resource_group_id: config.resource_group_id,
        configuration: {
          image: config.image,
          ports: config.ports,
          environment: config.environment,
          volumes: config.volumes,
        },
      });

      showContainerDialog = false;
      goto(`/resources`);
    } catch (e: any) {
      error = e.message || "Failed to deploy container";
    } finally {
      deploying = false;
    }
  }

  function handleCancelContainer() {
    showContainerDialog = false;
  }

  $: filteredTemplates = templates.filter((template) => {
    const matchesSearch =
      template.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      template.description.toLowerCase().includes(searchQuery.toLowerCase());

    return matchesSearch;
  });

  $: containerTemplates = filteredTemplates.filter(
    (t) => t.resource_type === "docker-container"
  );

  $: stackTemplates = filteredTemplates.filter(
    (t) => t.resource_type === "docker-stack"
  );
</script>

<div class="container mx-auto p-6 max-w-7xl">
  <div class="mb-8">
    <h1 class="text-4xl font-bold mb-2">Marketplace</h1>
    <p class="text-gray-600">
      Wählen Sie aus vorgefertigten Docker Stack Vorlagen
    </p>
  </div>

  {#if error}
    <div
      class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded mb-4"
    >
      {error}
    </div>
  {/if}

  <!-- Search Bar -->
  <div class="mb-6">
    <div class="relative">
      <Search
        class="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4"
      />
      <Input
        type="text"
        placeholder="Suchen Sie nach Docker Containern oder Stacks..."
        bind:value={searchQuery}
        class="pl-10"
      />
    </div>
  </div>

  <!-- Popular Templates -->
  {#if !searchQuery && popularTemplates.length > 0}
    <div class="mb-8">
      <h2 class="text-2xl font-semibold mb-4 flex items-center gap-2">
        <Star class="h-6 w-6 text-yellow-500" />
        Beliebt
      </h2>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each popularTemplates as template}
          <Card.Root
            class="hover:shadow-lg transition-shadow cursor-pointer border-2 border-yellow-100"
          >
            <Card.Header>
              <div class="flex items-start justify-between">
                <div class="flex items-center gap-3">
                  <span class="text-4xl">{template.icon}</span>
                  <div>
                    <Card.Title class="text-lg">{template.name}</Card.Title>
                    <Badge variant="secondary" class="mt-1">Docker Stack</Badge>
                  </div>
                </div>
                <Star class="h-5 w-5 text-yellow-500 fill-yellow-500" />
              </div>
            </Card.Header>
            <Card.Content>
              <p class="text-sm text-gray-600 mb-4">{template.description}</p>
              <div class="flex items-center gap-2 text-sm text-gray-500">
                <Layers class="h-4 w-4" />
                <span>{template.configuration.services.length} Services</span>
              </div>
            </Card.Content>
            <Card.Footer>
              <Button
                class="w-full"
                onclick={() => {
                  if (template.resource_type === "docker-container") {
                    openContainerDialog(template);
                  } else {
                    openStackDialog(template);
                  }
                }}
              >
                Bereitstellen
              </Button>
            </Card.Footer>
          </Card.Root>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Docker Container -->
  {#if containerTemplates.length > 0}
    <div class="mb-8">
      <h2 class="text-2xl font-semibold mb-4 flex items-center gap-2">
        <Container class="h-6 w-6" />
        Docker Container
      </h2>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each containerTemplates as template}
          <Card.Root class="hover:shadow-lg transition-shadow">
            <Card.Header>
              <div class="flex items-center gap-3">
                <span class="text-3xl">{template.icon}</span>
                <div>
                  <Card.Title class="text-lg">{template.name}</Card.Title>
                  <Badge variant="outline" class="mt-1">Container</Badge>
                </div>
              </div>
            </Card.Header>
            <Card.Content>
              <p class="text-sm text-gray-600 mb-4">{template.description}</p>
            </Card.Content>
            <Card.Footer>
              <Button
                class="w-full"
                variant="outline"
                onclick={() => openContainerDialog(template)}
              >
                Bereitstellen
              </Button>
            </Card.Footer>
          </Card.Root>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Docker Stacks -->
  {#if stackTemplates.length > 0}
    <div class="mb-6">
      <h2 class="text-2xl font-semibold mb-4 flex items-center gap-2">
        <Layers class="h-6 w-6" />
        Docker Stacks
      </h2>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each stackTemplates as template}
          <Card.Root class="hover:shadow-lg transition-shadow">
            <Card.Header>
              <div class="flex items-center gap-3">
                <span class="text-3xl">{template.icon}</span>
                <div>
                  <Card.Title class="text-lg">{template.name}</Card.Title>
                  <Badge variant="outline" class="mt-1">Stack</Badge>
                </div>
              </div>
            </Card.Header>
            <Card.Content>
              <p class="text-sm text-gray-600 mb-4">{template.description}</p>

              <div class="space-y-2">
                <p class="text-xs font-semibold text-gray-500 uppercase">
                  Enthaltene Services:
                </p>
                <div class="flex flex-wrap gap-1">
                  {#each template.configuration.services as service}
                    <Badge variant="secondary" class="text-xs">
                      {service.name}
                    </Badge>
                  {/each}
                </div>
              </div>
            </Card.Content>
            <Card.Footer>
              <Button
                class="w-full"
                variant="outline"
                onclick={() => openStackDialog(template)}
              >
                Bereitstellen
              </Button>
            </Card.Footer>
          </Card.Root>
        {/each}
      </div>
    </div>
  {/if}

  {#if containerTemplates.length === 0 && stackTemplates.length === 0}
    <Card.Root>
      <Card.Content class="py-12 text-center">
        <p class="text-gray-500">Keine Vorlagen gefunden</p>
      </Card.Content>
    </Card.Root>
  {/if}
</div>

<!-- Stack Deploy Dialog -->
<Dialog.Root bind:open={showStackDialog}>
  <Dialog.Content class="max-w-2xl">
    <Dialog.Header>
      <Dialog.Title
        >Docker Stack bereitstellen: {selectedStackTemplate?.name}</Dialog.Title
      >
      <Dialog.Description>
        Konfigurieren Sie die Bereitstellung dieses Docker Stacks
      </Dialog.Description>
    </Dialog.Header>

    <div class="space-y-4 py-4">
      <div class="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-4">
        <div class="flex items-start gap-3">
          <span class="text-3xl">{selectedStackTemplate?.icon}</span>
          <div>
            <h3 class="font-semibold">{selectedStackTemplate?.name}</h3>
            <p class="text-sm text-gray-600 mt-1">
              {selectedStackTemplate?.description}
            </p>
          </div>
        </div>
      </div>

      <div class="border rounded-lg p-4 bg-gray-50">
        <p class="font-semibold mb-2 text-sm">Stack Konfiguration:</p>
        <div class="space-y-2">
          {#each selectedStackTemplate?.configuration.services || [] as service}
            <div class="flex items-center justify-between text-sm">
              <div>
                <span class="font-mono font-semibold">{service.name}</span>
                <span class="text-gray-500 ml-2">{service.image}</span>
              </div>
              {#if service.ports && service.ports.length > 0}
                <Badge variant="outline">
                  {service.ports
                    .map(
                      (p: any) => `${p.container}${p.host ? ":" + p.host : ""}`
                    )
                    .join(", ")}
                </Badge>
              {/if}
            </div>
          {/each}
        </div>
      </div>

      <div class="space-y-2">
        <Label for="deployName">Name *</Label>
        <Input
          id="deployName"
          bind:value={deployName}
          placeholder="z.B. wordpress-prod"
        />
      </div>

      <div class="space-y-2">
        <Label for="resourceGroup">Resource Group *</Label>
        <select
          id="resourceGroup"
          bind:value={deployResourceGroupId}
          class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visual:ring-offset-2"
        >
          <option value="">Wählen Sie eine Resource Group</option>
          {#each resourceGroups as rg}
            <option value={rg.id}>{rg.name}</option>
          {/each}
        </select>
      </div>
    </div>

    <Dialog.Footer>
      <Button variant="outline" onclick={() => (showStackDialog = false)}>
        Abbrechen
      </Button>
      <Button onclick={handleDeployStack}>Bereitstellen</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<!-- Container Deploy Dialog -->
<DeployDockerContainerDialog
  bind:open={showContainerDialog}
  {resourceGroups}
  on:deploy={handleDeployContainer}
  on:cancel={handleCancelContainer}
/>
