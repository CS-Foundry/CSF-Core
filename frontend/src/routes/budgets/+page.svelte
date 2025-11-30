<script lang="ts">
  import { onMount } from "svelte";
  import { browser } from "$app/environment";
  import { authStore } from "$lib/stores/auth";
  import { BudgetService } from "$lib/services/budget";
  import type { Budget, BudgetOverview } from "$lib/types";
  import { Button } from "$lib/components/ui/button";
  import { ChevronLeft, ChevronRight } from "@lucide/svelte";
  import { toast } from "svelte-sonner";
  import BudgetRadialChart from "$lib/components/budget/budget-radial-chart.svelte";
  import BudgetOverviewCard from "$lib/components/budget/budget-overview-card.svelte";

  let loading = $state(true);
  let selectedYear = $state(new Date().getFullYear());
  let selectedMonth = $state(new Date().getMonth() + 1); // 1-12
  let budgetOverview: BudgetOverview | null = $state(null);
  let currentBudget: Budget | null = $state(null);

  // Demo-Daten für November 2025
  const demoBudget: Budget = {
    id: "demo-budget-1",
    user_id: "demo-user",
    month: "2025-11",
    total_budget: 2500,
    categories: [
      { category: "Lebensmittel", allocated_amount: 600, spent_amount: 480 },
      { category: "Transport", allocated_amount: 300, spent_amount: 320 },
      { category: "Unterhaltung", allocated_amount: 200, spent_amount: 150 },
      { category: "Wohnung", allocated_amount: 800, spent_amount: 800 },
      { category: "Gesundheit", allocated_amount: 150, spent_amount: 95 },
      { category: "Sonstiges", allocated_amount: 450, spent_amount: 280 },
    ],
    created_at: "2025-11-01T00:00:00",
    updated_at: "2025-11-14T00:00:00",
  };

  const demoBudgetOverview: BudgetOverview = {
    budget: demoBudget,
    total_spent: 2125,
    remaining: 375,
    percentage_used: 85,
    categories: [
      {
        category: "Lebensmittel",
        allocated: 600,
        spent: 480,
        remaining: 120,
        percentage_used: 80,
      },
      {
        category: "Transport",
        allocated: 300,
        spent: 320,
        remaining: -20,
        percentage_used: 106.67,
      },
      {
        category: "Unterhaltung",
        allocated: 200,
        spent: 150,
        remaining: 50,
        percentage_used: 75,
      },
      {
        category: "Wohnung",
        allocated: 800,
        spent: 800,
        remaining: 0,
        percentage_used: 100,
      },
      {
        category: "Gesundheit",
        allocated: 150,
        spent: 95,
        remaining: 55,
        percentage_used: 63.33,
      },
      {
        category: "Sonstiges",
        allocated: 450,
        spent: 280,
        remaining: 170,
        percentage_used: 62.22,
      },
    ],
  };

  const monthNames = [
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

  let currentMonthString = $derived(
    `${selectedYear}-${String(selectedMonth).padStart(2, "0")}`
  );

  onMount(async () => {
    if (browser && $authStore.isAuthenticated) {
      await loadBudgetData();
    }
  });

  $effect(() => {
    if (browser && $authStore.isAuthenticated && $authStore.token) {
      loadBudgetData();
    }
  });

  async function loadBudgetData() {
    const token = $authStore.token;
    if (!token) return;

    try {
      loading = true;

      // Demo-Modus: Zeige Demo-Daten für November 2025
      if (selectedYear === 2025 && selectedMonth === 11) {
        budgetOverview = demoBudgetOverview;
        currentBudget = demoBudget;
        loading = false;
        return;
      }

      budgetOverview = await BudgetService.getBudgetOverview(
        currentMonthString,
        token
      );
      currentBudget = budgetOverview.budget;
    } catch (error) {
      // Budget doesn't exist for this month yet
      budgetOverview = null;
      currentBudget = null;
    } finally {
      loading = false;
    }
  }

  function previousMonth() {
    if (selectedMonth === 1) {
      selectedMonth = 12;
      selectedYear--;
    } else {
      selectedMonth--;
    }
  }

  function nextMonth() {
    if (selectedMonth === 12) {
      selectedMonth = 1;
      selectedYear++;
    } else {
      selectedMonth++;
    }
  }

  function goToCurrentMonth() {
    const now = new Date();
    selectedYear = now.getFullYear();
    selectedMonth = now.getMonth() + 1;
  }

  async function handleBudgetUpdate(
    totalBudget: number,
    categories: { category: string; allocated_amount: number }[]
  ) {
    const token = $authStore.token;
    if (!token) return;

    try {
      if (currentBudget) {
        // Update existing budget
        await BudgetService.updateBudget(
          currentMonthString,
          {
            total_budget: totalBudget,
            categories: categories,
          },
          token
        );
        toast.success("Budget erfolgreich aktualisiert");
      } else {
        // Create new budget
        await BudgetService.createBudget(
          {
            month: currentMonthString,
            total_budget: totalBudget,
            categories: categories,
          },
          token
        );
        toast.success("Budget erfolgreich erstellt");
      }
      await loadBudgetData();
    } catch (error) {
      toast.error("Fehler beim Speichern des Budgets");
      console.error(error);
    }
  }
</script>

<div class="h-screen w-full p-6 overflow-auto">
  <div class="w-full mx-auto space-y-6 px-4">
    <!-- Header mit Monatsauswahl -->
    <div class="flex items-center justify-between">
      <h1 class="text-3xl font-bold">Budget-Verwaltung</h1>
      <div class="flex items-center gap-2">
        <Button variant="outline" size="icon" onclick={previousMonth}>
          <ChevronLeft class="h-4 w-4" />
        </Button>
        <Button
          variant="outline"
          onclick={goToCurrentMonth}
          class="min-w-[200px]"
        >
          {monthNames[selectedMonth - 1]}
          {selectedYear}
        </Button>
        <Button variant="outline" size="icon" onclick={nextMonth}>
          <ChevronRight class="h-4 w-4" />
        </Button>
      </div>
    </div>

    {#if loading}
      <div class="text-center py-12">
        <p class="text-muted-foreground">Lade Budget-Daten...</p>
      </div>
    {:else}
      <!-- Layout genau wie bei subscriptions -->
      <div class="grid grid-cols-[400px_1fr] gap-6">
        <!-- Budget Radial Chart - feste Breite -->
        <div>
          <BudgetRadialChart overview={budgetOverview} />
        </div>

        <!-- Budget Overview - restliche Breite -->
        <div>
          <BudgetOverviewCard overview={budgetOverview} />
        </div>
      </div>
    {/if}
  </div>
</div>
