<script lang="ts">
  import * as Chart from "$lib/components/ui/chart/index.js";
  import * as Card from "$lib/components/ui/card/index.js";
  import { AreaChart } from "layerchart";
  import TrendingUpIcon from "@lucide/svelte/icons/trending-up";

  type Props = {
    expenses: Array<{
      id: number;
      date: string;
      category: string;
      amount: number;
      description: string;
    }>;
    title?: string;
    description?: string;
  };

  let {
    expenses,
    title = "Ausgaben Trend",
    description = "Verlauf der letzten 30 Tage",
  }: Props = $props();

  // Gruppiere Ausgaben nach Datum
  const dailyTotals = $derived(() => {
    const grouped = expenses.reduce(
      (acc, expense) => {
        const date = new Date(expense.date).toISOString().split("T")[0];
        acc[date] = (acc[date] || 0) + expense.amount;
        return acc;
      },
      {} as Record<string, number>
    );

    // Sortiere nach Datum und konvertiere zu Chart-Format
    return Object.entries(grouped)
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([date, amount]) => ({
        date: new Date(date).toLocaleDateString("de-DE", {
          month: "short",
          day: "numeric",
        }),
        amount,
        fullDate: date,
      }));
  });

  const chartConfig = {
    amount: {
      label: "Ausgaben",
      color: "var(--chart-1)",
    },
  } satisfies Chart.ChartConfig;

  const totalTrend = $derived(() => {
    const data = dailyTotals();
    return data.reduce((acc: number, curr) => acc + curr.amount, 0);
  });

  const averageDaily = $derived(() => {
    const data = dailyTotals();
    return data.length > 0 ? totalTrend() / data.length : 0;
  });
</script>

<Card.Root>
  <Card.Header>
    <Card.Title>{title}</Card.Title>
    <Card.Description>{description}</Card.Description>
  </Card.Header>
  <Card.Content>
    {#if dailyTotals().length > 0}
      <Chart.Container config={chartConfig} class="h-[200px]">
        <AreaChart
          data={dailyTotals()}
          x="date"
          y="amount"
          yNice
          padding={{ left: 12, right: 12, top: 12, bottom: 24 }}
        >
          {#snippet tooltip()}
            <Chart.Tooltip hideLabel />
          {/snippet}
        </AreaChart>
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
          Durchschnitt: €{averageDaily().toFixed(2)}/Tag
          <TrendingUpIcon class="size-4" />
        </div>
        <div class="flex items-center gap-2 leading-none text-muted-foreground">
          Gesamtbetrag: €{totalTrend().toFixed(2)}
        </div>
      </div>
    </div>
  </Card.Footer>
</Card.Root>
