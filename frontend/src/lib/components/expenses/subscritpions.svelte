<script lang="ts">
  import * as Card from "$lib/components/ui/card/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Calendar } from "$lib/components/ui/calendar/index.js";
  import {
    FlexRender,
    createSvelteTable,
  } from "$lib/components/ui/data-table/index.js";
  import { renderComponent } from "$lib/components/ui/data-table/render-helpers.js";
  import {
    getCoreRowModel,
    getSortedRowModel,
    type ColumnDef,
    type SortingState,
  } from "@tanstack/table-core";
  import { Plus } from "@lucide/svelte";
  import DataTableActions from "./data-table-actions.svelte";
  import BadgeCell from "./badge-cell.svelte";
  import type { CalendarDate } from "@internationalized/date";

  type Subscription = {
    id: number;
    name: string;
    amount: number;
    billingDay: number;
    category: string;
  };

  type Props = {
    subscriptions: Subscription[];
    selectedDate: CalendarDate;
    totalSubscriptions: number;
    onAddSubscription: () => void;
    onEditSubscription: (id: number) => void;
    onDeleteSubscription: (id: number) => void;
  };

  let {
    subscriptions,
    selectedDate,
    totalSubscriptions,
    onAddSubscription,
    onEditSubscription,
    onDeleteSubscription,
  }: Props = $props();

  // Table state
  let sorting = $state<SortingState>([]);

  // Subscription Columns
  const subscriptionColumns: ColumnDef<Subscription>[] = [
    {
      accessorKey: "name",
      header: "Name",
      cell: ({ row }) => row.getValue("name"),
    },
    {
      accessorKey: "category",
      header: "Kategorie",
      cell: ({ row }) => {
        const category = row.getValue("category") as string;
        return renderComponent(BadgeCell, {
          value: category,
          variant: "secondary",
        });
      },
    },
    {
      accessorKey: "billingDay",
      header: "Abbuchungstag",
      cell: ({ row }) => {
        const day = row.getValue("billingDay") as number;
        return renderComponent(BadgeCell, {
          value: `${day}. des Monats`,
          variant: "outline",
        });
      },
    },
    {
      accessorKey: "amount",
      header: "Betrag",
      cell: ({ row }) => {
        const amount = parseFloat(row.getValue("amount"));
        return `€${amount.toFixed(2)}`;
      },
    },
    {
      id: "actions",
      cell: ({ row }) => {
        return renderComponent(DataTableActions, {
          onEdit: () => onEditSubscription(row.original.id),
          onDelete: () => onDeleteSubscription(row.original.id),
        });
      },
    },
  ];

  const subscriptionTable = createSvelteTable({
    get data() {
      return subscriptions;
    },
    columns: subscriptionColumns,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    state: {
      get sorting() {
        return sorting;
      },
    },
    onSortingChange: (updater) => {
      if (updater instanceof Function) {
        sorting = updater(sorting);
      } else {
        sorting = updater;
      }
    },
  });
</script>

<div class="grid grid-cols-[300px_1fr] gap-4 h-full">
  <!-- Kalender -->
  <Card.Root class="h-fit">
    <Card.Header>
      <Card.Title>Abbuchungskalender</Card.Title>
    </Card.Header>
    <Card.Content>
      <Calendar bind:value={selectedDate} class="rounded-md border" />
      <div class="mt-4 space-y-2">
        <p class="text-sm font-medium">Legende:</p>
        <div class="flex items-center gap-2 text-xs text-muted-foreground">
          <div class="h-3 w-3 rounded-full bg-primary"></div>
          <span>Abbuchungstage</span>
        </div>
      </div>
    </Card.Content>
  </Card.Root>

  <!-- Subscriptions Tabelle -->
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <h2 class="text-2xl font-bold tracking-tight">Meine Abonnements</h2>
      <Button size="sm" onclick={onAddSubscription}>
        <Plus class="mr-2 h-4 w-4" />
        Neues Abo
      </Button>
    </div>

    <div class="rounded-md border">
      <table class="w-full">
        <thead class="border-b">
          {#each subscriptionTable.getHeaderGroups() as headerGroup}
            <tr>
              {#each headerGroup.headers as header}
                <th
                  class="h-12 px-4 text-left align-middle font-medium text-muted-foreground"
                >
                  {#if !header.isPlaceholder}
                    <FlexRender
                      content={header.column.columnDef.header}
                      context={header.getContext()}
                    />
                  {/if}
                </th>
              {/each}
            </tr>
          {/each}
        </thead>
        <tbody>
          {#each subscriptionTable.getRowModel().rows as row}
            <tr class="border-b transition-colors hover:bg-muted/50">
              {#each row.getVisibleCells() as cell}
                <td class="p-4 align-middle">
                  <FlexRender
                    content={cell.column.columnDef.cell}
                    context={cell.getContext()}
                  />
                </td>
              {/each}
            </tr>
          {:else}
            <tr>
              <td colspan={subscriptionColumns.length} class="h-24 text-center">
                Keine Abonnements vorhanden.
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div class="pt-4 border-t">
      <div class="flex justify-between items-center">
        <span class="text-sm font-medium">Monatliche Gesamtsumme:</span>
        <span class="text-lg font-bold">€{totalSubscriptions.toFixed(2)}</span>
      </div>
    </div>
  </div>
</div>
