<script lang="ts">
    import { Activity, Shield, Zap, Infinity } from "@lucide/svelte";
</script>

<div class="hidden lg:flex lg:w-1/2 p-6">
    <div
        class="relative w-full rounded-3xl overflow-hidden flex flex-col justify-between p-10"
    >
        <img
            src="/gradients/Gradient-dark.webp"
            alt=""
            class="absolute inset-0 w-full h-full object-cover dark:hidden"
        />
        <img
            src="/gradients/Gradient-light.webp"
            alt=""
            class="absolute inset-0 w-full h-full object-cover hidden dark:block"
        />

        <div class="absolute inset-0 pointer-events-none overflow-hidden">
            <div
                class="absolute -top-20 left-1/3 w-px h-[140%] bg-white/5 rotate-12 blur-sm"
            ></div>
            <div
                class="absolute -top-20 left-1/2 w-px h-[140%] bg-white/5 rotate-12 blur-sm"
            ></div>
            <div
                class="absolute -top-20 right-1/4 w-px h-[140%] bg-white/5 rotate-12 blur-sm"
            ></div>
        </div>

        <img
            src="/logo/logo-csfx.svg"
            alt=""
            class="absolute -top-10 -right-16 w-96 opacity-[0.12] pointer-events-none select-none"
            aria-hidden="true"
        />

        <div class="relative z-10 flex items-center gap-3"></div>

        <div class="relative z-10">
            <p
                class="text-panel-foreground/50 text-xs uppercase tracking-widest mb-4 font-medium"
            >
                CSFX Hypervisor
            </p>
            <h2
                class="text-panel-foreground text-5xl font-bold leading-tight mb-5 tracking-tight"
            >
                Welcome<br />to CSFX
            </h2>
            <p
                class="text-panel-foreground/60 text-sm leading-relaxed max-w-xs mb-3"
            >
                Orchestrate and manage your entire cloud infrastructure from a
                single hypervisor platform.
            </p>
            <p class="text-panel-foreground/40 text-xs">
                Trusted by teams running mission-critical workloads
            </p>
        </div>

        <div class="relative z-10">
            <div
                class="relative bg-panel-foreground/10 border border-panel-foreground/10 backdrop-blur-sm p-6 rounded-2xl"
            >
                <div class="grid grid-cols-2 gap-5">
                    <div class="flex flex-col gap-2">
                        <Infinity class="size-4 text-panel-foreground/70" />
                        <span
                            class="text-panel-foreground text-xs font-semibold"
                            >Orchestration</span
                        >
                        <span
                            class="text-panel-foreground/50 text-xs leading-relaxed"
                            >Deploy and scale workloads across any cloud</span
                        >
                    </div>
                    <div class="flex flex-col gap-2">
                        <Zap class="size-4 text-panel-foreground/70" />
                        <span
                            class="text-panel-foreground text-xs font-semibold"
                            >Hypervisor Layer</span
                        >
                        <span
                            class="text-panel-foreground/50 text-xs leading-relaxed"
                            >Bare-metal performance with VM flexibility</span
                        >
                    </div>
                    <div class="flex flex-col gap-2">
                        <Shield class="size-4 text-panel-foreground/70" />
                        <span
                            class="text-panel-foreground text-xs font-semibold"
                            >Isolation</span
                        >
                        <span
                            class="text-panel-foreground/50 text-xs leading-relaxed"
                            >Secure tenancy with strict resource boundaries</span
                        >
                    </div>
                    <div class="flex flex-col gap-2">
                        <Activity class="size-4 text-panel-foreground/70" />
                        <span
                            class="text-panel-foreground text-xs font-semibold"
                            >Observability</span
                        >
                        <span
                            class="text-panel-foreground/50 text-xs leading-relaxed"
                            >Full-stack metrics, logs, and tracing built-in</span
                        >
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>
