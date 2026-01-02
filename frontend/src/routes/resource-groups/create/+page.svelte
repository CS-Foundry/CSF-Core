<script lang="ts">
  import { goto } from '$app/navigation';
  import { createResourceGroup } from '$lib/services/resource-groups';
  import type { CreateResourceGroupRequest } from '$lib/types/resource-group';
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
  import { ArrowLeft, Save } from '@lucide/svelte';

  let formData = $state<CreateResourceGroupRequest>({
    name: '',
    description: '',
    location: '',
  });

  let errors = $state<Record<string, string>>({});
  let loading = $state(false);
  let generalError = $state<string | null>(null);

  function validateForm(): boolean {
    const newErrors: Record<string, string> = {};

    if (!formData.name.trim()) {
      newErrors.name = 'Name ist erforderlich';
    } else if (formData.name.length < 3) {
      newErrors.name = 'Name muss mindestens 3 Zeichen lang sein';
    }

    errors = newErrors;
    return Object.keys(newErrors).length === 0;
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    generalError = null;

    if (!validateForm()) {
      return;
    }

    loading = true;
    try {
      const payload: CreateResourceGroupRequest = {
        name: formData.name.trim(),
      };

      if (formData.description?.trim()) {
        payload.description = formData.description.trim();
      }

      if (formData.location?.trim()) {
        payload.location = formData.location.trim();
      }

      await createResourceGroup(payload);
      goto('/resource-groups');
    } catch (e) {
      generalError = e instanceof Error ? e.message : 'Failed to create resource group';
    } finally {
      loading = false;
    }
  }

  function handleCancel() {
    goto('/resource-groups');
  }
</script>

<div class="container mx-auto p-6 max-w-2xl">
  <div class="mb-6">
    <Button variant="ghost" onclick={handleCancel} class="mb-4">
      <ArrowLeft class="h-4 w-4 mr-2" />
      Zurück
    </Button>
    <h1 class="text-3xl font-bold">Neue Resource Group erstellen</h1>
    <p class="text-muted-foreground mt-2">
      Erstelle eine neue Resource Group, um deine Ressourcen zu organisieren
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
        <CardDescription>
          Gib die grundlegenden Informationen für die Resource Group ein
        </CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <div class="space-y-2">
          <Label for="name">
            Name <span class="text-destructive">*</span>
          </Label>
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
          <p class="text-sm text-muted-foreground">
            Optional: Eine kurze Beschreibung der Resource Group
          </p>
        </div>

        <div class="space-y-2">
          <Label for="location">Location</Label>
          <Input
            id="location"
            bind:value={formData.location}
            placeholder="z.B. eu-west-1, us-east-1"
          />
          <p class="text-sm text-muted-foreground">
            Optional: Der geografische Standort oder die Region
          </p>
        </div>
      </CardContent>
    </Card>

    <div class="flex gap-2 mt-6">
      <Button type="submit" disabled={loading}>
        <Save class="h-4 w-4 mr-2" />
        {loading ? 'Wird erstellt...' : 'Resource Group erstellen'}
      </Button>
      <Button type="button" variant="outline" onclick={handleCancel}>Abbrechen</Button>
    </div>
  </form>
</div>
