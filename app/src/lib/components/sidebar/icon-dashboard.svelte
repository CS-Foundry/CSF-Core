<script lang="ts">
    interface Props {
        size?: number | string;
        color?: string;
        strokeWidth?: number | string;
        class?: string;
    }

    let {
        size = 16,
        color = 'currentColor',
        strokeWidth = 2,
        class: className = '',
    }: Props = $props();
</script>

<div class="icon-dashboard-wrapper {className}" aria-label="dashboard" role="img">
    <svg
        xmlns="http://www.w3.org/2000/svg"
        width={size}
        height={size}
        viewBox="0 0 24 24"
        fill="none"
        stroke={color}
        stroke-width={strokeWidth}
        stroke-linecap="round"
        stroke-linejoin="round"
    >
        <path d="m12 14 4-4" class="icon-dashboard-needle" />
        <path d="M3.34 19a10 10 0 1 1 17.32 0" />
    </svg>
</div>

<style>
    :global(.icon-dashboard-wrapper) {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        overflow: visible;
    }

    :global(.icon-dashboard-needle) {
        transform-origin: 12px 14px;
        transition: transform 0.6s cubic-bezier(0.16, 1.4, 0.3, 1);
    }

    :global(a:hover .icon-dashboard-needle),
    :global(button:hover .icon-dashboard-needle) {
        transform: rotate(72deg);
    }
</style>
