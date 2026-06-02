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

<div class="icon-rg-wrapper {className}" aria-label="resource-groups" role="img">
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
        <path d="M7 2h10" class="icon-rg-line-1" />
        <path d="M5 6h14" class="icon-rg-line-2" />
        <rect width="18" height="12" x="3" y="10" rx="2" />
    </svg>
</div>

<style>
    :global(.icon-rg-wrapper) {
        display: inline-flex;
        align-items: center;
        justify-content: center;
    }

    :global(.icon-rg-line-1),
    :global(.icon-rg-line-2) {
        transform-origin: center;
        opacity: 1;
        transform: translateY(0);
    }

    :global(a:hover .icon-rg-line-1),
    :global(button:hover .icon-rg-line-1) {
        animation: icon-rg-appear1 0.6s forwards;
    }

    :global(a:hover .icon-rg-line-2),
    :global(button:hover .icon-rg-line-2) {
        animation: icon-rg-appear2 0.6s forwards;
    }

    @keyframes icon-rg-appear1 {
        0%   { opacity: 0; transform: translateY(3px); }
        60%  { opacity: 0; transform: translateY(3px); }
        80%  { opacity: 1; transform: translateY(0); }
        100% { opacity: 1; transform: translateY(0); }
    }

    @keyframes icon-rg-appear2 {
        0%   { opacity: 0; transform: translateY(3px); }
        40%  { opacity: 0; transform: translateY(3px); }
        60%  { opacity: 1; transform: translateY(0); }
        100% { opacity: 1; transform: translateY(0); }
    }
</style>
