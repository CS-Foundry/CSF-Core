<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { getResourceGroup, updateResourceGroup } from '$lib/services/resource-groups';
  import type { ResourceGroup, UpdateResourceGroupRequest } from '$lib/types/resource-group';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Textarea } from '$lib/components/ui/textarea';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { ArrowLeft, Save, RefreshCw } from '@lucide/svelte';

  let resourceGroup = $state<ResourceGroup | null>(null);
  let formData = $state({
    name: '',
    description: '',
    location: '',
  });

  let errors = $state<Record<string, string>>({});
  let loading = $state(false);
  let initialLoading = $state(true);
  let generalError = $state<string | null>(null);

  const resourceGroupId = $derived($page.params.id);

  async function loadResourceGroup() {
    if (!resourceGroupId) return;

    initialLoading = true;
    try {
      resourceGroup = await getResourceGroup(resourceGroupId);
      formData = {
        name: resourceGroup.name,
        description: resourceGroup.description || '',
        location: resourceGroup.location || '',
      };
    } catch (e) {
      generalError = e instanceof Error ? e.message : 'Failed to load resource group';
    } finally {
      initialLoading = false;
    }
  }

  function validateForm(): boolean {
    const newErrors: Record<string, string> = {};

    if (formData.name.trim() && formData.name.length < 3) {
      newErrors.name = 'Name muss mindestens 3 Zeichen lang sein';
    }

    errors = newErrors;
    return Object.keys(newErrors).length === 0;
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!resourceGroupId) return;

    generalError = null;

    if (!validateForm()) {
      return;
    }

    loading = true;
    try {
      const payload: UpdateResourceGroupRequest = {};

      if (formData.name.trim() && formData.name !== resourceGroup?.name) {
        payload.name = formData.name.trim();
      }

      if (formData.description.trim() !== (resourceGroup?.description || '')) {
        payload.description = formData.description.trim() || undefined;
      }

      if (formData.location.trim() !== (resourceGroup?.location || '')) {
        payload.location = formData.location.trim() || undefined;
      }

      await updateResourceGroup(resourceGroupId, payload);
      goto(`/resource-groups/${resourceGroupId}`);
    } catch (e) {
      generalError = e instanceof Error ? e.message : 'Failed to update resource group';
    } finally {
      loading = false;
    }
  }

  function handleCancel() {
    goto(`/resource-groups/${resourceGroupId}`);
  }

  onMount(() => {
    loadResourceGroup();
  });
</script>

<div class="container mx-auto p-6 max-w-2xl">
  {#if initialLoading}
    <Card>
      <CardContent class="pt-6">
        <div class="flex items-center justify-center py-8">
          <RefreshCw class="h-8 w-8 animate-spin text-muted-foreground" />
        </div>
      </CardContent>
    </Card>
  {:else if !resourceGroup}
    <Card class="border-destructive">
      <CardContent class="pt-6">
        <p class="text-destructive">Resource Group nicht gefunden</p>
        <Button onclick={() => goto('/resource-groups')} class="mt-4">Zurück zur Übersicht</Button>
      </CardContent>
    </Card>
  {:else}
    <div class="mb-6">
      <Button variant="ghost" onclick={handleCancel} class="mb-4">
        <ArrowLeft class="h-4 w-4 mr-2" />
        Zurück
      </Button>
      <h1 class="text-3xl font-bold">Resource Group bearbeiten</h1>
      <p class="text-muted-foreground mt-2">
        Aktualisiere die Details der Resource Group "{resourceGroup.name}"
      </p>
    </div>

    {#if generalError}
      <Card class="mb-6 border-destructive">
        <CardContent class="pt-6">
          <p class="text-destructive">{generalError}</p>
        </CardContent>
      </Card>
    {/if}

    <form onsubmit={handleSubmit}>
      <Card>
        <CardHeader>
          <CardTitle>Resource Group Details</CardTitle>
          <CardDescription>Aktualisiere die Informationen für die Resource Group</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="space-y-2">
            <Label for="name">Name</Label>
            <Input
              id="name"
              bind:value={formData.name}
              placeholder="z.B. production-resources"
              class={errors.name ? 'border-destructive' : ''}
            />
            {#if errors.name}
              <p class="text-sm text-destructive">{errors.name}</p>
            {/if}
            <p class="text-sm text-muted-foreground">
              Der Name muss innerhalb deiner Organisation eindeutig sein
            </p>
          </div>

          <div class="space-y-2">
            <Label for="description">Beschreibung</Label>
            <Textarea
              id="description"
              bind:value={formData.description}
              placeholder="Beschreibe den Zweck dieser Resource Group"
              rows={3}
            />
          </div>

          <div class="space-y-2">
            <Label for="location">Location</Label>
            <Input
              id="location"
              bind:value={formData.location}
              placeholder="z.B. eu-west-1, us-east-1"
            />
          </div>
        </CardContent>
      </Card>

      <div class="flex gap-2 mt-6">
        <Button type="submit" disabled={loading}>
          <Save class="h-4 w-4 mr-2" />
          {loading ? 'Wird gespeichert...' : 'Änderungen speichern'}
        </Button>
        <Button type="button" variant="outline" onclick={handleCancel}>Abbrechen</Button>
      </div>
    </form>
  {/if}
</div>
