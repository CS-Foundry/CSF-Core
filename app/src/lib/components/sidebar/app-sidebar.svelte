<script lang="ts" module>
    import BotIcon from "@lucide/svelte/icons/bot";
    import ChartPieIcon from "@lucide/svelte/icons/chart-pie";
    import MapIcon from "@lucide/svelte/icons/map";
    import BookOpenIcon from "@lucide/svelte/icons/book-open";
    import {
        LayoutDashboard,
        Server,
        Layers,
        ShipWheel,
        Container,
        LaptopMinimal,
        Activity,
        Logs,
    } from "@lucide/svelte";
    import IconBucket from "./icon-bucket.svelte";

    const data = {
        navBottom: [
            {
                title: "Monitoring",
                url: "#",
                icon: Activity,
            },
            {
                title: "Logs",
                url: "#",
                icon: Logs,
            },
        ],
        navMain: [
            {
                title: "Dashboard",
                url: "/",
                icon: LayoutDashboard,
                isActive: true,
            },
            {
                title: "Nodes",
                url: "/nodes",
                icon: Server,
            },
            {
                title: "Resource groups",
                url: "#",
                icon: Layers,
            },
        ],
        projects: [
            {
                name: "S3 Buckets",
                url: "#",
                icon: IconBucket,
            },
            {
                name: "Kubernetes",
                url: "#",
                icon: ShipWheel,
            },
            {
                name: "Docker",
                url: "#",
                icon: Container,
            },
            {
                name: "Virtual Machines",
                url: "#",
                icon: LaptopMinimal,
            },
        ],
    };
</script>

<script lang="ts">
    import NavMain from "./nav-main.svelte";
    import NavProjects from "./nav-projects.svelte";
    import NavUser from "./nav-user.svelte";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import { useSidebar } from "$lib/components/ui/sidebar/context.svelte.js";
    import type { ComponentProps } from "svelte";

    let {
        ref = $bindable(null),
        collapsible = "icon",
        ...restProps
    }: ComponentProps<typeof Sidebar.Root> = $props();

    const sidebar = useSidebar();
</script>

<Sidebar.Root bind:ref {collapsible} {...restProps}>
    <Sidebar.Header>
        <button
            onclick={() => sidebar.toggle()}
            class="flex items-center gap-2 px-2 py-1 rounded-md hover:bg-sidebar-accent transition-colors w-full text-left cursor-pointer"
        >
            <img
                src="/logo/logo-csfx.svg"
                alt="CSFX"
                class="size-6 invert dark:invert-0 shrink-0"
            />
            <div
                class="flex flex-col leading-tight group-data-[collapsible=icon]:hidden"
            >
                <span class="font-semibold text-sm tracking-wide">CSFX</span>
                <span class="text-muted-foreground text-xs"
                    >Hypervisor v0.1</span
                >
            </div>
        </button>
    </Sidebar.Header>
    <Sidebar.Content>
        <NavMain items={data.navMain} />
        <Sidebar.Separator />
        <NavProjects projects={data.projects} />
        <NavMain items={data.navBottom} label="" class="mt-auto" />
    </Sidebar.Content>
    <Sidebar.Footer>
        <NavUser />
    </Sidebar.Footer>
    <Sidebar.Rail />
</Sidebar.Root>
