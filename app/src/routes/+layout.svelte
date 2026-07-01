<script lang="ts">
    import "./layout.css";
    import { browser } from "$app/environment";

    let { children } = $props();

    if (browser) {
        const mq = window.matchMedia("(prefers-color-scheme: dark)");
        const apply = (dark: boolean) =>
            document.documentElement.classList.toggle("dark", dark);
        apply(mq.matches);
        mq.addEventListener("change", (e) => apply(e.matches));
    }
</script>

<svelte:head>
    <link rel="icon" type="image/svg+xml" href="/logo/logo-csfx.svg" />
</svelte:head>

{@render children()}
