<script lang="ts">
  import { onMount } from "svelte";
  import { browser } from "$app/environment";
  import { authStore } from "$lib/stores/auth";
  import Expenses from "$lib/components/expenses/expenses.svelte";
  import ExpensesWithFilters from "$lib/components/expenses/expenses-with-filters.svelte";
  import {
    ExpenseService,
    type Expense,
    type CreateExpenseRequest,
  } from "$lib/services/expenses";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select";
  import { toast } from "svelte-sonner";

  let allExpenses: Expense[] = $state([]);
  let loading = $state(true);
  let token = $state("");

  // Filter states
  let selectedMonth = $state(new Date().getMonth() + 1); // 1-12
  let selectedYear = $state(new Date().getFullYear());
  let selectedCategory = $state("all");
  let amountSearch = $state("");

  // Dialog state
  let showAddDialog = $state(false);
  let showEditDialog = $state(false);
  let editingExpense: Expense | null = $state(null);

  // Form state
  let formData = $state({
    description: "",
    amount: 0,
    date: "",
    category: "Lebensmittel",
  });

  const categories = [
    "Lebensmittel",
    "Transport",
    "Unterhaltung",
    "Gesundheit",
    "Bildung",
    "Sonstiges",
  ];

  onMount(async () => {
    if (browser) {
      // Initial load will be handled by the reactive statement below
    }
  });

  // Reactive statement to load data when auth state is ready
  $effect(() => {
    if (browser && $authStore.isAuthenticated && $authStore.token) {
      token = $authStore.token;
      loadExpenses();
    } else if (browser && !$authStore.isAuthenticated) {
      // Reset data when user is not authenticated
      allExpenses = [];
      loading = false;
    }
  });

  async function loadExpenses() {
    try {
      loading = true;
      allExpenses = await ExpenseService.getExpenses(token);
    } catch (error) {
      console.error("Failed to load expenses:", error);
      toast.error("Fehler beim Laden der Ausgaben");
    } finally {
      loading = false;
    }
  }

  // Filter expenses for selected month/year and apply category/amount filters
  let expenses = $derived(
    allExpenses
      .filter((exp) => {
        const expDate = new Date(exp.date);
        const monthMatches = expDate.getMonth() + 1 === selectedMonth;
        const yearMatches = expDate.getFullYear() === selectedYear;
        const categoryMatches =
          selectedCategory === "all" || exp.category === selectedCategory;

        let amountMatches = true;
        if (amountSearch.trim()) {
          const searchAmount = parseFloat(amountSearch);
          if (!isNaN(searchAmount)) {
            const expAmount = parseFloat(exp.amount.toString());
            // Exact match or within 0.01 tolerance for floating point comparison
            amountMatches = Math.abs(expAmount - searchAmount) < 0.01;
          }
        }

        return monthMatches && yearMatches && categoryMatches && amountMatches;
      })
      .map((e) => ({
        id: parseInt(e.id.replace(/-/g, "").slice(0, 8), 16),
        date: new Date(e.date).toLocaleDateString("de-DE"),
        category: e.category,
        amount: parseFloat(e.amount.toString()),
        description: e.description,
      }))
  );

  // Filter handlers
  function handleMonthChange(month: number) {
    selectedMonth = month;
  }

  function handleYearChange(year: number) {
    selectedYear = year;
  }

  function handleCategoryChange(category: string) {
    selectedCategory = category;
  }

  function handleAmountSearchChange(amount: string) {
    amountSearch = amount;
  }

  function handleAddExpense() {
    const now = new Date();
    formData = {
      description: "",
      amount: 0,
      date: now.toISOString().slice(0, 16), // Format: YYYY-MM-DDTHH:MM for datetime-local
      category: "Lebensmittel",
    };
    showAddDialog = true;
  }

  async function submitAddExpense() {
    try {
      // Convert datetime-local format to backend format (YYYY-MM-DD HH:MM:SS)
      const dateForBackend = formData.date.replace("T", " ") + ":00";
      const request: CreateExpenseRequest = {
        description: formData.description,
        amount: formData.amount,
        date: dateForBackend,
        category: formData.category,
      };

      await ExpenseService.createExpense(request, token);
      showAddDialog = false;
      await loadExpenses();
      toast.success("Ausgabe erfolgreich erstellt");
    } catch (error) {
      console.error("Failed to create expense:", error);
      toast.error("Fehler beim Erstellen der Ausgabe");
    }
  }

  function handleEditExpense(id: number) {
    const expense = allExpenses[expenses.findIndex((e) => e.id === id)];
    if (expense) {
      editingExpense = expense;
      // Convert backend format (YYYY-MM-DD HH:MM:SS) to datetime-local format (YYYY-MM-DDTHH:MM)
      const dateForInput = expense.date.slice(0, 16).replace(" ", "T");
      formData = {
        description: expense.description,
        amount: parseFloat(expense.amount.toString()),
        date: dateForInput,
        category: expense.category,
      };
      showEditDialog = true;
    }
  }

  async function submitEditExpense() {
    if (!editingExpense) return;

    try {
      // Convert datetime-local format to backend format (YYYY-MM-DD HH:MM:SS)
      const dateForBackend = formData.date.replace("T", " ") + ":00";
      await ExpenseService.updateExpense(
        editingExpense.id,
        {
          description: formData.description,
          amount: formData.amount,
          date: dateForBackend,
          category: formData.category,
        },
        token
      );
      showEditDialog = false;
      await loadExpenses();
      toast.success("Ausgabe erfolgreich aktualisiert");
    } catch (error) {
      console.error("Failed to update expense:", error);
      toast.error("Fehler beim Aktualisieren der Ausgabe");
    }
  }

  async function handleDeleteExpense(id: number) {
    const expense = allExpenses[expenses.findIndex((e) => e.id === id)];
    if (!expense) return;

    if (!confirm("Möchten Sie diese Ausgabe wirklich löschen?")) return;

    try {
      await ExpenseService.deleteExpense(expense.id, token);
      await loadExpenses();
      toast.success("Ausgabe erfolgreich gelöscht");
    } catch (error) {
      console.error("Failed to delete expense:", error);
      toast.error("Fehler beim Löschen der Ausgabe");
    }
  }

  function handleUploadFile(type: "image" | "pdf") {
    toast.info(`${type === "image" ? "Bild" : "PDF"}-Upload kommt bald!`);
  }
