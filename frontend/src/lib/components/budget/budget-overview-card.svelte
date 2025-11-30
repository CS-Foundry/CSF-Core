<script lang="ts">
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Progress } from "$lib/components/ui/progress";
  import { TrendingUp, TrendingDown, AlertCircle } from "@lucide/svelte";
  import type { BudgetOverview } from "$lib/types";

  interface Props {
    overview: BudgetOverview | null;
  }

  let { overview }: Props = $props();

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
  <Card class="w-full">
    <CardHeader>
      <CardTitle class="text-2xl">Budget-Übersicht</CardTitle>
      <CardDescription>
        {new Date(overview.budget.month + "-01").toLocaleDateString("de-DE", {
          month: "long",
          year: "numeric",
        })}
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-6">
      <!-- Category Breakdown -->
      <div class="space-y-4">
        <h3 class="text-lg font-semibold">Kategorien</h3>
        {#each overview.categories as category}
          <div class="space-y-2">
            <div class="flex justify-between text-sm">
              <span class="font-medium">{category.category}</span>
              <span class={getStatusColor(category.percentage_used)}>
                €{category.spent.toFixed(2)} / €{category.allocated.toFixed(2)}
              </span>
            </div>
            <Progress
              value={category.percentage_used}
              class={getProgressColor(category.percentage_used)}
            />
            <div class="flex justify-between text-xs text-muted-foreground">
              <span>
                {#if category.remaining < 0}
                  Überschritten um €{Math.abs(category.remaining).toFixed(2)}
                {:else}
                  Verbleibend: €{category.remaining.toFixed(2)}
                {/if}
              </span>
              <span>{category.percentage_used.toFixed(1)}%</span>
            </div>
          </div>
        {/each}
      </div>
    </CardContent>
  </Card>
{:else}
  <Card class="w-full">
    <CardContent class="py-12">
      <p class="text-center text-muted-foreground">
        Für diesen Monat ist noch kein Budget festgelegt
      </p>
    </CardContent>
  </Card>
{/if}
