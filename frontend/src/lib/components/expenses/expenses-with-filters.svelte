<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
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
  import { Plus, FileText, Image, Search, Filter } from "@lucide/svelte";
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
    selectedMonth: number;
    selectedYear: number;
    selectedCategory: string;
    amountSearch: string;
    onAddExpense: () => void;
    onUploadFile: (type: "image" | "pdf") => void;
    onEditExpense: (id: number) => void;
    onDeleteExpense: (id: number) => void;
    onMonthChange: (month: number) => void;
    onYearChange: (year: number) => void;
    onCategoryChange: (category: string) => void;
    onAmountSearchChange: (amount: string) => void;
  };

  let {
    expenses,
    selectedMonth,
    selectedYear,
    selectedCategory,
    amountSearch,
    onAddExpense,
    onUploadFile,
    onEditExpense,
    onDeleteExpense,
    onMonthChange,
    onYearChange,
    onCategoryChange,
    onAmountSearchChange,
  }: Props = $props();

  // Table state
  let sorting = $state<SortingState>([]);
  let columnFilters = $state<ColumnFiltersState>([]);

  // Categories for filtering
  const allCategories = [
    "Lebensmittel",
    "Transport",
    "Unterhaltung",
    "Gesundheit",
    "Bildung",
    "Sonstiges",
  ];

  // Months for selection
  const months = [
    { value: 1, label: "Januar" },
    { value: 2, label: "Februar" },
    { value: 3, label: "März" },
    { value: 4, label: "April" },
    { value: 5, label: "Mai" },
    { value: 6, label: "Juni" },
    { value: 7, label: "Juli" },
    { value: 8, label: "August" },
    { value: 9, label: "September" },
    { value: 10, label: "Oktober" },
    { value: 11, label: "November" },
    { value: 12, label: "Dezember" },
  ];

  // Generate years (current year ± 5 years)
  const currentYear = new Date().getFullYear();
  const years = Array.from({ length: 11 }, (_, i) => currentYear - 5 + i);

  // Get month name
  const getMonthName = (monthNum: number) => {
    return (
      months.find((m) => m.value === monthNum)?.label || `Monat ${monthNum}`
    );
  };

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
    <h2 class="text-2xl font-bold tracking-tight">
      Ausgaben {getMonthName(selectedMonth)}
      {selectedYear}
    </h2>
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

  <!-- Filter Section -->
  <div class="grid grid-cols-1 md:grid-cols-4 gap-4 p-4 bg-muted/50 rounded-lg">
    <!-- Month Selection -->
    <div class="space-y-2">
      <Label for="month-select">Monat</Label>
      <select
        id="month-select"
        class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
        value={selectedMonth}
        onchange={(e) =>
          onMonthChange(parseInt((e.target as HTMLSelectElement).value))}
      >
        {#each months as month}
          <option value={month.value}>
            {month.label}
          </option>
        {/each}
      </select>
    </div>

    <!-- Year Selection -->
    <div class="space-y-2">
      <Label for="year-select">Jahr</Label>
      <select
        id="year-select"
        class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
        value={selectedYear}
        onchange={(e) =>
          onYearChange(parseInt((e.target as HTMLSelectElement).value))}
      >
        {#each years as year}
          <option value={year}>
            {year}
          </option>
        {/each}
      </select>
    </div>

    <!-- Category Filter -->
    <div class="space-y-2">
      <Label for="category-filter">Kategorie</Label>
      <select
        id="category-filter"
        class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
        value={selectedCategory}
        onchange={(e) =>
          onCategoryChange((e.target as HTMLSelectElement).value)}
      >
        <option value="all">Alle Kategorien</option>
        {#each allCategories as category}
          <option value={category}>
            {category}
          </option>
        {/each}
      </select>
    </div>

    <!-- Amount Search -->
    <div class="space-y-2">
      <Label for="amount-search">Betrag suchen</Label>
      <div class="relative">
        <Search class="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
        <Input
          id="amount-search"
          type="number"
          step="0.01"
          placeholder="z.B. 15.99"
          class="pl-8"
          value={amountSearch}
          oninput={(e) =>
            onAmountSearchChange((e.target as HTMLInputElement).value)}
        />
      </div>
    </div>
  </div>

  <!-- Results Summary -->
  <div class="flex items-center justify-between text-sm text-muted-foreground">
    <div class="flex items-center gap-2">
      <Filter class="h-4 w-4" />
      <span>
        {expenses.length} Ausgabe{expenses.length !== 1 ? "n" : ""} gefunden
      </span>
    </div>
    <div>
      Gesamt: €{expenses.reduce((sum, exp) => sum + exp.amount, 0).toFixed(2)}
    </div>
  </div>

  <!-- Data Table -->
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
              {#if selectedCategory !== "all" || amountSearch.trim()}
                Keine Ausgaben für die gewählten Filter gefunden.
              {:else}
                Keine Ausgaben für {getMonthName(selectedMonth)}
                {selectedYear} vorhanden.
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>
