<script lang="ts">
  import * as Chart from "$lib/components/ui/chart/index.js";
  import * as Card from "$lib/components/ui/card/index.js";
  import { PieChart, Text } from "layerchart";
  import { Progress } from "$lib/components/ui/progress";
  import { TrendingUp, TrendingDown, AlertCircle } from "@lucide/svelte";
  import type { BudgetOverview } from "$lib/types";

  interface Props {
    overview: BudgetOverview | null;
  }

  let { overview }: Props = $props();

  // Nur zwei Farben: Ausgegeben und Verfügbar
  const spentColor = "hsl(221.2, 83.2%, 53.3%)"; // Blau für ausgegeben
  const availableColor = "hsl(240, 5.9%, 90%)"; // Grau für verfügbar

  let chartData = $derived.by(() => {
    if (!overview) return [];

    const spent = overview.total_spent;
    const remaining = Math.max(0, overview.remaining); // Nur positive Werte

    return [
      {
        label: "Ausgegeben",
        amount: spent,
        color: spentColor,
      },
      {
        label: "Verfügbar",
        amount: remaining,
        color: availableColor,
      },
    ];
  });

  let chartConfig = $derived({
    ausgegeben: {
      label: "Ausgegeben",
      color: spentColor,
    },
    verfügbar: {
      label: "Verfügbar",
      color: availableColor,
    },
  });

  let totalSpent = $derived(overview?.total_spent ?? 0);
  let totalBudget = $derived(overview?.budget.total_budget ?? 0);
  let percentageUsed = $derived(overview?.percentage_used ?? 0);
  let remaining = $derived(overview?.remaining ?? 0);

  function getStatusColor(percentage: number): string {
    if (percentage >= 100) return "text-red-600";
    if (percentage >= 80) return "text-yellow-600";
    return "text-green-600";
  }

  function getProgressColor(percentage: number): string {
    if (percentage >= 100) return "bg-red-600";
    if (percentage >= 80) return "bg-yellow-600";
    return "bg-green-600";
  }
</script>

{#if overview}
  <Card.Root>
    <Card.Header class="items-center pb-0">
      <Card.Title class="text-2xl">Budget-Verteilung</Card.Title>
      <Card.Description>
        {new Date(overview.budget.month + "-01").toLocaleDateString("de-DE", {
          month: "long",
          year: "numeric",
        })}
      </Card.Description>
    </Card.Header>
    <Card.Content class="pb-0">
      <!-- Radial Chart -->
      <Chart.Container
        config={chartConfig}
        class="mx-auto aspect-square w-full max-h-[500px]"
      >
        <PieChart
          data={chartData}
          key="label"
          value="amount"
          c="color"
          innerRadius={120}
          padding={50}
          range={[-90, 90]}
          props={{ pie: { sort: null } }}
          cornerRadius={8}
        >
          {#snippet aboveMarks()}
            <Text
              value={`€${totalSpent.toFixed(0)}`}
              textAnchor="middle"
              verticalAnchor="middle"
              class="fill-foreground text-5xl! font-bold"
              dy={-25}
            />
            <Text
              value="ausgegeben"
              textAnchor="middle"
              verticalAnchor="middle"
              class="fill-muted-foreground! text-muted-foreground text-lg!"
              dy={5}
            />
            <Text
              value={`von €${totalBudget.toFixed(0)}`}
              textAnchor="middle"
              verticalAnchor="middle"
              class="fill-muted-foreground! text-muted-foreground text-base!"
              dy={30}
            />
          {/snippet}
          {#snippet tooltip()}
            <Chart.Tooltip hideLabel />
          {/snippet}
        </PieChart>
      </Chart.Container>
      <!-- Total Budget Block -->
      <div class="p-6 border rounded-lg space-y-4 bg-muted/30">
        <div class="flex justify-between items-start">
          <div>
            <p class="text-sm text-muted-foreground">Gesamtbudget</p>
            <p class="text-3xl font-bold">
              €{overview.budget.total_budget.toFixed(2)}
            </p>
          </div>
          <div class="text-right">
            <p class="text-sm text-muted-foreground">Ausgegeben</p>
            <p
              class="text-2xl font-semibold {getStatusColor(
                overview.percentage_used
              )}"
            >
              €{overview.total_spent.toFixed(2)}
            </p>
          </div>
        </div>

        <Progress
          value={overview.percentage_used}
          class={getProgressColor(overview.percentage_used)}
        />

        <div class="flex justify-between items-center text-sm">
          <div class="flex items-center gap-2">
            {#if overview.remaining < 0}
              <TrendingDown class="h-4 w-4 text-red-600" />
              <span class="text-red-600 font-medium">
                Überschritten um €{Math.abs(overview.remaining).toFixed(2)}
              </span>
            {:else if overview.percentage_used >= 80}
              <AlertCircle class="h-4 w-4 text-yellow-600" />
              <span class="text-yellow-600 font-medium">
                Verbleibend: €{overview.remaining.toFixed(2)}
              </span>
            {:else}
              <TrendingUp class="h-4 w-4 text-green-600" />
              <span class="text-green-600 font-medium">
                Verbleibend: €{overview.remaining.toFixed(2)}
              </span>
            {/if}
          </div>
          <span class="text-muted-foreground"
            >{overview.percentage_used.toFixed(1)}% verwendet</span
          >
        </div>
      </div>
    </Card.Content>
  </Card.Root>
{:else}
  <Card.Root>
    <Card.Content class="py-12 flex items-center justify-center">
      <p class="text-center text-muted-foreground">
        Keine Budget-Daten verfügbar
      </p>
    </Card.Content>
  </Card.Root>
{/if}
