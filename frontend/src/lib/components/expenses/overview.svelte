<script lang="ts">
  import * as Card from "$lib/components/ui/card/index.js";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import ExpensesType from "$lib/components/expenses/charts/expenses-type.svelte";
  import ExpensesTrend from "$lib/components/expenses/charts/expenses-trend.svelte";
  import ExpensesComparison from "$lib/components/expenses/charts/expenses-comparison.svelte";

  type Props = {
    totalExpenses: number;
    totalSubscriptions: number;
    monthlyTotal: number;
    categoryTotals: Record<string, number>;
    selectedMonth: number;
    selectedYear: number;
    expenses: Array<{
      id: number;
      date: string;
      category: string;
      amount: number;
      description: string;
    }>;
  };

  let {
    totalExpenses,
    totalSubscriptions,
    monthlyTotal,
    categoryTotals,
    selectedMonth,
    selectedYear,
    expenses,
  }: Props = $props();

  // Get month name
  const months = [
    "Januar",
    "Februar",
    "März",
    "April",
    "Mai",
    "Juni",
    "Juli",
    "August",
    "September",
    "Oktober",
    "November",
    "Dezember",
  ];

  const getMonthName = (monthNum: number) => {
    return months[monthNum - 1] || `Monat ${monthNum}`;
  };

  // Filtere nur einmalige Ausgaben für das Kategorie-Diagramm
  const expenseCategories = $derived(() => {
    const filtered = Object.fromEntries(
      Object.entries(categoryTotals).filter(([key]) => key !== "Subscriptions")
    );
    return Object.keys(filtered).length > 0 ? filtered : categoryTotals;
  });

  // Bereite Ausgaben für Trend-Chart vor (letzte 30 Tage)
  const recentExpenses = $derived(() => {
    const thirtyDaysAgo = new Date();
    thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30);

    return expenses
      .filter((expense) => new Date(expense.date) >= thirtyDaysAgo)
      .sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime());
  });
</script>

<div class="space-y-6">
  <div class="grid gap-4 md:grid-cols-3">
    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">Gesamtausgaben</Card.Title>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          class="h-4 w-4 text-muted-foreground"
        >
          <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
        </svg>
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">€{totalExpenses.toFixed(2)}</div>
        <p class="text-xs text-muted-foreground">
          {getMonthName(selectedMonth)}
          {selectedYear}
        </p>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">Subscriptions</Card.Title>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          class="h-4 w-4 text-muted-foreground"
        >
          <rect width="20" height="14" x="2" y="5" rx="2" />
          <path d="M2 10h20" />
        </svg>
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">
          €{totalSubscriptions.toFixed(2)}
        </div>
        <p class="text-xs text-muted-foreground">Monatliche Abos</p>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">Gesamtsumme</Card.Title>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          class="h-4 w-4 text-muted-foreground"
        >
          <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
          <circle cx="9" cy="7" r="4" />
          <path d="M22 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75" />
        </svg>
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">€{monthlyTotal.toFixed(2)}</div>
        <p class="text-xs text-muted-foreground">
          {getMonthName(selectedMonth)}
          {selectedYear}
        </p>
      </Card.Content>
    </Card.Root>
  </div>

  <div class="grid gap-4 md:grid-cols-2">
    <Card.Root>
      <Card.Header>
        <Card.Title>Ausgaben nach Kategorie</Card.Title>
      </Card.Header>
      <Card.Content>
        <div class="space-y-4">
          {#each Object.entries(categoryTotals) as [category, amount]}
            <div class="flex items-center">
              <div class="flex-1">
                <p class="text-sm font-medium">{category}</p>
                <div
                  class="mt-1 h-2 w-full bg-secondary rounded-full overflow-hidden"
                >
                  <div
                    class="h-full bg-primary transition-all"
                    style="width: {(amount / totalExpenses) * 100}%"
                  ></div>
                </div>
              </div>
              <div class="ml-4 text-sm font-medium">
                €{amount.toFixed(2)}
              </div>
            </div>
          {/each}
        </div>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header>
        <Card.Title>Neueste Ausgaben</Card.Title>
      </Card.Header>
      <Card.Content>
        <div class="space-y-3">
          {#each expenses.slice(0, 4) as expense}
            <div class="flex items-center justify-between">
              <div>
                <p class="text-sm font-medium">{expense.description}</p>
                <p class="text-xs text-muted-foreground">{expense.date}</p>
              </div>
              <Badge variant="outline">{expense.category}</Badge>
              <span class="text-sm font-medium"
                >€{expense.amount.toFixed(2)}</span
              >
            </div>
          {/each}
        </div>
      </Card.Content>
    </Card.Root>
  </div>

  <!-- Diagramm-Sektion -->
  <div class="grid gap-4 lg:grid-cols-3 md:grid-cols-2">
    <ExpensesType
      categoryTotals={expenseCategories()}
      title="Kategorieverteilung"
      description="Ausgaben nach Kategorien"
    />
    <ExpensesTrend
      expenses={recentExpenses()}
      title="30-Tage Trend"
      description="Ausgabenverlauf der letzten 30 Tage"
    />
    <ExpensesComparison
      {totalExpenses}
      {totalSubscriptions}
      {monthlyTotal}
      title="Ausgabenvergleich"
      description="Einmalig vs. Subscriptions"
    />
  </div>
</div>
