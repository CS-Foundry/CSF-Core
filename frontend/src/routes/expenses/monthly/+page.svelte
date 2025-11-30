<script lang="ts">
  import { onMount } from "svelte";
  import { browser } from "$app/environment";
  import { authStore } from "$lib/stores/auth";
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import { CalendarDate } from "@internationalized/date";
  import Overview from "$lib/components/expenses/overview.svelte";
  import Expenses from "$lib/components/expenses/expenses.svelte";
  import ExpensesWithFilters from "$lib/components/expenses/expenses-with-filters.svelte";
  import Subscriptions from "$lib/components/expenses/subscritpions.svelte";
  import {
    ExpenseService,
    type Expense as ExpenseType,
  } from "$lib/services/expenses";
  import {
    SubscriptionService,
    type Subscription as SubscriptionType,
  } from "$lib/services/subscriptions";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select";
  import { toast } from "svelte-sonner";

  // Logging utility
  const logger = {
    info: (message: string, data?: any) => {
      console.log(`[MonthlyExpenses] ${message}`, data ? data : "");
    },
    warn: (message: string, data?: any) => {
      console.warn(`[MonthlyExpenses] ${message}`, data ? data : "");
    },
    error: (message: string, error?: any) => {
      console.error(`[MonthlyExpenses] ${message}`, error ? error : "");
    },
    debug: (message: string, data?: any) => {
      if (import.meta.env.DEV) {
        console.debug(`[MonthlyExpenses] ${message}`, data ? data : "");
      }
    },
  };

  type Expense = {
    id: number;
    date: string;
    category: string;
    amount: number;
    description: string;
  };

  type Subscription = {
    id: number;
    name: string;
    amount: number;
    billingDay: number;
    category: string;
  };

  let loading = $state(true);
  let allExpenses: ExpenseType[] = $state([]);
  let allSubscriptions: SubscriptionType[] = $state([]);

  // Filter states
  let selectedMonth = $state(new Date().getMonth() + 1); // 1-12
  let selectedYear = $state(new Date().getFullYear());
  let selectedCategory = $state("all");
  let amountSearch = $state("");

  // Dialog states
  let showAddExpenseDialog = $state(false);
  let showEditExpenseDialog = $state(false);
  let editingExpense: ExpenseType | null = $state(null);

  let showAddSubscriptionDialog = $state(false);
  let showEditSubscriptionDialog = $state(false);
  let editingSubscription: SubscriptionType | null = $state(null);

  // Form states
  let expenseFormData = $state({
    description: "",
    amount: 0,
    date: "",
    category: "Lebensmittel",
  });

  let subscriptionFormData = $state({
    name: "",
    amount: 0,
    billing_cycle: "monthly",
    next_billing_date: "",
    category: "Streaming",
  });

  const expenseCategories = [
    "Lebensmittel",
    "Transport",
    "Unterhaltung",
    "Gesundheit",
    "Bildung",
    "Sonstiges",
  ];

  const subscriptionCategories = [
    "Streaming",
    "Software",
    "Cloud Services",
    "Fitness",
    "Zeitschriften",
    "Sonstiges",
  ];

  const billingCycles = [
    { value: "monthly", label: "Monatlich" },
    { value: "yearly", label: "Jährlich" },
    { value: "quarterly", label: "Quartalsweise" },
    { value: "weekly", label: "Wöchentlich" },
  ];

  onMount(async () => {
    logger.info("Component mounted");
    if (browser) {
      logger.debug("Browser environment detected");
      // Initial load will be handled by the reactive statement below
    } else {
      logger.debug("SSR environment, skipping browser-specific code");
    }
  });

  // Reactive statement to load data when auth state is ready
  $effect(() => {
    if (browser && $authStore.isAuthenticated && $authStore.token) {
      logger.info("Auth state ready, loading data");
      loadData();
    } else if (browser && !$authStore.isAuthenticated) {
      logger.warn("User not authenticated");
      // Reset data when user is not authenticated
      allExpenses = [];
      allSubscriptions = [];
      loading = false;
    }
  });

  async function loadData() {
    const token = $authStore.token;
    if (!token) {
      logger.warn("No token available for data loading");
      return;
    }

    logger.info("Starting data load process");
    try {
      loading = true;
      const [expensesData, subscriptionsData] = await Promise.all([
        ExpenseService.getExpenses(token),
        SubscriptionService.getSubscriptions(token),
      ]);

      logger.info(
        `Data loaded successfully - Expenses: ${expensesData.length}, Subscriptions: ${subscriptionsData.length}`
      );
      allExpenses = expensesData;
      allSubscriptions = subscriptionsData;
    } catch (error) {
      logger.error("Failed to load data", error);
      toast.error("Fehler beim Laden der Daten");
    } finally {
      loading = false;
      logger.debug("Data loading process completed");
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

  // Transform subscriptions
  let subscriptions = $derived(
    allSubscriptions.map((s) => {
      const nextDate = new Date(s.next_billing_date);
      return {
        id: parseInt(s.id.replace(/-/g, "").slice(0, 8), 16),
        name: s.name,
        amount: parseFloat(s.amount.toString()),
        billingDay: nextDate.getDate(),
        category: s.category,
      };
    })
  );

  // Kalender-Konfiguration
  const now = new Date();
  let selectedDate = $state(
    new CalendarDate(now.getFullYear(), now.getMonth() + 1, now.getDate())
  );
  let markedDates = $derived(subscriptions.map((sub) => sub.billingDay));

  // Statistiken berechnen
  let totalExpenses = $derived(
    expenses.reduce((sum, exp) => sum + exp.amount, 0)
  );

  let totalSubscriptions = $derived(
    allSubscriptions.reduce((sum, s) => {
      const amount = parseFloat(s.amount.toString());
      switch (s.billing_cycle) {
        case "yearly":
          return sum + amount / 12;
        case "quarterly":
          return sum + amount / 3;
        case "weekly":
          return sum + amount * 4;
        default:
          return sum + amount;
      }
    }, 0)
  );

  let monthlyTotal = $derived(totalExpenses + totalSubscriptions);

  // Kategorien für Diagramm
  let categoryTotals = $derived(
    expenses.reduce(
      (acc, exp) => {
        acc[exp.category] = (acc[exp.category] || 0) + exp.amount;
        return acc;
      },
      {} as Record<string, number>
    )
  );

  // Expense handlers
  function handleAddExpense() {
    const now = new Date();
    expenseFormData = {
      description: "",
      amount: 0,
      date: now.toISOString().slice(0, 16), // Format: YYYY-MM-DDTHH:MM for datetime-local
      category: "Lebensmittel",
    };
    showAddExpenseDialog = true;
  }

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

  async function submitAddExpense() {
    logger.info("Submitting new expense", {
      amount: expenseFormData.amount,
      category: expenseFormData.category,
    });
    const token = $authStore.token;
    if (!token) {
      logger.error("No token available for expense submission");
      return;
    }

    try {
      // Convert datetime-local format to backend format (YYYY-MM-DD HH:MM:SS)
      const dateForBackend = expenseFormData.date.replace("T", " ") + ":00";
      const expenseData = {
        ...expenseFormData,
        date: dateForBackend,
      };

      logger.debug("Creating expense with data", {
        date: dateForBackend,
        description: expenseData.description,
      });
      await ExpenseService.createExpense(expenseData, token);
      logger.info("Expense created successfully");
      showAddExpenseDialog = false;
      await loadData();
      toast.success("Ausgabe erfolgreich erstellt");
    } catch (error) {
      logger.error("Failed to create expense", error);
      toast.error("Fehler beim Erstellen der Ausgabe");
    }
  }

  function handleEditExpense(id: number) {
    const expense = allExpenses[expenses.findIndex((e) => e.id === id)];
    if (expense) {
      editingExpense = expense;
      // Convert backend format (YYYY-MM-DD HH:MM:SS) to datetime-local format (YYYY-MM-DDTHH:MM)
      const dateForInput = expense.date.slice(0, 16).replace(" ", "T");
      expenseFormData = {
        description: expense.description,
        amount: parseFloat(expense.amount.toString()),
        date: dateForInput,
        category: expense.category,
      };
      showEditExpenseDialog = true;
    }
  }

  async function submitEditExpense() {
    if (!editingExpense) {
      logger.warn("No expense selected for editing");
      return;
    }

    logger.info("Submitting expense edit", {
      id: editingExpense.id,
      amount: expenseFormData.amount,
    });
    const token = $authStore.token;
    if (!token) {
      logger.error("No token available for expense edit");
      return;
    }

    try {
      // Convert datetime-local format to backend format (YYYY-MM-DD HH:MM:SS)
      const dateForBackend = expenseFormData.date.replace("T", " ") + ":00";
      const expenseData = {
        ...expenseFormData,
        date: dateForBackend,
      };

      logger.debug("Updating expense with data", {
        date: dateForBackend,
        description: expenseData.description,
      });
      await ExpenseService.updateExpense(editingExpense.id, expenseData, token);
      logger.info("Expense updated successfully", { id: editingExpense.id });
      showEditExpenseDialog = false;
      await loadData();
      toast.success("Ausgabe erfolgreich aktualisiert");
    } catch (error) {
      logger.error("Failed to update expense", error);
      toast.error("Fehler beim Aktualisieren der Ausgabe");
    }
  }

  async function handleDeleteExpense(id: number) {
    const expense = allExpenses[expenses.findIndex((e) => e.id === id)];
    if (!expense) {
      logger.warn("Expense not found for deletion", { id });
      return;
    }

    logger.info("Deleting expense", {
      id: expense.id,
      description: expense.description,
    });
    if (!confirm("Möchten Sie diese Ausgabe wirklich löschen?")) {
      logger.info("Expense deletion cancelled by user", { id: expense.id });
      return;
    }

    const token = $authStore.token;
    if (!token) {
      logger.error("No token available for expense deletion");
      return;
    }

    try {
      await ExpenseService.deleteExpense(expense.id, token);
      logger.info("Expense deleted successfully", { id: expense.id });
      await loadData();
      toast.success("Ausgabe erfolgreich gelöscht");
    } catch (error) {
      logger.error("Failed to delete expense", error);
      toast.error("Fehler beim Löschen der Ausgabe");
    }
  }

  function handleUploadFile(type: "image" | "pdf") {
    toast.info(`${type === "image" ? "Bild" : "PDF"}-Upload kommt bald!`);
  }

  // Subscription handlers
  function handleAddSubscription() {
    const now = new Date();
    subscriptionFormData = {
      name: "",
      amount: 0,
      billing_cycle: "monthly",
      next_billing_date: now.toISOString().slice(0, 16), // Format: YYYY-MM-DDTHH:MM
      category: "Streaming",
    };
    showAddSubscriptionDialog = true;
  }

  async function submitAddSubscription() {
    logger.info("Submitting new subscription", {
      name: subscriptionFormData.name,
      amount: subscriptionFormData.amount,
    });
    const token = $authStore.token;
    if (!token) {
      logger.error("No token available for subscription submission");
      return;
    }

    try {
      // Convert datetime-local format to backend format (YYYY-MM-DD HH:MM:SS)
      const dateForBackend =
        subscriptionFormData.next_billing_date.replace("T", " ") + ":00";
      const subscriptionData = {
        ...subscriptionFormData,
        next_billing_date: dateForBackend,
      };

      logger.debug("Creating subscription with data", {
        name: subscriptionData.name,
        nextBillingDate: dateForBackend,
      });
      await SubscriptionService.createSubscription(subscriptionData, token);
      logger.info("Subscription created successfully");
      showAddSubscriptionDialog = false;
      await loadData();
      toast.success("Abonnement erfolgreich erstellt");
    } catch (error) {
      logger.error("Failed to create subscription", error);
      toast.error("Fehler beim Erstellen des Abonnements");
    }
  }

  function handleEditSubscription(id: number) {
    const subscription =
      allSubscriptions[subscriptions.findIndex((s) => s.id === id)];
    if (subscription) {
      editingSubscription = subscription;
      // Convert backend format (YYYY-MM-DD HH:MM:SS) to datetime-local format (YYYY-MM-DDTHH:MM)
      const dateForInput = subscription.next_billing_date
        .slice(0, 16)
        .replace(" ", "T");
      subscriptionFormData = {
        name: subscription.name,
        amount: parseFloat(subscription.amount.toString()),
        billing_cycle: subscription.billing_cycle,
        next_billing_date: dateForInput,
        category: subscription.category,
      };
      showEditSubscriptionDialog = true;
    }
  }

  async function submitEditSubscription() {
    if (!editingSubscription) {
      logger.warn("No subscription selected for editing");
      return;
    }

    logger.info("Submitting subscription edit", {
      id: editingSubscription.id,
      name: subscriptionFormData.name,
    });
    const token = $authStore.token;
    if (!token) {
      logger.error("No token available for subscription edit");
      return;
    }

    try {
      // Convert datetime-local format to backend format (YYYY-MM-DD HH:MM:SS)
      const dateForBackend =
        subscriptionFormData.next_billing_date.replace("T", " ") + ":00";
      const subscriptionData = {
        ...subscriptionFormData,
        next_billing_date: dateForBackend,
      };

      logger.debug("Updating subscription with data", {
        name: subscriptionData.name,
        nextBillingDate: dateForBackend,
      });
      await SubscriptionService.updateSubscription(
        editingSubscription.id,
        subscriptionData,
        token
      );
      logger.info("Subscription updated successfully", {
        id: editingSubscription.id,
      });
      showEditSubscriptionDialog = false;
      await loadData();
      toast.success("Abonnement erfolgreich aktualisiert");
    } catch (error) {
      logger.error("Failed to update subscription", error);
      toast.error("Fehler beim Aktualisieren des Abonnements");
    }
  }

  async function handleDeleteSubscription(id: number) {
    const subscription =
      allSubscriptions[subscriptions.findIndex((s) => s.id === id)];
    if (!subscription) {
      logger.warn("Subscription not found for deletion", { id });
      return;
    }

    logger.info("Deleting subscription", {
      id: subscription.id,
      name: subscription.name,
    });
    if (!confirm("Möchten Sie dieses Abonnement wirklich löschen?")) {
      logger.info("Subscription deletion cancelled by user", {
        id: subscription.id,
      });
      return;
    }

    const token = $authStore.token;
    if (!token) {
      logger.error("No token available for subscription deletion");
      return;
    }

    try {
      await SubscriptionService.deleteSubscription(subscription.id, token);
      logger.info("Subscription deleted successfully", { id: subscription.id });
      await loadData();
      toast.success("Abonnement erfolgreich gelöscht");
    } catch (error) {
      logger.error("Failed to delete subscription", error);
      toast.error("Fehler beim Löschen des Abonnements");
    }
  }
</script>

<div class="h-screen w-full p-6 overflow-auto">
  <Tabs.Root value="overview" class="w-full h-full flex flex-col">
    <Tabs.List class="mb-4">
      <Tabs.Trigger value="overview">Overview</Tabs.Trigger>
      <Tabs.Trigger value="expenses">Expenses</Tabs.Trigger>
      <Tabs.Trigger value="subscription">Subscriptions</Tabs.Trigger>
    </Tabs.List>

    <!-- Overview Tab -->
    <Tabs.Content value="overview" class="flex-1">
      <Overview
        {totalExpenses}
        {totalSubscriptions}
        {monthlyTotal}
        {categoryTotals}
        {selectedMonth}
        {selectedYear}
        {expenses}
      />
    </Tabs.Content>

    <!-- Expenses Tab -->
    <Tabs.Content value="expenses" class="flex-1">
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
    </Tabs.Content>

    <!-- Subscriptions Tab -->
    <Tabs.Content value="subscription" class="flex-1">
      <Subscriptions
        {subscriptions}
        {selectedDate}
        {totalSubscriptions}
        onAddSubscription={handleAddSubscription}
        onEditSubscription={handleEditSubscription}
        onDeleteSubscription={handleDeleteSubscription}
      />
    </Tabs.Content>
  </Tabs.Root>
</div>

<!-- Add Expense Dialog -->
<Dialog.Root bind:open={showAddExpenseDialog}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Neue Ausgabe hinzufügen</Dialog.Title>
    </Dialog.Header>
    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="description">Beschreibung</Label>
        <Input
          id="description"
          bind:value={expenseFormData.description}
          placeholder="z.B. Einkauf bei Rewe"
        />
      </div>
      <div class="grid gap-2">
        <Label for="amount">Betrag (€)</Label>
        <Input
          id="amount"
          type="number"
          step="0.01"
          bind:value={expenseFormData.amount}
          placeholder="0.00"
        />
      </div>
      <div class="grid gap-2">
        <Label for="date">Datum</Label>
        <Input
          id="date"
          type="datetime-local"
          bind:value={expenseFormData.date}
        />
      </div>
      <div class="grid gap-2">
        <Label for="category">Kategorie</Label>
        <Select.Root>
          <Select.Trigger>
            <Select.Value placeholder="Kategorie wählen" />
          </Select.Trigger>
          <Select.Content>
            {#each expenseCategories as category}
              <Select.Item
                value={category}
                onclick={() => (expenseFormData.category = category)}
              >
                {category}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (showAddExpenseDialog = false)}>
        Abbrechen
      </Button>
      <Button onclick={submitAddExpense}>Speichern</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<!-- Edit Expense Dialog -->
<Dialog.Root bind:open={showEditExpenseDialog}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Ausgabe bearbeiten</Dialog.Title>
    </Dialog.Header>
    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="edit-description">Beschreibung</Label>
        <Input id="edit-description" bind:value={expenseFormData.description} />
      </div>
      <div class="grid gap-2">
        <Label for="edit-amount">Betrag (€)</Label>
        <Input
          id="edit-amount"
          type="number"
          step="0.01"
          bind:value={expenseFormData.amount}
        />
      </div>
      <div class="grid gap-2">
        <Label for="edit-date">Datum</Label>
        <Input
          id="edit-date"
          type="datetime-local"
          bind:value={expenseFormData.date}
        />
      </div>
      <div class="grid gap-2">
        <Label for="edit-category">Kategorie</Label>
        <Select.Root>
          <Select.Trigger>
            <Select.Value placeholder={expenseFormData.category} />
          </Select.Trigger>
          <Select.Content>
            {#each expenseCategories as category}
              <Select.Item
                value={category}
                onclick={() => (expenseFormData.category = category)}
              >
                {category}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (showEditExpenseDialog = false)}>
        Abbrechen
      </Button>
      <Button onclick={submitEditExpense}>Aktualisieren</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<!-- Add Subscription Dialog -->
<Dialog.Root bind:open={showAddSubscriptionDialog}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Neues Abonnement hinzufügen</Dialog.Title>
    </Dialog.Header>
    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="sub-name">Name</Label>
        <Input
          id="sub-name"
          bind:value={subscriptionFormData.name}
          placeholder="z.B. Netflix"
        />
      </div>
      <div class="grid gap-2">
        <Label for="sub-amount">Betrag (€)</Label>
        <Input
          id="sub-amount"
          type="number"
          step="0.01"
          bind:value={subscriptionFormData.amount}
          placeholder="0.00"
        />
      </div>
      <div class="grid gap-2">
        <Label for="sub-billing_cycle">Abrechnungszyklus</Label>
        <Select.Root>
          <Select.Trigger>
            <Select.Value placeholder="Abrechnungszyklus wählen" />
          </Select.Trigger>
          <Select.Content>
            {#each billingCycles as cycle}
              <Select.Item
                value={cycle.value}
                onclick={() =>
                  (subscriptionFormData.billing_cycle = cycle.value)}
              >
                {cycle.label}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
      <div class="grid gap-2">
        <Label for="sub-next_billing_date">Nächste Abbuchung</Label>
        <Input
          id="sub-next_billing_date"
          type="datetime-local"
          bind:value={subscriptionFormData.next_billing_date}
        />
      </div>
      <div class="grid gap-2">
        <Label for="sub-category">Kategorie</Label>
        <Select.Root>
          <Select.Trigger>
            <Select.Value placeholder="Kategorie wählen" />
          </Select.Trigger>
          <Select.Content>
            {#each subscriptionCategories as category}
              <Select.Item
                value={category}
                onclick={() => (subscriptionFormData.category = category)}
              >
                {category}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
    </div>
    <Dialog.Footer>
      <Button
        variant="outline"
        onclick={() => (showAddSubscriptionDialog = false)}
      >
        Abbrechen
      </Button>
      <Button onclick={submitAddSubscription}>Speichern</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<!-- Edit Subscription Dialog -->
<Dialog.Root bind:open={showEditSubscriptionDialog}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Abonnement bearbeiten</Dialog.Title>
    </Dialog.Header>
    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="edit-sub-name">Name</Label>
        <Input id="edit-sub-name" bind:value={subscriptionFormData.name} />
      </div>
      <div class="grid gap-2">
        <Label for="edit-sub-amount">Betrag (€)</Label>
        <Input
          id="edit-sub-amount"
          type="number"
          step="0.01"
          bind:value={subscriptionFormData.amount}
        />
      </div>
      <div class="grid gap-2">
        <Label for="edit-sub-billing_cycle">Abrechnungszyklus</Label>
        <Select.Root>
          <Select.Trigger>
            <Select.Value placeholder={subscriptionFormData.billing_cycle} />
          </Select.Trigger>
          <Select.Content>
            {#each billingCycles as cycle}
              <Select.Item
                value={cycle.value}
                onclick={() =>
                  (subscriptionFormData.billing_cycle = cycle.value)}
              >
                {cycle.label}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
      <div class="grid gap-2">
        <Label for="edit-sub-next_billing_date">Nächste Abbuchung</Label>
        <Input
          id="edit-sub-next_billing_date"
          type="datetime-local"
          bind:value={subscriptionFormData.next_billing_date}
        />
      </div>
      <div class="grid gap-2">
        <Label for="edit-sub-category">Kategorie</Label>
        <Select.Root>
          <Select.Trigger>
            <Select.Value placeholder={subscriptionFormData.category} />
          </Select.Trigger>
          <Select.Content>
            {#each subscriptionCategories as category}
              <Select.Item
                value={category}
                onclick={() => (subscriptionFormData.category = category)}
              >
                {category}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
    </div>
    <Dialog.Footer>
      <Button
        variant="outline"
        onclick={() => (showEditSubscriptionDialog = false)}
      >
        Abbrechen
      </Button>
      <Button onclick={submitEditSubscription}>Aktualisieren</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
