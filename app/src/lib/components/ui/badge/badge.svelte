<script lang="ts" module>
	import { cn, type WithElementRef } from "$lib/utils.js";
	import type { HTMLAnchorAttributes } from "svelte/elements";
	import { type VariantProps, tv } from "tailwind-variants";

	export const badgeVariants = tv({
		base: "inline-flex items-center justify-center gap-1 rounded-full border px-2 py-0.5 text-xs font-medium w-fit whitespace-nowrap shrink-0 [&>svg]:size-3 [&>svg]:pointer-events-none transition-colors overflow-hidden",
		variants: {
			variant: {
				default: "border-transparent bg-primary text-primary-foreground",
				secondary: "border-transparent bg-secondary text-secondary-foreground",
				outline: "border-border bg-background text-foreground",
				success: "border-green-500/20 bg-green-500/15 text-green-500",
				warning: "border-yellow-500/20 bg-yellow-500/15 text-yellow-500",
				destructive: "border-destructive/20 bg-destructive/15 text-destructive",
				pending: "border-blue-500/20 bg-blue-500/15 text-blue-500",
			},
		},
		defaultVariants: {
			variant: "default",
		},
	});

	export type BadgeVariant = VariantProps<typeof badgeVariants>["variant"];
</script>

<script lang="ts">
	import Loader2Icon from "@lucide/svelte/icons/loader-2";

	let {
		ref = $bindable(null),
		class: className,
		variant = "default",
		loading = false,
		href,
		children,
		...restProps
	}: WithElementRef<HTMLAnchorAttributes> & {
		variant?: BadgeVariant;
		loading?: boolean;
	} = $props();
</script>

<svelte:element
	this={href ? "a" : "span"}
	bind:this={ref}
	data-slot="badge"
	{href}
	class={cn(badgeVariants({ variant }), className)}
	{...restProps}
>
	{#if loading}
		<Loader2Icon class="animate-spin" />
	{/if}
	{@render children?.()}
</svelte:element>
