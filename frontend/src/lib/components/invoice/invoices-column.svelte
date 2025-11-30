<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import {
    Upload,
    Trash2,
    FileText,
    CheckCircle2,
    AlertCircle,
  } from "@lucide/svelte";
  import type { Invoice } from "$lib/types";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";

  interface Props {
    invoices: Invoice[];
    onUpload: () => void;
    onDelete: (id: string) => void;
  }

  let { invoices, onUpload, onDelete }: Props = $props();

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString("de-DE");
  }
</script>

<Card class="h-full flex flex-col">
  <CardHeader>
    <div class="flex items-center justify-between">
      <div>
        <CardTitle>Rechnungen & Belege</CardTitle>
        <CardDescription>Hochgeladene Dokumente zur Überprüfung</CardDescription
        >
      </div>
      <Button size="sm" onclick={onUpload}>
        <Upload class="h-4 w-4 mr-2" />
        Beleg hochladen
      </Button>
    </div>
  </CardHeader>
  <CardContent class="flex-1 overflow-auto">
    <div class="space-y-3">
      {#each invoices as invoice}
        <div
          class="p-4 border rounded-lg hover:bg-accent/50 transition-colors"
          class:border-green-500={invoice.verified}
          class:bg-green-50={invoice.verified}
          class:dark:bg-green-950={invoice.verified}
        >
          <div class="flex justify-between items-start mb-2">
            <div class="flex items-start gap-3 flex-1">
              <FileText class="h-5 w-5 mt-1 text-muted-foreground" />
              <div class="flex-1">
                <p class="font-medium">{invoice.description}</p>
                <p class="text-sm text-muted-foreground">
                  {formatDate(invoice.date)}
                </p>
                {#if invoice.file_url}
                  <a
                    href={invoice.file_url}
                    target="_blank"
                    class="text-xs text-blue-600 hover:underline"
                  >
                    Dokument ansehen
                  </a>
                {/if}
              </div>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-lg font-semibold"
                >€{invoice.amount.toFixed(2)}</span
              >
              <Button
                size="icon"
                variant="ghost"
                onclick={() => onDelete(invoice.id)}
              >
                <Trash2 class="h-4 w-4" />
              </Button>
            </div>
          </div>
          <div class="flex items-center justify-between text-sm">
            <span class="px-2 py-1 bg-secondary rounded-md"
              >{invoice.category}</span
            >
            {#if invoice.verified}
              <div
                class="flex items-center gap-1 text-green-600 dark:text-green-400"
              >
                <CheckCircle2 class="h-4 w-4" />
                <span class="font-medium">Verifiziert</span>
              </div>
            {:else}
              <div
                class="flex items-center gap-1 text-yellow-600 dark:text-yellow-400"
              >
                <AlertCircle class="h-4 w-4" />
                <span>Nicht verifiziert</span>
              </div>
            {/if}
          </div>
        </div>
      {/each}

      {#if invoices.length === 0}
        <div class="text-center py-12 text-muted-foreground">
          <p>Keine Belege vorhanden</p>
          <p class="text-sm mt-2">
            Laden Sie Rechnungen oder Belege zur Überprüfung hoch
          </p>
        </div>
      {/if}
    </div>
  </CardContent>
</Card>
