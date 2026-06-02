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

<div class="icon-ship-wrapper {className}" aria-label="docker" role="img">
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
        class="icon-ship-svg"
    >
        <path
            d="M2 21c.6.5 1.2 1 2.5 1 2.5 0 2.5-2 5-2 1.3 0 1.9.5 2.5 1 .6.5 1.2 1 2.5 1 2.5 0 2.5-2 5-2 1.3 0 1.9.5 2.5 1"
            class="icon-ship-wave"
        />
        <g class="icon-ship-body">
            <path d="M19.38 20A11.6 11.6 0 0 0 21 14l-9-4-9 4c0 2.9.94 5.34 2.81 7.76" />
            <path d="M19 13V7a2 2 0 0 0-2-2H7a2 2 0 0 0-2 2v6" />
            <path d="M12 10v4" />
            <path d="M12 2v3" />
        </g>
    </svg>
</div>

<style>
    :global(.icon-ship-wrapper) {
        display: inline-flex;
        align-items: center;
        justify-content: center;
    }

    :global(.icon-ship-svg) {
        overflow: visible;
    }

    :global(.icon-ship-wave) {
        stroke-dasharray: 30;
        stroke-dashoffset: 30;
        opacity: 0;
        transition: none;
    }

    :global(a:hover .icon-ship-wave),
    :global(button:hover .icon-ship-wave) {
        animation: icon-ship-wave-in 0.6s ease-in-out forwards;
    }

    :global(.icon-ship-body) {
        transform-origin: 12px 13px;
    }

    :global(a:hover .icon-ship-body),
    :global(button:hover .icon-ship-body) {
        animation: icon-ship-rock 2.4s ease-in-out infinite;
    }

    @keyframes icon-ship-wave-in {
        0%   { stroke-dashoffset: 30; opacity: 0; }
        100% { stroke-dashoffset: 0;  opacity: 1; }
    }

    @keyframes icon-ship-rock {
        0%, 100% { transform: rotate(-3deg); }
        50%       { transform: rotate(3deg); }
    }
</style>
