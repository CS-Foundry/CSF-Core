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

<div class="icon-activity-wrapper {className}" aria-label="activity" role="img">
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
        <path
            class="icon-activity-path"
            d="M22 12h-2.48a2 2 0 0 0-1.93 1.46l-2.35 8.36a.25.25 0 0 1-.48 0L9.24 2.18a.25.25 0 0 0-.48 0l-2.35 8.36A2 2 0 0 1 4.49 12H2"
        />
    </svg>
</div>

<style>
    :global(.icon-activity-wrapper) {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        overflow: visible;
    }

    :global(.icon-activity-path) {
        stroke-dasharray: 50;
        stroke-dashoffset: 0;
    }

    :global(a:hover .icon-activity-path),
    :global(button:hover .icon-activity-path) {
        animation: icon-activity-draw 0.6s ease-in-out forwards;
    }

    @keyframes icon-activity-draw {
        0% { stroke-dashoffset: 50; }
        15% { stroke-dashoffset: 50; }
        100% { stroke-dashoffset: 100; }
    }
</style>
