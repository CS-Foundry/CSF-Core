<script lang="ts">
  import { Globe, MapPin, Activity } from '@lucide/svelte';
  import * as Card from '$lib/components/ui/card';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import { Badge } from '$lib/components/ui/badge';

  interface Props {
    loading: boolean;
  }

  let { loading }: Props = $props();

  // Mock data für Traffic-Visualisierung
  const trafficData = [
    { country: 'Germany', requests: 1247, lat: 51.1657, lon: 10.4515 },
    { country: 'USA', requests: 892, lat: 37.0902, lon: -95.7129 },
    { country: 'UK', requests: 634, lat: 55.3781, lon: -3.436 },
    { country: 'France', requests: 521, lat: 46.2276, lon: 2.2137 },
    { country: 'Netherlands', requests: 412, lat: 52.1326, lon: 5.2913 },
  ];

  const totalRequests = trafficData.reduce((sum, t) => sum + t.requests, 0);
</script>

<Card.Root>
  <Card.Header>
    <Card.Title class="flex items-center gap-2">
      <Globe class="h-5 w-5" />
      Traffic Map
    </Card.Title>
    <Card.Description>Echtzeit-Anfragen nach Geolocation</Card.Description>
  </Card.Header>
  <Card.Content class="space-y-4">
    {#if loading}
      <Skeleton class="h-48 w-full" />
    {:else}
      <!-- Simple World Map Visualization -->
      <div class="relative h-48 bg-muted/30 rounded-lg overflow-hidden border">
        <div class="absolute inset-0 flex items-center justify-center">
          <Globe class="h-24 w-24 text-muted-foreground/20" />
        </div>
        <div class="absolute top-4 right-4">
          <Badge variant="secondary" class="bg-primary text-primary-foreground">
            <Activity class="mr-1 h-3 w-3" />
            {totalRequests} requests/5min
          </Badge>
        </div>
      </div>

      <!-- Traffic by Region -->
      <div class="space-y-2">
        <p class="text-sm font-medium">Top Regions</p>
        {#each trafficData.slice(0, 5) as traffic}
          <div class="flex items-center justify-between text-sm">
            <div class="flex items-center gap-2">
              <MapPin class="h-3 w-3 text-muted-foreground" />
              <span>{traffic.country}</span>
            </div>
            <div class="flex items-center gap-2">
              <div class="w-20 h-2 bg-muted rounded-full overflow-hidden">
                <div
                  class="h-full bg-primary rounded-full"
                  style="width: {(traffic.requests / trafficData[0].requests) * 100}%"
                ></div>
              </div>
              <span class="text-xs text-muted-foreground w-12 text-right">{traffic.requests}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </Card.Content>
</Card.Root>
