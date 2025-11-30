<script lang="ts">
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { MoreHorizontal, Pencil, Trash } from "@lucide/svelte";

  type Props = {
    onEdit: () => void;
    onDelete: () => void;
  };

  let { onEdit, onDelete }: Props = $props();
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger>
    {#snippet child({ props })}
      <Button variant="ghost" size="sm" class="h-8 w-8 p-0" {...props}>
        <span class="sr-only">Menü öffnen</span>
        <MoreHorizontal class="h-4 w-4" />
      </Button>
    {/snippet}
  </DropdownMenu.Trigger>
  <DropdownMenu.Content align="end">
    <DropdownMenu.Item onclick={onEdit}>
      <Pencil class="mr-2 h-4 w-4" />
      Bearbeiten
    </DropdownMenu.Item>
    <DropdownMenu.Separator />
    <DropdownMenu.Item onclick={onDelete} class="text-destructive">
      <Trash class="mr-2 h-4 w-4" />
      Löschen
    </DropdownMenu.Item>
  </DropdownMenu.Content>
</DropdownMenu.Root>
