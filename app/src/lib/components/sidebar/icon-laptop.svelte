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

<div class="icon-vm-wrapper {className}" aria-label="virtual-machines" role="img">
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
        <path d="M22 13v2a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h7" />
        <path d="M8 21h8" />
        <path d="M12 17v4" />
        <g class="icon-vm-cog">
            <path d="m15.2 4.9-.9-.4" />
            <path d="m15.2 7.1-.9.4" />
            <path d="m16.9 3.2-.4-.9" />
            <path d="m16.9 8.8-.4.9" />
            <path d="m19.5 2.3-.4.9" />
            <path d="m19.5 9.7-.4-.9" />
            <path d="m21.7 4.5-.9.4" />
            <path d="m21.7 7.5-.9-.4" />
            <circle cx="18" cy="6" r="3" />
        </g>
    </svg>
</div>

<style>
    :global(.icon-vm-wrapper) {
        display: inline-flex;
        align-items: center;
        justify-content: center;
    }

    :global(.icon-vm-cog) {
        transform-origin: 18px 6px;
        transition: transform 0.8s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    }

    :global(a:hover .icon-vm-cog),
    :global(button:hover .icon-vm-cog) {
        transform: rotate(180deg);
    }
</style>
