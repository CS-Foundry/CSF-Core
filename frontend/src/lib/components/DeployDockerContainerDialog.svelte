<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Textarea } from "$lib/components/ui/textarea";
  import { Plus, X, Rocket, RefreshCw } from "@lucide/svelte";

  export let open = false;
  export let resourceGroups: Array<{ id: string; name: string }> = [];
  export let initialData: any = null;

  const dispatch = createEventDispatcher();

  // Form state
  let containerName = initialData?.name || "";
  let dockerImage = initialData?.configuration?.image || "";
  let selectedResourceGroupId = initialData?.resource_group_id || "";
  let description = initialData?.description || "";

  // Ports: [{container: 80, host: 8080}]
  let ports: Array<{ container: number; host: number }> =
    initialData?.configuration?.ports || [];

  // Environment variables: {KEY: "value"}
  let envVars: Array<{ key: string; value: string }> = [];

  // Volumes: [{host: "/path", container: "/mount"}]
  let volumes: Array<{ host: string; container: string }> =
    initialData?.configuration?.volumes || [];

  // Parse initial env vars
  if (initialData?.configuration?.environment) {
    envVars = Object.entries(initialData.configuration.environment).map(
      ([key, value]) => ({ key, value: value as string })
    );
  }

  let deploying = false;
  let error = "";

  function addPort() {
    ports = [...ports, { container: 80, host: 8080 }];
  }

  function removePort(index: number) {
    ports = ports.filter((_, i) => i !== index);
  }

  function addEnvVar() {
    envVars = [...envVars, { key: "", value: "" }];
  }

  function removeEnvVar(index: number) {
    envVars = envVars.filter((_, i) => i !== index);
  }

  function addVolume() {
    volumes = [...volumes, { host: "", container: "" }];
  }

  function removeVolume(index: number) {
    volumes = volumes.filter((_, i) => i !== index);
  }

  async function handleDeploy() {
    error = "";

    // Validation
    if (!containerName.trim()) {
      error = "Container-Name ist erforderlich";
      return;
    }
    if (!dockerImage.trim()) {
      error = "Docker-Image ist erforderlich";
      return;
    }
    if (!selectedResourceGroupId) {
      error = "Bitte wähle eine Resource Group";
      return;
    }

    // Convert env vars array to object
    const environment: Record<string, string> = {};
    envVars.forEach(({ key, value }) => {
      if (key.trim()) {
        environment[key.trim()] = value;
      }
    });

    const deployConfig = {
      name: containerName.trim(),
      image: dockerImage.trim(),
      resource_group_id: selectedResourceGroupId,
      description: description.trim() || undefined,
      ports: ports.filter((p) => p.container > 0 && p.host > 0),
      environment,
      volumes: volumes.filter((v) => v.host.trim() && v.container.trim()),
    };

    try {
      deploying = true;
      dispatch("deploy", deployConfig);
    } finally {
      deploying = false;
    }
  }

  function handleCancel() {
    dispatch("cancel");
    resetForm();
  }

  function resetForm() {
    containerName = "";
    dockerImage = "";
    selectedResourceGroupId = "";
    description = "";
    ports = [];
    envVars = [];
    volumes = [];
    error = "";
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-w-3xl max-h-[90vh] overflow-y-auto">
    <Dialog.Header>
      <Dialog.Title class="flex items-center gap-2">
        <Rocket class="h-5 w-5" />
        Docker Container bereitstellen
      </Dialog.Title>
      <Dialog.Description>
        Konfiguriere und starte einen Docker Container auf der Host-Maschine
      </Dialog.Description>
    </Dialog.Header>

    {#if error}
      <div
        class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded mb-4"
      >
        {error}
      </div>
    {/if}

    <div class="space-y-6 py-4">
      <!-- Container Name -->
      <div class="space-y-2">
        <Label for="container-name">Container Name *</Label>
        <Input
          id="container-name"
          bind:value={containerName}
          placeholder="z.B. my-web-server"
          disabled={deploying}
        />
      </div>

      <!-- Docker Image -->
      <div class="space-y-2">
        <Label for="docker-image">Docker Image *</Label>
        <Input
          id="docker-image"
          bind:value={dockerImage}
          placeholder="z.B. nginx:latest, postgres:16-alpine"
          disabled={deploying}
        />
        <p class="text-sm text-gray-500">
          Das Image wird automatisch gepullt, falls nicht vorhanden
        </p>
      </div>

      <!-- Resource Group -->
      <div class="space-y-2">
        <Label for="resource-group">Resource Group *</Label>
        <select
          id="resource-group"
          bind:value={selectedResourceGroupId}
          class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
          disabled={deploying}
        >
          <option value="">Wähle eine Resource Group</option>
          {#each resourceGroups as rg}
            <option value={rg.id}>{rg.name}</option>
          {/each}
        </select>
      </div>

      <!-- Description -->
      <div class="space-y-2">
        <Label for="description">Beschreibung (optional)</Label>
        <Textarea
          id="description"
          bind:value={description}
          placeholder="Beschreibe den Zweck dieses Containers..."
          rows={2}
          disabled={deploying}
        />
      </div>

      <!-- Ports -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <Label>Port Mappings</Label>
          <Button
            variant="outline"
            size="sm"
            onclick={addPort}
            disabled={deploying}
          >
            <Plus class="h-4 w-4 mr-1" />
            Port hinzufügen
          </Button>
        </div>
        {#if ports.length === 0}
          <p class="text-sm text-gray-500">Keine Port-Mappings konfiguriert</p>
        {:else}
          <div class="space-y-2">
            {#each ports as port, index}
              <div class="flex items-center gap-2">
                <Input
                  type="number"
                  bind:value={port.host}
                  placeholder="Host Port"
                  class="flex-1"
                  disabled={deploying}
                />
                <span class="text-gray-500">→</span>
                <Input
                  type="number"
                  bind:value={port.container}
                  placeholder="Container Port"
                  class="flex-1"
                  disabled={deploying}
                />
                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => removePort(index)}
                  disabled={deploying}
                >
                  <X class="h-4 w-4" />
                </Button>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Environment Variables -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <Label>Umgebungsvariablen</Label>
          <Button
            variant="outline"
            size="sm"
            onclick={addEnvVar}
            disabled={deploying}
          >
            <Plus class="h-4 w-4 mr-1" />
            Variable hinzufügen
          </Button>
        </div>
        {#if envVars.length === 0}
          <p class="text-sm text-gray-500">
            Keine Umgebungsvariablen konfiguriert
          </p>
        {:else}
          <div class="space-y-2">
            {#each envVars as envVar, index}
              <div class="flex items-center gap-2">
                <Input
                  bind:value={envVar.key}
                  placeholder="KEY"
                  class="flex-1"
                  disabled={deploying}
                />
                <span class="text-gray-500">=</span>
                <Input
                  bind:value={envVar.value}
                  placeholder="value"
                  class="flex-1"
                  disabled={deploying}
                />
                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => removeEnvVar(index)}
                  disabled={deploying}
                >
                  <X class="h-4 w-4" />
                </Button>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Volumes -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <Label>Volume Mounts</Label>
          <Button
            variant="outline"
            size="sm"
            onclick={addVolume}
            disabled={deploying}
          >
            <Plus class="h-4 w-4 mr-1" />
            Volume hinzufügen
          </Button>
        </div>
        {#if volumes.length === 0}
          <p class="text-sm text-gray-500">Keine Volumes konfiguriert</p>
        {:else}
          <div class="space-y-2">
            {#each volumes as volume, index}
              <div class="flex items-center gap-2">
                <Input
                  bind:value={volume.host}
                  placeholder="/host/path"
                  class="flex-1"
                  disabled={deploying}
                />
                <span class="text-gray-500">→</span>
                <Input
                  bind:value={volume.container}
                  placeholder="/container/path"
                  class="flex-1"
                  disabled={deploying}
                />
                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => removeVolume(index)}
                  disabled={deploying}
                >
                  <X class="h-4 w-4" />
                </Button>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    <Dialog.Footer>
      <Button variant="outline" onclick={handleCancel} disabled={deploying}>
        Abbrechen
      </Button>
      <Button onclick={handleDeploy} disabled={deploying}>
        {#if deploying}
          <RefreshCw class="h-4 w-4 mr-2 animate-spin" />
          Wird bereitgestellt...
        {:else}
          <Rocket class="h-4 w-4 mr-2" />
          Bereitstellen
        {/if}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
