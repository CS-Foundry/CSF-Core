<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { listResourceGroups, deleteResourceGroup } from '$lib/services/resource-groups';
  import type { ResourceGroup } from '$lib/types/resource-group';
  import { Button } from '$lib/components/ui/button';
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from '$lib/components/ui/table';
  import { Badge } from '$lib/components/ui/badge';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { FolderOpen, Plus, Trash2, Edit, RefreshCw } from '@lucide/svelte';
  import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
  } from '$lib/components/ui/alert-dialog';

  let resourceGroups = $state<ResourceGroup[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let deleteDialogOpen = $state(false);
  let groupToDelete = $state<ResourceGroup | null>(null);

  async function loadResourceGroups() {
    loading = true;
    error = null;
    try {
      resourceGroups = await listResourceGroups();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load resource groups';
    } finally {
      loading = false;
    }
  }

  function handleCreateGroup() {
    goto('/resource-groups/create');
  }

  function handleEditGroup(id: string) {
    goto(`/resource-groups/${id}/edit`);
  }

  function handleViewGroup(id: string) {
    goto(`/resource-groups/${id}`);
  }

  function openDeleteDialog(group: ResourceGroup) {
    groupToDelete = group;
    deleteDialogOpen = true;
  }

  async function handleDeleteGroup() {
    if (!groupToDelete) return;

    try {
      await deleteResourceGroup(groupToDelete.id);
      deleteDialogOpen = false;
      groupToDelete = null;
      await loadResourceGroups();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to delete resource group';
    }
  }

  function formatDate(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleDateString('de-DE', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  }

  onMount(() => {
    loadResourceGroups();
  });
</script>

<div class="container mx-auto p-6">
  <div class="flex items-center justify-between mb-6">
    <div>
      <h1 class="text-3xl font-bold flex items-center gap-2">
        <FolderOpen class="h-8 w-8" />
        Resource Groups
      </h1>
      <p class="text-muted-foreground mt-2">
        Verwalte deine Azure-ähnlichen Resource Groups für Docker, KVM und andere Ressourcen
      </p>
    </div>
    <div class="flex gap-2">
      <Button variant="outline" size="icon" onclick={loadResourceGroups}>
        <RefreshCw class="h-4 w-4" />
      </Button>
      <Button onclick={handleCreateGroup}>
        <Plus class="h-4 w-4 mr-2" />
        Neue Resource Group
      </Button>
    </div>
  </div>

  {#if error}
    <Card class="mb-6 border-destructive">
      <CardContent class="pt-6">
        <p class="text-destructive">{error}</p>
      </CardContent>
    </Card>
  {/if}

  {#if loading}
    <Card>
      <CardContent class="pt-6">
        <div class="flex items-center justify-center py-8">
          <RefreshCw class="h-8 w-8 animate-spin text-muted-foreground" />
        </div>
      </CardContent>
    </Card>
  {:else if resourceGroups.length === 0}
    <Card>
      <CardContent class="pt-6">
        <div class="text-center py-12">
          <FolderOpen class="h-16 w-16 mx-auto text-muted-foreground mb-4" />
          <h3 class="text-lg font-semibold mb-2">Keine Resource Groups</h3>
          <p class="text-muted-foreground mb-4">
            Erstelle deine erste Resource Group, um Ressourcen zu organisieren
          </p>
          <Button onclick={handleCreateGroup}>
            <Plus class="h-4 w-4 mr-2" />
            Erste Resource Group erstellen
          </Button>
        </div>
      </CardContent>
    </Card>
  {:else}
    <Card>
      <CardHeader>
        <CardTitle>Resource Groups ({resourceGroups.length})</CardTitle>
        <CardDescription>
          Klicke auf eine Resource Group, um Details zu sehen und Ressourcen zu verwalten
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Beschreibung</TableHead>
              <TableHead>Location</TableHead>
              <TableHead>Erstellt am</TableHead>
              <TableHead>Tags</TableHead>
              <TableHead class="text-right">Aktionen</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {#each resourceGroups as group (group.id)}
              <TableRow
                class="cursor-pointer hover:bg-muted/50"
                onclick={() => handleViewGroup(group.id)}
              >
                <TableCell class="font-medium">{group.name}</TableCell>
                <TableCell class="text-muted-foreground">
                  {group.description || '—'}
                </TableCell>
                <TableCell>
                  {#if group.location}
                    <Badge variant="outline">{group.location}</Badge>
                  {:else}
                    <span class="text-muted-foreground">—</span>
                  {/if}
                </TableCell>
                <TableCell class="text-muted-foreground">
                  {formatDate(group.created_at)}
                </TableCell>
                <TableCell>
                  {#if group.tags && Object.keys(group.tags).length > 0}
                    <div class="flex gap-1 flex-wrap">
                      {#each Object.entries(group.tags).slice(0, 2) as [key, value]}
                        <Badge variant="secondary" class="text-xs">
                          {key}: {value}
                        </Badge>
                      {/each}
                      {#if Object.keys(group.tags).length > 2}
                        <Badge variant="secondary" class="text-xs">
                          +{Object.keys(group.tags).length - 2}
                        </Badge>
                      {/if}
                    </div>
                  {:else}
                    <span class="text-muted-foreground">—</span>
                  {/if}
                </TableCell>
                <TableCell class="text-right">
                  <div class="flex justify-end gap-2">
                    <Button
                      variant="ghost"
                      size="icon"
                      onclick={(e) => {
                        e.stopPropagation();
                        handleEditGroup(group.id);
                      }}
                    >
                      <Edit class="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      onclick={(e) => {
                        e.stopPropagation();
                        openDeleteDialog(group);
                      }}
                    >
                      <Trash2 class="h-4 w-4 text-destructive" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            {/each}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  {/if}
</div>

<AlertDialog bind:open={deleteDialogOpen}>
  <AlertDialogContent>
    <AlertDialogHeader>
      <AlertDialogTitle>Resource Group löschen?</AlertDialogTitle>
      <AlertDialogDescription>
        Möchtest du die Resource Group "{groupToDelete?.name}" wirklich löschen? Diese Aktion kann
        nicht rückgängig gemacht werden.
      </AlertDialogDescription>
    </AlertDialogHeader>
    <AlertDialogFooter>
      <AlertDialogCancel>Abbrechen</AlertDialogCancel>
      <AlertDialogAction onclick={handleDeleteGroup}>Löschen</AlertDialogAction>
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>
