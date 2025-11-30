<script lang="ts">
  import TrendingUpIcon from "@lucide/svelte/icons/trending-up";
  import * as Chart from "$lib/components/ui/chart/index.js";
  import * as Card from "$lib/components/ui/card/index.js";
  import { PieChart, Text } from "layerchart";

  type Props = {
    categoryTotals: Record<string, number>;
    title?: string;
    description?: string;
  };

  let {
    categoryTotals,
    title = "Ausgaben nach Kategorie",
    description = "Monatliche Übersicht",
  }: Props = $props();

  // Konvertiere categoryTotals zu Chart-Daten
  const chartData = $derived(
    Object.entries(categoryTotals).map(([category, amount], index) => ({
      category,
      amount,
      color: `var(--chart-${(index % 5) + 1})`,
    }))
  );

  const chartConfig = $derived({
    amount: { label: "Betrag" },
    ...Object.fromEntries(
      Object.keys(categoryTotals).map((category, index) => [
        category.toLowerCase().replace(/\s+/g, ""),
        {
          label: category,
          color: `var(--chart-${(index % 5) + 1})`,
        },
      ])
    ),
  } satisfies Chart.ChartConfig);

  const totalAmount = $derived(
    chartData.reduce((acc, curr) => acc + curr.amount, 0)
  );
</script>

<Card.Root class="flex flex-col">
  <Card.Header class="items-center">
    <Card.Title>{title}</Card.Title>
    <Card.Description>{description}</Card.Description>
  </Card.Header>
  <Card.Content class="flex-1">
    {#if chartData.length > 0}
      <Chart.Container
        config={chartConfig}
        class="mx-auto aspect-square max-h-[250px]"
      >
        <PieChart
          data={chartData}
          key="category"
          value="amount"
          c="color"
          innerRadius={60}
          padding={28}
          props={{ pie: { motion: "tween" } }}
        >
          {#snippet aboveMarks()}
            <Text
              value="€{totalAmount.toFixed(2)}"
              textAnchor="middle"
              verticalAnchor="middle"
              class="fill-foreground text-3xl! font-bold"
              dy={3}
            />
            <Text
              value="Gesamt"
              textAnchor="middle"
              verticalAnchor="middle"
              class="fill-muted-foreground! text-muted-foreground"
              dy={22}
            />
          {/snippet}
          {#snippet tooltip()}
            <Chart.Tooltip hideLabel />
          {/snippet}
        </PieChart>
      </Chart.Container>
    {:else}
      <div
        class="flex items-center justify-center h-[250px] text-muted-foreground"
      >
        Keine Ausgabendaten vorhanden
      </div>
    {/if}
  </Card.Content>
  <Card.Footer class="flex-col gap-2 text-sm">
    <div class="flex items-center gap-2 font-medium leading-none">
      {#if totalAmount > 0}
        Gesamtausgaben: €{totalAmount.toFixed(2)}
        <TrendingUpIcon class="size-4" />
      {/if}
    </div>
    <div class="text-muted-foreground leading-none">
      Aufschlüsselung nach Kategorien
    </div>
  </Card.Footer>
</Card.Root>
