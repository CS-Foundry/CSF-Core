<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import {
    getResourceGroup,
    deleteResourceGroup,
  } from "$lib/services/resource-groups";
  import type { ResourceGroup } from "$lib/types/resource-group";
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

  let resourceGroup = $state<ResourceGroup | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let deleteDialogOpen = $state(false);

  const resourceGroupId = $derived($page.params.id);

  async function loadResourceGroup() {
    if (!resourceGroupId) return;

    loading = true;
    error = null;
    try {
      resourceGroup = await getResourceGroup(resourceGroupId);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load resource group";
    } finally {
      loading = false;
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
          <CardTitle class="flex items-center gap-2">
            <Server class="h-5 w-5" />
            Ressourcen
          </CardTitle>
          <CardDescription>
            Ressourcen in dieser Resource Group (Docker, KVM, etc.)
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div class="text-center py-12">
            <Server class="h-16 w-16 mx-auto text-muted-foreground mb-4" />
            <h3 class="text-lg font-semibold mb-2">
              Keine Ressourcen vorhanden
            </h3>
            <p class="text-muted-foreground mb-4">
              Die Ressourcenverwaltung wird in einem zukünftigen Update
              hinzugefügt
            </p>
            <Button variant="outline" disabled>
              <Server class="h-4 w-4 mr-2" />
              Ressource hinzufügen (Demnächst)
            </Button>
          </div>
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
