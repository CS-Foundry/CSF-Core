<script lang="ts">
  import * as Chart from "$lib/components/ui/chart/index.js";
  import * as Card from "$lib/components/ui/card/index.js";
  import { BarChart } from "layerchart";
  import TrendingUpIcon from "@lucide/svelte/icons/trending-up";

  type Props = {
    totalExpenses: number;
    totalSubscriptions: number;
    monthlyTotal: number;
    title?: string;
    description?: string;
  };

  let {
    totalExpenses,
    totalSubscriptions,
    monthlyTotal,
    title = "Ausgaben vs. Subscriptions",
    description = "Vergleich der Ausgabenarten",
  }: Props = $props();

  const chartData = $derived([
    {
      category: "Einmalig",
      amount: totalExpenses,
      color: "var(--chart-1)",
    },
    {
      category: "Subscriptions",
      amount: totalSubscriptions,
      color: "var(--chart-2)",
    },
  ]);

  const chartConfig = {
    amount: {
      label: "Betrag",
    },
    einmalig: {
      label: "Einmalige Ausgaben",
      color: "var(--chart-1)",
    },
    subscriptions: {
      label: "Subscriptions",
      color: "var(--chart-2)",
    },
  } satisfies Chart.ChartConfig;

  const savings = $derived(() => {
    if (totalExpenses === 0) return 0;
    const subscriptionYearly = totalSubscriptions * 12;
    const difference = totalExpenses - subscriptionYearly;
    return (difference / totalExpenses) * 100;
  });
</script>

<Card.Root>
  <Card.Header>
    <Card.Title>{title}</Card.Title>
    <Card.Description>{description}</Card.Description>
  </Card.Header>
  <Card.Content>
    {#if totalExpenses > 0 || totalSubscriptions > 0}
      <Chart.Container config={chartConfig} class="h-[200px]">
        <BarChart
          data={chartData}
          x="category"
          y="amount"
          yNice
          padding={{ left: 12, right: 12, top: 12, bottom: 24 }}
        >
          {#snippet tooltip()}
            <Chart.Tooltip hideLabel />
          {/snippet}
        </BarChart>
      </Chart.Container>
    {:else}
      <div
        class="flex items-center justify-center h-[200px] text-muted-foreground"
      >
        Keine Ausgabendaten vorhanden
      </div>
    {/if}
  </Card.Content>
  <Card.Footer>
    <div class="flex w-full items-start gap-2 text-sm">
      <div class="grid gap-2">
        <div class="flex items-center gap-2 font-medium leading-none">
          Gesamtsumme: €{monthlyTotal.toFixed(2)}
          <TrendingUpIcon class="size-4" />
        </div>
        <div class="flex items-center gap-2 leading-none text-muted-foreground">
          {#if savings() > 0}
            {savings().toFixed(1)}% mehr einmalige Ausgaben
          {:else if savings() < 0}
            {Math.abs(savings()).toFixed(1)}% mehr Subscriptions
          {:else}
            Ausgewogenes Verhältnis
          {/if}
        </div>
      </div>
    </div>
  </Card.Footer>
</Card.Root>
