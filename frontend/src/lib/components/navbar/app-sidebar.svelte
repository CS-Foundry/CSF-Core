<script lang="ts" module>
  import {
    HandCoins,
    RepeatIcon,
    ChartPieIcon,
    LayoutDashboardIcon,
    PiggyBank,
    ReceiptText,
  } from "@lucide/svelte";

  // Data for FinanceVault app
  const data = {
    user: {
      name: "Max Mustermann",
      email: "max@example.com",
      avatar: "/avatars/user.jpg",
    },
    teams: [
      {
        name: "FinanceVault",
        plan: "Premium",
      },
    ],
    navMain: [
      {
        title: "Dashboard",
        url: "/",
        icon: LayoutDashboardIcon,
        isActive: true,
      },
      {
        title: "Expenses",
        url: "/expenses/monthly",
        icon: HandCoins,
      },
      {
        title: "Subscriptions",
        url: "/subscriptions",
        icon: RepeatIcon,
      },
      {
        title: "Budgets",
        url: "/budgets",
        icon: PiggyBank,
      },
      {
        title: "Invoices",
        url: "/invoices",
        icon: ReceiptText,
      },
      {
        title: "Reports",
        url: "/reports",
        icon: ChartPieIcon,
      },
    ],
  };
</script>

<script lang="ts">
  import NavMain from "./nav-main.svelte";
  import NavProjects from "./nav-projects.svelte";
  import NavUser from "./nav-user.svelte";
  import TeamSwitcher from "./team-switcher.svelte";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import type { ComponentProps } from "svelte";

  let {
    ref = $bindable(null),
    collapsible = "icon",
    ...restProps
  }: ComponentProps<typeof Sidebar.Root> = $props();
</script>

<Sidebar.Root {collapsible} {...restProps}>
  <Sidebar.Header>
    <TeamSwitcher />
  </Sidebar.Header>
  <Sidebar.Content>
    <NavMain items={data.navMain} />
  </Sidebar.Content>
  <Sidebar.Footer>
    <NavUser />
  </Sidebar.Footer>
  <Sidebar.Rail />
</Sidebar.Root>
