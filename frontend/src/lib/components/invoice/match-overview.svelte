<script lang="ts">
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { CheckCircle2, XCircle, ArrowRightLeft } from "@lucide/svelte";
  import type { InvoiceOverview } from "$lib/types";

  interface Props {
    overview: InvoiceOverview | null;
    onMatch: () => void;
  }

  let { overview, onMatch }: Props = $props();
</script>

<Card>
  <CardHeader>
    <CardTitle>Abgleich-Übersicht</CardTitle>
    <CardDescription>Status der Rechnungsprüfung</CardDescription>
  </CardHeader>
  <CardContent>
    {#if overview}
      <div class="space-y-4">
        <div class="grid grid-cols-2 gap-4">
          <div class="p-4 bg-muted rounded-lg">
            <p class="text-sm text-muted-foreground">Manuelle Einträge</p>
            <p class="text-2xl font-bold">{overview.total_manual}</p>
          </div>
          <div class="p-4 bg-muted rounded-lg">
            <p class="text-sm text-muted-foreground">Rechnungen</p>
            <p class="text-2xl font-bold">{overview.total_invoices}</p>
          </div>
        </div>

        <div class="grid grid-cols-3 gap-3 text-center">
          <div class="p-3 border rounded-lg">
            <CheckCircle2 class="h-5 w-5 mx-auto mb-1 text-green-600" />
            <p class="text-lg font-semibold">{overview.matched_count}</p>
            <p class="text-xs text-muted-foreground">Abgeglichen</p>
          </div>
          <div class="p-3 border rounded-lg">
            <XCircle class="h-5 w-5 mx-auto mb-1 text-yellow-600" />
            <p class="text-lg font-semibold">{overview.unmatched_manual}</p>
            <p class="text-xs text-muted-foreground">
              Nicht abgeglichen (Manuell)
            </p>
          </div>
          <div class="p-3 border rounded-lg">
            <XCircle class="h-5 w-5 mx-auto mb-1 text-red-600" />
            <p class="text-lg font-semibold">{overview.unmatched_invoices}</p>
            <p class="text-xs text-muted-foreground">
              Nicht abgeglichen (Rechnungen)
            </p>
          </div>
        </div>

        <Button class="w-full" onclick={onMatch}>
          <ArrowRightLeft class="h-4 w-4 mr-2" />
          Automatisch abgleichen
        </Button>
      </div>
    {:else}
      <p class="text-center text-muted-foreground py-6">
        Keine Daten für diesen Monat
      </p>
    {/if}
  </CardContent>
</Card>
