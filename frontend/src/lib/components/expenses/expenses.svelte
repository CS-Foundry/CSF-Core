<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import {
    FlexRender,
    createSvelteTable,
  } from "$lib/components/ui/data-table/index.js";
  import { renderComponent } from "$lib/components/ui/data-table/render-helpers.js";
  import {
    getCoreRowModel,
    getSortedRowModel,
    getFilteredRowModel,
    type ColumnDef,
    type SortingState,
    type ColumnFiltersState,
  } from "@tanstack/table-core";
  import { Plus, FileText, Image } from "@lucide/svelte";
  import DataTableActions from "./data-table-actions.svelte";
  import BadgeCell from "./badge-cell.svelte";

  type Expense = {
    id: number;
    date: string;
    category: string;
    amount: number;
    description: string;
  };

  type Props = {
    expenses: Expense[];
    onAddExpense: () => void;
    onUploadFile: (type: "image" | "pdf") => void;
    onEditExpense: (id: number) => void;
    onDeleteExpense: (id: number) => void;
  };

  let {
    expenses,
    onAddExpense,
    onUploadFile,
    onEditExpense,
    onDeleteExpense,
  }: Props = $props();

  // Table state
  let sorting = $state<SortingState>([]);
  let columnFilters = $state<ColumnFiltersState>([]);

  // Expense Columns
  const expenseColumns: ColumnDef<Expense>[] = [
    {
      accessorKey: "date",
      header: "Datum",
      cell: ({ row }) => row.getValue("date"),
    },
    {
      accessorKey: "category",
      header: "Kategorie",
      cell: ({ row }) => {
        const category = row.getValue("category") as string;
        return renderComponent(BadgeCell, {
          value: category,
          variant: "outline",
        });
      },
    },
    {
      accessorKey: "description",
      header: "Beschreibung",
      cell: ({ row }) => row.getValue("description"),
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
          onEdit: () => onEditExpense(row.original.id),
          onDelete: () => onDeleteExpense(row.original.id),
        });
      },
    },
  ];

  // Create table
  const expenseTable = createSvelteTable({
    get data() {
      return expenses;
    },
    columns: expenseColumns,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    state: {
      get sorting() {
        return sorting;
      },
      get columnFilters() {
        return columnFilters;
      },
    },
    onSortingChange: (updater) => {
      if (updater instanceof Function) {
        sorting = updater(sorting);
      } else {
        sorting = updater;
      }
    },
    onColumnFiltersChange: (updater) => {
      if (updater instanceof Function) {
        columnFilters = updater(columnFilters);
      } else {
        columnFilters = updater;
      }
    },
  });
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <h2 class="text-2xl font-bold tracking-tight">Alle Ausgaben</h2>
    <div class="flex gap-2">
      <Button size="sm" variant="outline" onclick={() => onUploadFile("image")}>
        <Image class="mr-2 h-4 w-4" />
        Bild hochladen
      </Button>
      <Button size="sm" variant="outline" onclick={() => onUploadFile("pdf")}>
        <FileText class="mr-2 h-4 w-4" />
        PDF hochladen
      </Button>
      <Button size="sm" onclick={onAddExpense}>
        <Plus class="mr-2 h-4 w-4" />
        Neue Ausgabe
      </Button>
    </div>
  </div>

  <div class="rounded-md border">
    <table class="w-full">
      <thead class="border-b">
        {#each expenseTable.getHeaderGroups() as headerGroup}
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
        {#each expenseTable.getRowModel().rows as row}
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
            <td colspan={expenseColumns.length} class="h-24 text-center">
              Keine Ausgaben vorhanden.
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>
