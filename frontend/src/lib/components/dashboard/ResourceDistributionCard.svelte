<script lang="ts">
  import { Layers, Package, Database, Container } from '@lucide/svelte';
  import * as Card from '$lib/components/ui/card';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import { Progress } from '$lib/components/ui/progress';

  interface Props {
    loading: boolean;
  }

  let { loading }: Props = $props();

  // Mock data für Ressourcenverteilung
  const resourceGroups = [
    { name: 'Production Services', usage: 78, color: 'bg-blue-500', containers: 12 },
    { name: 'Development', usage: 45, color: 'bg-green-500', containers: 8 },
    { name: 'Testing', usage: 32, color: 'bg-yellow-500', containers: 5 },
    { name: 'Monitoring', usage: 23, color: 'bg-purple-500', containers: 3 },
  ];

  const totalContainers = resourceGroups.reduce((sum, g) => sum + g.containers, 0);
</script>

<Card.Root>
  <Card.Header>
    <Card.Title class="flex items-center gap-2">
      <Layers class="h-5 w-5" />
      Resource Distribution
    </Card.Title>
    <Card.Description>Ressourcenverteilung nach Gruppen</Card.Description>
  </Card.Header>
  <Card.Content class="space-y-4">
    {#if loading}
      <div class="space-y-4">
        <Skeleton class="h-16 w-full" />
        <Skeleton class="h-16 w-full" />
        <Skeleton class="h-16 w-full" />
      </div>
    {:else}
      <!-- Resource Groups Overview -->
      <div class="grid grid-cols-2 gap-4 pb-4 border-b">
        <div class="flex items-center gap-2">
          <Container class="h-4 w-4 text-muted-foreground" />
          <div>
            <p class="text-2xl font-bold">{totalContainers}</p>
            <p class="text-xs text-muted-foreground">Total Containers</p>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <Package class="h-4 w-4 text-muted-foreground" />
          <div>
            <p class="text-2xl font-bold">{resourceGroups.length}</p>
            <p class="text-xs text-muted-foreground">Resource Groups</p>
          </div>
        </div>
      </div>

      <!-- Resource Groups Details -->
      <div class="space-y-4">
        {#each resourceGroups as group}
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <div class="w-2 h-2 rounded-full {group.color}"></div>
                <span class="text-sm font-medium">{group.name}</span>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-xs text-muted-foreground">{group.containers} containers</span>
                <span class="text-sm font-bold">{group.usage}%</span>
              </div>
            </div>
            <Progress value={group.usage} class="h-2" />
          </div>
        {/each}
      </div>

      <!-- Quick Stats -->
      <div class="pt-4 border-t">
        <div class="flex items-center justify-between text-xs text-muted-foreground">
          <span>Avg. Resource Usage:</span>
          <span class="font-semibold text-foreground">
            {(resourceGroups.reduce((sum, g) => sum + g.usage, 0) / resourceGroups.length).toFixed(
              1
            )}%
          </span>
        </div>
      </div>
    {/if}
  </Card.Content>
</Card.Root>
