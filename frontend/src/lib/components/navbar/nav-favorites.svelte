<script lang="ts">
  import * as Sidebar from '$lib/components/ui/sidebar/index.js';
  import { StarIcon } from '@lucide/svelte';

  let {
    favorites,
  }: {
    favorites: {
      name: string;
      company: string;
      url: string;
      initials: string;
    }[];
  } = $props();
</script>

<Sidebar.Group class="group-data-[collapsible=icon]:hidden">
  <Sidebar.GroupLabel>
    <StarIcon class="h-4 w-4 text-yellow-500 fill-yellow-500" />
    <span>Favoriten</span>
  </Sidebar.GroupLabel>
  <Sidebar.Menu>
    {#each favorites as item (item.name)}
      <Sidebar.MenuItem>
        <a href={item.url}>
          <Sidebar.MenuButton>
            <div class="flex items-center gap-3 w-full">
              <div
                class="flex h-8 w-8 items-center justify-center rounded-full bg-muted text-sm font-medium"
              >
                {item.initials}
              </div>
              <div class="flex flex-col items-start text-left flex-1 min-w-0">
                <span class="font-medium text-sm truncate w-full">{item.name}</span>
                <span class="text-xs text-muted-foreground truncate w-full">{item.company}</span>
              </div>
            </div>
          </Sidebar.MenuButton>
        </a>
      </Sidebar.MenuItem>
    {/each}
  </Sidebar.Menu>
</Sidebar.Group>
