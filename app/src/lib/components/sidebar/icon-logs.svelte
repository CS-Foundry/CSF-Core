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

<div class="icon-logs-wrapper {className}" aria-label="logs" role="img">
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
        <rect width="8" height="4" x="8" y="2" rx="1" ry="1" class="icon-logs-clip" />
        <path
            d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"
            class="icon-logs-board"
        />
        <path d="M12 11h4" class="icon-logs-item-1 icon-logs-item" />
        <path d="M12 16h4" class="icon-logs-item-2 icon-logs-item" />
        <path d="M8 11h.01" class="icon-logs-bullet icon-logs-bullet-1" />
        <path d="M8 16h.01" class="icon-logs-bullet icon-logs-bullet-2" />
    </svg>
</div>

<style>
    :global(.icon-logs-wrapper) {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        overflow: visible;
    }

    :global(.icon-logs-item) {
        opacity: 1;
    }

    :global(.icon-logs-bullet) {
        opacity: 1;
    }

    :global(a:hover .icon-logs-item),
    :global(button:hover .icon-logs-item) {
        opacity: 0;
    }

    :global(a:hover .icon-logs-bullet),
    :global(button:hover .icon-logs-bullet) {
        opacity: 0;
    }

    :global(a:hover .icon-logs-clip),
    :global(button:hover .icon-logs-clip) {
        animation: icon-logs-clip-bounce 0.5s ease-in-out;
    }

    :global(a:hover .icon-logs-board),
    :global(button:hover .icon-logs-board) {
        animation: icon-logs-board-shake 0.5s ease-in-out;
    }

    :global(a:hover .icon-logs-item-1),
    :global(button:hover .icon-logs-item-1) {
        animation: icon-logs-line-in 0.2s ease-out forwards;
        animation-delay: 0.15s;
    }

    :global(a:hover .icon-logs-bullet-1),
    :global(button:hover .icon-logs-bullet-1) {
        animation: icon-logs-bullet-in 0.2s ease-out forwards;
        animation-delay: 0.2s;
    }

    :global(a:hover .icon-logs-item-2),
    :global(button:hover .icon-logs-item-2) {
        animation: icon-logs-line-in 0.2s ease-out forwards;
        animation-delay: 0.3s;
    }

    :global(a:hover .icon-logs-bullet-2),
    :global(button:hover .icon-logs-bullet-2) {
        animation: icon-logs-bullet-in 0.2s ease-out forwards;
        animation-delay: 0.35s;
    }

    @keyframes icon-logs-clip-bounce {
        0%   { transform: translateY(0); }
        25%  { transform: translateY(-2px); }
        50%  { transform: translateY(1px); }
        100% { transform: translateY(0); }
    }

    @keyframes icon-logs-board-shake {
        0%   { transform: rotate(0deg); }
        25%  { transform: rotate(-1deg); }
        75%  { transform: rotate(1deg); }
        100% { transform: rotate(0deg); }
    }

    @keyframes icon-logs-bullet-in {
        from { opacity: 0; transform: translateX(-6px) scale(0.8); }
        to   { opacity: 1; transform: translateX(0) scale(1); }
    }

    @keyframes icon-logs-line-in {
        from { opacity: 0; transform: translateX(-8px); }
        to   { opacity: 1; transform: translateX(0); }
    }
</style>
