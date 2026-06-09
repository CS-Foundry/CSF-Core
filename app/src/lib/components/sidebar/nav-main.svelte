<script lang="ts">
	import * as Sidebar from "$lib/components/ui/sidebar/index.js";

	let {
		items,
		label = "Platform",
		class: className = "",
	}: {
		items: {
			title: string;
			url: string;
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			icon?: any;
			isActive?: boolean;
		}[];
		label?: string;
		class?: string;
	} = $props();
</script>

<Sidebar.Group class={className}>
	{#if label}
		<Sidebar.GroupLabel class="group-data-[collapsible=icon]:hidden">{label}</Sidebar.GroupLabel>
	{/if}
	<Sidebar.Menu>
		{#each items as item (item.title)}
			<Sidebar.MenuItem>
				<Sidebar.MenuButton isActive={item.isActive} tooltipContent={item.title}>
					{#snippet child({ props })}
						<a href={item.url} {...props}>
							{#if item.icon}
								<item.icon />
							{/if}
							<span>{item.title}</span>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		{/each}
	</Sidebar.Menu>
</Sidebar.Group>
