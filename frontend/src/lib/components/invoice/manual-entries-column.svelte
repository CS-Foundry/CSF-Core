<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Select from "$lib/components/ui/select";
  import { Plus, Trash2, Edit } from "@lucide/svelte";
  import type { ManualEntry } from "$lib/types";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";

  interface Props {
    entries: ManualEntry[];
    onAdd: () => void;
    onEdit: (id: string) => void;
    onDelete: (id: string) => void;
  }

  let { entries, onAdd, onEdit, onDelete }: Props = $props();

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString("de-DE");
  }
</script>

<Card class="h-full flex flex-col">
  <CardHeader>
    <div class="flex items-center justify-between">
      <div>
        <CardTitle>Manuelle Einträge</CardTitle>
        <CardDescription
          >Einträge von Kontoauszügen oder Notizen</CardDescription
        >
      </div>
      <Button size="sm" onclick={onAdd}>
        <Plus class="h-4 w-4 mr-2" />
        Eintrag hinzufügen
      </Button>
    </div>
  </CardHeader>
  <CardContent class="flex-1 overflow-auto">
    <div class="space-y-3">
      {#each entries as entry}
        <div
          class="p-4 border rounded-lg hover:bg-accent/50 transition-colors"
          class:border-green-500={entry.matched}
          class:bg-green-50={entry.matched}
          class:dark:bg-green-950={entry.matched}
        >
          <div class="flex justify-between items-start mb-2">
            <div class="flex-1">
              <p class="font-medium">{entry.description}</p>
              <p class="text-sm text-muted-foreground">
                {formatDate(entry.date)}
              </p>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-lg font-semibold"
                >€{entry.amount.toFixed(2)}</span
              >
              <Button
                size="icon"
                variant="ghost"
                onclick={() => onEdit(entry.id)}
              >
                <Edit class="h-4 w-4" />
              </Button>
              <Button
                size="icon"
                variant="ghost"
                onclick={() => onDelete(entry.id)}
              >
                <Trash2 class="h-4 w-4" />
              </Button>
            </div>
          </div>
          <div class="flex items-center justify-between text-sm">
            <span class="px-2 py-1 bg-secondary rounded-md"
              >{entry.category}</span
            >
            {#if entry.matched}
              <span class="text-green-600 dark:text-green-400 font-medium"
                >✓ Abgeglichen</span
              >
            {:else}
              <span class="text-muted-foreground">Nicht abgeglichen</span>
            {/if}
          </div>
        </div>
      {/each}

      {#if entries.length === 0}
        <div class="text-center py-12 text-muted-foreground">
          <p>Keine manuellen Einträge vorhanden</p>
          <p class="text-sm mt-2">
            Fügen Sie Einträge von Ihrem Kontoauszug hinzu
          </p>
        </div>
      {/if}
    </div>
  </CardContent>
</Card>
