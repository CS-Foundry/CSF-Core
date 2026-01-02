<script lang="ts">
  import { onMount } from 'svelte';
  import { organizationService } from '$lib/services/organization';
  import type { Organization } from '$lib/types/organization';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Building2 } from '@lucide/svelte';

  let organization = $state<Organization | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let success = $state<string | null>(null);

  let editMode = $state(false);
  let formData = $state({
    name: '',
    description: '',
  });

  async function loadOrganization() {
    loading = true;
    error = null;
    try {
      organization = await organizationService.getOrganization();
      formData = {
        name: organization.name,
        description: organization.description || '',
      };
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load organization';
    } finally {
      loading = false;
    }
  }

  async function handleUpdate() {
    loading = true;
    error = null;
    success = null;
    try {
      organization = await organizationService.updateOrganization({
        name: formData.name,
        description: formData.description || null,
      });
      editMode = false;
      success = 'Organization updated successfully';
      setTimeout(() => (success = null), 3000);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to update organization';
    } finally {
      loading = false;
    }
  }

  function cancelEdit() {
    if (organization) {
      formData = {
        name: organization.name,
        description: organization.description || '',
      };
    }
    editMode = false;
  }

  onMount(() => {
    loadOrganization();
  });
</script>

<div class="rounded-lg border bg-card p-6">
  <div class="mb-6 flex items-start justify-between">
    <div class="flex items-center gap-3">
      <div class="rounded-full bg-primary/10 p-3">
        <Building2 class="h-6 w-6 text-primary" />
      </div>
      <div>
        <h2 class="text-2xl font-bold">Organization</h2>
        <p class="text-sm text-muted-foreground">Manage your organization settings</p>
      </div>
    </div>
    {#if !editMode && organization}
      <Button onclick={() => (editMode = true)}>Edit</Button>
    {/if}
  </div>

  {#if error}
    <div class="mb-4 rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
      {error}
    </div>
  {/if}

  {#if success}
    <div class="mb-4 rounded-lg border border-green-600 bg-green-600/10 p-4 text-green-600">
      {success}
    </div>
  {/if}

  {#if loading && !organization}
    <div class="text-center">Loading...</div>
  {:else if organization}
    <div class="space-y-4">
      <div class="grid gap-2">
        <Label for="org-name">Organization Name</Label>
        {#if editMode}
          <Input id="org-name" bind:value={formData.name} />
        {:else}
          <p class="text-lg font-medium">{organization.name}</p>
        {/if}
      </div>

      <div class="grid gap-2">
        <Label for="org-description">Description</Label>
        {#if editMode}
          <textarea
            id="org-description"
            bind:value={formData.description}
            rows="4"
            class="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
          ></textarea>
        {:else}
          <p class="text-muted-foreground">
            {organization.description || 'No description set'}
          </p>
        {/if}
      </div>

      {#if !editMode}
        <div class="grid grid-cols-2 gap-4 pt-4 border-t">
          <div>
            <Label class="text-sm text-muted-foreground">Created</Label>
            <p class="font-medium">
              {new Date(organization.created_at).toLocaleString()}
            </p>
          </div>
          <div>
            <Label class="text-sm text-muted-foreground">Last Updated</Label>
            <p class="font-medium">
              {new Date(organization.updated_at).toLocaleString()}
            </p>
          </div>
        </div>
      {/if}

      {#if editMode}
        <div class="flex gap-2 pt-4">
          <Button onclick={handleUpdate} disabled={loading}>Save Changes</Button>
          <Button variant="outline" onclick={cancelEdit}>Cancel</Button>
        </div>
      {/if}
    </div>
  {/if}
</div>
