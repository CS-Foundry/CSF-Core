<script lang="ts">
	import Icon from "@iconify/svelte";
	import type { HTMLAttributes } from "svelte/elements";

	let { class: className, ...restProps }: HTMLAttributes<HTMLSpanElement> = $props();
</script>

<Icon icon="tabler:bucket" class={className} {...restProps} />
