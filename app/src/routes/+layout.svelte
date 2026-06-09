<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { browser } from '$app/environment';

	let { children } = $props();

	if (browser) {
		const mq = window.matchMedia('(prefers-color-scheme: dark)');
		const apply = (dark: boolean) => document.documentElement.classList.toggle('dark', dark);
		apply(mq.matches);
		mq.addEventListener('change', (e) => apply(e.matches));
	}
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>
{@render children()}
