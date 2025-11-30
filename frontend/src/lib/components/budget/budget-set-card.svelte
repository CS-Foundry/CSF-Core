<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Plus, Minus } from "@lucide/svelte";
  import * as Chart from "$lib/components/ui/chart/index.js";
  import { PieChart, Text } from "layerchart";

  interface Props {
    totalBudget: number;
    categories: { category: string; allocated_amount: number }[];
    onUpdate: (
      totalBudget: number,
      categories: { category: string; allocated_amount: number }[]
    ) => void;
  }

  let {
    totalBudget = $bindable(0),
    categories = $bindable([]),
    onUpdate,
  }: Props = $props();

  // Demo-Daten
  const demoTotal = 2500;
  const demoCategories = [
    { category: "Lebensmittel", allocated_amount: 600 },
    { category: "Transport", allocated_amount: 300 },
    { category: "Unterhaltung", allocated_amount: 200 },
    { category: "Wohnung", allocated_amount: 800 },
    { category: "Gesundheit", allocated_amount: 150 },
    { category: "Sonstiges", allocated_amount: 450 },
  ];

  let localTotal = $state(totalBudget || demoTotal);
  let localCategories = $state(
    categories.length > 0 ? [...categories] : [...demoCategories]
  );

  // Chart colors
  const chartColors = [
    "hsl(var(--chart-1))",
    "hsl(var(--chart-2))",
    "hsl(var(--chart-3))",
    "hsl(var(--chart-4))",
    "hsl(var(--chart-5))",
    "hsl(220, 70%, 50%)",
    "hsl(280, 70%, 50%)",
    "hsl(340, 70%, 50%)",
  ];

  // Prepare chart data
  let chartData = $derived(
    localCategories
      .filter((cat) => cat.category && cat.allocated_amount > 0)
      .map((cat, index) => ({
        category: cat.category,
        amount: cat.allocated_amount,
        color: chartColors[index % chartColors.length],
      }))
  );

  let totalAllocated = $derived(
    localCategories.reduce((sum, cat) => sum + (cat.allocated_amount || 0), 0)
  );

  let chartConfig = $derived(
    localCategories.reduce(
      (config, cat, index) => {
        if (cat.category) {
          config[cat.category] = {
            label: cat.category,
            color: chartColors[index % chartColors.length],
          };
        }
        return config;
      },
      {} as Record<string, { label: string; color: string }>
    )
  );

  function addCategory() {
    localCategories = [
      ...localCategories,
      { category: "", allocated_amount: 0 },
    ];
  }

  function removeCategory(index: number) {
    localCategories = localCategories.filter((_, i) => i !== index);
  }

  function saveBudget() {
    onUpdate(localTotal, localCategories);
  }

  $effect(() => {
    x;
    localTotal = totalBudget;
    localCategories = [...categories];
  });
</script>

<Card class="w-full">
  <CardHeader>
    <CardTitle class="text-2xl">Budget festlegen</CardTitle>
    <CardDescription>
      Legen Sie Ihr monatliches Gesamtbudget fest und verteilen Sie es auf
      Kategorien
    </CardDescription>
  </CardHeader>
  <CardContent class="space-y-6">
    <!-- Budget Input -->
    <div class="space-y-2">
      <Label for="total-budget">Gesamtbudget (€)</Label>
      <Input
        id="total-budget"
        type="number"
        bind:value={localTotal}
        placeholder="0.00"
        step="0.01"
        class="text-2xl font-bold"
      />
    </div>

    <!-- Pie Chart -->
    {#if chartData.length > 0}
      <div class="py-4">
        <Chart.Container
          config={chartConfig}
          class="mx-auto aspect-square max-h-[300px]"
        >
          <PieChart
            data={chartData}
            key="category"
            value="amount"
            c="color"
            innerRadius={80}
            padding={10}
            props={{ pie: { sort: null } }}
            cornerRadius={4}
          >
            {#snippet aboveMarks()}
              <Text
                value={`€${totalAllocated.toFixed(0)}`}
                textAnchor="middle"
                verticalAnchor="middle"
                class="fill-foreground text-2xl! font-bold"
                dy={-12}
              />
              <Text
                value="Zugewiesen"
                textAnchor="middle"
                verticalAnchor="middle"
                class="fill-muted-foreground! text-muted-foreground text-sm!"
                dy={12}
              />
            {/snippet}
            {#snippet tooltip()}
              <Chart.Tooltip hideLabel />
            {/snippet}
          </PieChart>
        </Chart.Container>

        <!-- Legend -->
        {#if chartData.length > 0}
          <div class="flex flex-wrap justify-center gap-3 mt-4">
            {#each chartData as item}
              <div class="flex items-center gap-2 text-sm">
                <div
                  class="w-3 h-3 rounded-full"
                  style="background-color: {item.color}"
                ></div>
                <span class="font-medium">{item.category}</span>
                <span class="text-muted-foreground"
                  >€{item.amount.toFixed(2)}</span
                >
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Categories -->
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <Label class="text-lg">Kategorien</Label>
        <Button size="sm" variant="outline" onclick={addCategory}>
          <Plus class="h-4 w-4 mr-2" />
          Kategorie hinzufügen
        </Button>
      </div>

      {#each localCategories as category, i}
        <div class="flex gap-2 items-end">
          <div class="flex-1 space-y-2">
            <Label for="category-{i}">Kategorie</Label>
            <Input
              id="category-{i}"
              type="text"
              bind:value={category.category}
              placeholder="z.B. Lebensmittel"
            />
          </div>
          <div class="flex-1 space-y-2">
            <Label for="amount-{i}">Betrag (€)</Label>
            <Input
              id="amount-{i}"
              type="number"
              bind:value={category.allocated_amount}
              placeholder="0.00"
              step="0.01"
            />
          </div>
          <Button
            size="icon"
            variant="destructive"
            onclick={() => removeCategory(i)}
          >
            <Minus class="h-4 w-4" />
          </Button>
        </div>
      {/each}

      {#if localCategories.length === 0}
        <p class="text-sm text-muted-foreground text-center py-4">
          Noch keine Kategorien hinzugefügt
        </p>
      {/if}
    </div>

    <Button class="w-full" size="lg" onclick={saveBudget}>
      Budget speichern
    </Button>
  </CardContent>
</Card>
