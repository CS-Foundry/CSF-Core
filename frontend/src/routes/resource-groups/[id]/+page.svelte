<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import {
    getResourceGroup,
    deleteResourceGroup,
  } from "$lib/services/resource-groups";
  import {
    listResourcesByGroup,
    createResource,
  } from "$lib/services/resources";
  import type { ResourceGroup } from "$lib/types/resource-group";
  import type { Resource } from "$lib/types/resource";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import {
    ArrowLeft,
    Edit,
    Trash2,
    RefreshCw,
    FolderOpen,
    MapPin,
    Calendar,
    User,
    Server,
    Plus,
    ShoppingBag,
    Package,
    Box,
  } from "@lucide/svelte";
  import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
  } from "$lib/components/ui/alert-dialog";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Textarea } from "$lib/components/ui/textarea";
  import {
    listTemplates,
    installTemplate,
    type MarketplaceTemplate,
  } from "$lib/services/marketplace";

  let resourceGroup = $state<ResourceGroup | null>(null);
  let resources = $state<Resource[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let deleteDialogOpen = $state(false);

  // Marketplace dialog
  let showMarketplaceDialog = $state(false);
  let marketplaceTemplates = $state<MarketplaceTemplate[]>([]);
  let searchQuery = $state("");
  let selectedTemplate = $state<MarketplaceTemplate | null>(null);
  let installing = $state(false);
  let customResourceName = $state("");

  const resourceGroupId = $derived($page.params.id);

  const filteredTemplates = $derived(
    searchQuery
      ? marketplaceTemplates.filter(
          (t) =>
            t.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
            t.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
            t.category.toLowerCase().includes(searchQuery.toLowerCase())
        )
      : marketplaceTemplates
  );

  async function loadResourceGroup() {
    if (!resourceGroupId) return;

    loading = true;
    error = null;
    try {
      resourceGroup = await getResourceGroup(resourceGroupId);
      resources = await listResourcesByGroup(resourceGroupId);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load resource group";
    } finally {
      loading = false;
    }
  }

  async function openMarketplaceDialog() {
    showMarketplaceDialog = true;
    searchQuery = "";
    selectedTemplate = null;
    customResourceName = "";

    // Load templates
    try {
      marketplaceTemplates = await listTemplates();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load marketplace";
    }
  }

  async function handleInstallTemplate() {
    if (!selectedTemplate || !customResourceName || !resourceGroupId) {
      error = "Bitte Template auswählen und Namen eingeben";
      return;
    }

    installing = true;
    error = null;

    try {
      await installTemplate({
        template_id: selectedTemplate.template_id,
        name: customResourceName,
        resource_group_id: resourceGroupId,
      });

      showMarketplaceDialog = false;
      await loadResourceGroup();
    } catch (e) {
      error = e instanceof Error ? e.message : "Installation fehlgeschlagen";
    } finally {
      installing = false;
    }
  }

  function handleEdit() {
    goto(`/resource-groups/${resourceGroupId}/edit`);
  }

  function openDeleteDialog() {
    deleteDialogOpen = true;
  }

  async function handleDelete() {
    if (!resourceGroupId) return;

    try {
      await deleteResourceGroup(resourceGroupId);
      goto("/resource-groups");
    } catch (e) {
      error =
        e instanceof Error ? e.message : "Failed to delete resource group";
      deleteDialogOpen = false;
    }
  }

  function handleBack() {
    goto("/resource-groups");
  }

  function formatDate(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleDateString("de-DE", {
      year: "numeric",
      month: "long",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  onMount(() => {
    loadResourceGroup();
  });
</script>

<div class="container mx-auto p-6">
  {#if loading}
    <Card>
      <CardContent class="pt-6">
        <div class="flex items-center justify-center py-8">
          <RefreshCw class="h-8 w-8 animate-spin text-muted-foreground" />
        </div>
      </CardContent>
    </Card>
  {:else if error || !resourceGroup}
    <Card class="border-destructive">
      <CardContent class="pt-6">
        <p class="text-destructive">
          {error || "Resource Group nicht gefunden"}
        </p>
        <Button onclick={handleBack} class="mt-4">Zurück zur Übersicht</Button>
      </CardContent>
    </Card>
  {:else}
    <div class="mb-6">
      <Button variant="ghost" onclick={handleBack} class="mb-4">
        <ArrowLeft class="h-4 w-4 mr-2" />
        Zurück
      </Button>
      <div class="flex items-start justify-between">
        <div>
          <h1 class="text-3xl font-bold flex items-center gap-2">
            <FolderOpen class="h-8 w-8" />
            {resourceGroup.name}
          </h1>
          {#if resourceGroup.description}
            <p class="text-muted-foreground mt-2">
              {resourceGroup.description}
            </p>
          {/if}
        </div>
        <div class="flex gap-2">
          <Button onclick={openMarketplaceDialog}>
            <Plus class="h-4 w-4 mr-2" />
            Ressource hinzufügen
          </Button>
          <Button variant="outline" onclick={handleEdit}>
            <Edit class="h-4 w-4 mr-2" />
            Bearbeiten
          </Button>
          <Button variant="destructive" onclick={openDeleteDialog}>
            <Trash2 class="h-4 w-4 mr-2" />
            Löschen
          </Button>
        </div>
      </div>
    </div>

    <div class="grid gap-6 md:grid-cols-2">
      <!-- Details Card -->
      <Card>
        <CardHeader>
          <CardTitle>Details</CardTitle>
          <CardDescription>Allgemeine Informationen</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-start gap-3">
            <FolderOpen class="h-5 w-5 text-muted-foreground mt-0.5" />
            <div class="flex-1">
              <p class="text-sm text-muted-foreground">Resource Group Name</p>
              <p class="font-medium">{resourceGroup.name}</p>
            </div>
          </div>

          {#if resourceGroup.location}
            <div class="flex items-start gap-3">
              <MapPin class="h-5 w-5 text-muted-foreground mt-0.5" />
              <div class="flex-1">
                <p class="text-sm text-muted-foreground">Location</p>
                <Badge variant="outline">{resourceGroup.location}</Badge>
              </div>
            </div>
          {/if}

          <div class="flex items-start gap-3">
            <Calendar class="h-5 w-5 text-muted-foreground mt-0.5" />
            <div class="flex-1">
              <p class="text-sm text-muted-foreground">Erstellt am</p>
              <p class="font-medium">{formatDate(resourceGroup.created_at)}</p>
            </div>
          </div>

          <div class="flex items-start gap-3">
            <Calendar class="h-5 w-5 text-muted-foreground mt-0.5" />
            <div class="flex-1">
              <p class="text-sm text-muted-foreground">Zuletzt aktualisiert</p>
              <p class="font-medium">{formatDate(resourceGroup.updated_at)}</p>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- Tags Card -->
      <Card>
        <CardHeader>
          <CardTitle>Tags</CardTitle>
          <CardDescription>Metadaten und Labels</CardDescription>
        </CardHeader>
        <CardContent>
          {#if resourceGroup.tags && Object.keys(resourceGroup.tags).length > 0}
            <div class="flex flex-wrap gap-2">
              {#each Object.entries(resourceGroup.tags) as [key, value]}
                <Badge variant="secondary">
                  {key}: {value}
                </Badge>
              {/each}
            </div>
          {:else}
            <p class="text-muted-foreground text-sm">Keine Tags vorhanden</p>
          {/if}
        </CardContent>
      </Card>

      <!-- Resources Card -->
      <Card class="md:col-span-2">
        <CardHeader>
          <div class="flex items-center justify-between">
            <div>
              <CardTitle class="flex items-center gap-2">
                <Server class="h-5 w-5" />
                Ressourcen
              </CardTitle>
              <CardDescription>
                Docker Container und Stacks in dieser Resource Group
              </CardDescription>
            </div>
            <Button size="sm" onclick={openMarketplaceDialog}>
              <Plus class="h-4 w-4 mr-2" />
              Neu
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {#if resources.length === 0}
            <div class="text-center py-12">
              <Package class="h-16 w-16 mx-auto text-muted-foreground mb-4" />
              <h3 class="text-lg font-semibold mb-2">
                Keine Ressourcen vorhanden
              </h3>
              <p class="text-muted-foreground mb-4">
                Erstelle deine erste Ressource oder wähle aus dem Marketplace
              </p>
              <div class="flex gap-2 justify-center">
                <Button onclick={openMarketplaceDialog}>
                  <Plus class="h-4 w-4 mr-2" />
                  Neue Ressource
                </Button>
                <Button
                  variant="outline"
                  onclick={() =>
                    goto(`/marketplace?resourceGroupId=${resourceGroupId}`)}
                >
                  <ShoppingBag class="h-4 w-4 mr-2" />
                  Marketplace
                </Button>
              </div>
            </div>
          {:else}
            <div class="space-y-3">
              {#each resources as resource}
                <div
                  class="flex items-center justify-between p-4 border rounded-lg hover:bg-gray-50 cursor-pointer transition-colors"
                  onclick={() => goto(`/resources/${resource.id}`)}
                >
                  <div class="flex items-center gap-3">
                    {#if resource.resource_type === "docker-stack"}
                      <Package class="h-8 w-8 text-blue-600" />
                    {:else}
                      <Box class="h-8 w-8 text-green-600" />
                    {/if}
                    <div>
                      <h4 class="font-semibold">{resource.name}</h4>
                      <p class="text-sm text-muted-foreground">
                        {resource.resource_type}
                        {#if resource.description}
                          · {resource.description}
                        {/if}
                      </p>
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <Badge
                      variant={resource.status === "running"
                        ? "default"
                        : "secondary"}
                    >
                      {resource.status}
                    </Badge>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </CardContent>
      </Card>
    </div>
  {/if}
</div>

<AlertDialog bind:open={deleteDialogOpen}>
  <AlertDialogContent>
    <AlertDialogHeader>
      <AlertDialogTitle>Resource Group löschen?</AlertDialogTitle>
      <AlertDialogDescription>
        Möchtest du die Resource Group "{resourceGroup?.name}" wirklich löschen?
        Diese Aktion kann nicht rückgängig gemacht werden. Alle Ressourcen in
        dieser Group werden ebenfalls entfernt.
      </AlertDialogDescription>
    </AlertDialogHeader>
    <AlertDialogFooter>
      <AlertDialogCancel>Abbrechen</AlertDialogCancel>
      <AlertDialogAction onclick={handleDelete} class="bg-destructive">
        Löschen
      </AlertDialogAction>
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>

<!-- Marketplace Dialog -->
<Dialog.Root bind:open={showMarketplaceDialog}>
  <Dialog.Content class="max-w-4xl max-h-[90vh] overflow-y-auto">
    <Dialog.Header>
      <Dialog.Title>Ressource aus Marketplace auswählen</Dialog.Title>
      <Dialog.Description>
        Wähle ein Template aus oder starte mit einem leeren Container/Stack
      </Dialog.Description>
    </Dialog.Header>

    <div class="space-y-4 py-4">
      {#if error}
        <div
          class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded"
        >
          {error}
        </div>
      {/if}

      <!-- Search -->
      <div class="space-y-2">
        <Label for="search">Suchen</Label>
        <Input
          id="search"
          bind:value={searchQuery}
          placeholder="Suche nach Templates..."
          class="w-full"
        />
      </div>

      <!-- Templates Grid -->
      <div
        class="grid grid-cols-1 md:grid-cols-2 gap-3 max-h-[400px] overflow-y-auto"
      >
        {#each filteredTemplates as template}
          <button
            type="button"
            class="relative flex flex-col items-start p-4 border-2 rounded-lg transition-all text-left {selectedTemplate?.template_id ===
            template.template_id
              ? 'border-blue-600 bg-blue-50'
              : 'border-gray-200 hover:border-gray-300'}"
            onclick={() => {
              selectedTemplate = template;
              customResourceName = template.name;
            }}
          >
            <div class="flex items-start gap-3 w-full">
              <span class="text-3xl flex-shrink-0">{template.icon}</span>
              <div class="flex-1 min-w-0">
                <div class="font-semibold flex items-center gap-2">
                  {template.name}
                  {#if template.popular}
                    <Badge variant="secondary" class="text-xs">Beliebt</Badge>
                  {/if}
                </div>
                <div class="text-xs text-gray-500 mt-1 line-clamp-2">
                  {template.description}
                </div>
                <div class="flex gap-2 mt-2">
                  <Badge variant="outline" class="text-xs">
                    {template.category}
                  </Badge>
                  <Badge variant="outline" class="text-xs">
                    {template.resource_type === "docker-stack"
                      ? "📦 Stack"
                      : "🐳 Container"}
                  </Badge>
                </div>
              </div>
            </div>
            {#if selectedTemplate?.template_id === template.template_id}
              <div class="absolute top-2 right-2">
                <div
                  class="w-5 h-5 bg-blue-600 rounded-full flex items-center justify-center"
                >
                  <svg
                    class="w-3 h-3 text-white"
                    fill="currentColor"
                    viewBox="0 0 20 20"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                      clip-rule="evenodd"
                    />
                  </svg>
                </div>
              </div>
            {/if}
          </button>
        {/each}
      </div>

      {#if selectedTemplate}
        <div class="space-y-4 border-t pt-4">
          <div class="space-y-2">
            <Label for="resourceName">Ressourcenname *</Label>
            <Input
              id="resourceName"
              bind:value={customResourceName}
              placeholder="z.B. webserver-prod"
              disabled={installing}
            />
          </div>

          <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
            <div class="flex gap-3">
              <div class="text-2xl">ℹ️</div>
              <div class="text-sm">
                <p class="font-semibold mb-1">Template Info:</p>
                <p class="text-gray-700">
                  {selectedTemplate.description}
                </p>
                {#if selectedTemplate.configuration?.services}
                  <p class="text-gray-700 mt-2">
                    Enthält {selectedTemplate.configuration.services.length} Service(s)
                  </p>
                {/if}
              </div>
            </div>
          </div>
        </div>
      {/if}
    </div>

    <Dialog.Footer>
      <Button
        variant="outline"
        onclick={() => (showMarketplaceDialog = false)}
        disabled={installing}
      >
        Abbrechen
      </Button>
      <Button
        onclick={handleInstallTemplate}
        disabled={installing || !selectedTemplate || !customResourceName}
      >
        {#if installing}
          <RefreshCw class="h-4 w-4 mr-2 animate-spin" />
          Installiere...
        {:else}
          <Plus class="h-4 w-4 mr-2" />
          Installieren
        {/if}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