</script>

<svelte:head>
  <title>Ausgaben Übersicht - FinanceVault</title>
</svelte:head>

<div class="container mx-auto py-6">
  {#if loading}
    <div class="flex items-center justify-center h-64">
      <p>Lade Ausgaben...</p>
    </div>
  {:else}
    <ExpensesWithFilters
      {expenses}
      {selectedMonth}
      {selectedYear}
      {selectedCategory}
      {amountSearch}
      onAddExpense={handleAddExpense}
      onUploadFile={handleUploadFile}
      onEditExpense={handleEditExpense}
      onDeleteExpense={handleDeleteExpense}
      onMonthChange={handleMonthChange}
      onYearChange={handleYearChange}
      onCategoryChange={handleCategoryChange}
      onAmountSearchChange={handleAmountSearchChange}
    />
  {/if}
</div>

<!-- Add Expense Dialog -->
<Dialog.Root bind:open={showAddDialog}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Neue Ausgabe hinzufügen</Dialog.Title>
    </Dialog.Header>
    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="description">Beschreibung</Label>
        <Input
          id="description"
          bind:value={formData.description}
          placeholder="z.B. Einkauf bei Rewe"
        />
      </div>
      <div class="grid gap-2">
        <Label for="amount">Betrag (€)</Label>
        <Input
          id="amount"
          type="number"
          step="0.01"
          bind:value={formData.amount}
          placeholder="0.00"
        />
      </div>
      <div class="grid gap-2">
        <Label for="date">Datum</Label>
        <Input id="date" type="datetime-local" bind:value={formData.date} />
      </div>
      <div class="grid gap-2">
        <Label for="category">Kategorie</Label>
        <select
          id="category"
          class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
          bind:value={formData.category}
        >
          {#each categories as category}
            <option value={category}>
              {category}
            </option>
          {/each}
        </select>
      </div>
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (showAddDialog = false)}>
        Abbrechen
      </Button>
      <Button onclick={submitAddExpense}>Speichern</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<!-- Edit Expense Dialog -->
<Dialog.Root bind:open={showEditDialog}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Ausgabe bearbeiten</Dialog.Title>
    </Dialog.Header>
    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="edit-description">Beschreibung</Label>
        <Input id="edit-description" bind:value={formData.description} />
      </div>
      <div class="grid gap-2">
        <Label for="edit-amount">Betrag (€)</Label>
        <Input
          id="edit-amount"
          type="number"
          step="0.01"
          bind:value={formData.amount}
        />
      </div>
      <div class="grid gap-2">
        <Label for="edit-date">Datum</Label>
        <Input
          id="edit-date"
          type="datetime-local"
          bind:value={formData.date}
        />
      </div>
      <div class="grid gap-2">
        <Label for="edit-category">Kategorie</Label>
        <select
          id="edit-category"
          class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
          bind:value={formData.category}
        >
          {#each categories as category}
            <option value={category}>
              {category}
            </option>
          {/each}
        </select>
      </div>
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (showEditDialog = false)}>
        Abbrechen
      </Button>
      <Button onclick={submitEditExpense}>Aktualisieren</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
