<script lang="ts">
	import * as Sidebar from "$lib/components/ui/sidebar/index.js";

	let {
		projects,
	}: {
		projects: {
			name: string;
			url: string;
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			icon: any;
			isActive?: boolean;
		}[];
	} = $props();
</script>

<Sidebar.Group>
	<Sidebar.GroupLabel class="group-data-[collapsible=icon]:hidden">Resources</Sidebar.GroupLabel>
	<Sidebar.Menu>
		{#each projects as item (item.name)}
			<Sidebar.MenuItem>
				<Sidebar.MenuButton isActive={item.isActive} tooltipContent={item.name}>
					{#snippet child({ props })}
						<a href={item.url} {...props}>
							<item.icon />
							<span>{item.name}</span>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		{/each}
	</Sidebar.Menu>
</Sidebar.Group>
