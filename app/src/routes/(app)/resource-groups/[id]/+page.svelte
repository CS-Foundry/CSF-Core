<script lang="ts">
    import { onMount, tick } from "svelte";
    import { page } from "$app/stores";
    import { goto } from "$app/navigation";
    import { auth } from "$lib/auth/store.svelte";
    import {
        getResourceGroup,
        listResourceGroupWorkloads,
        listResourceGroupVolumes,
        createWorkload,
        deleteWorkload,
        createVolume,
        deleteVolume,
        streamWorkloadLogs,
        openWorkloadExecSocket,
        type ResourceGroup,
        type Workload,
        type Volume,
        type PortMapping,
        type VolumeMount,
    } from "$lib/api/resource-groups";
    import type { Terminal } from "@xterm/xterm";
    import type { FitAddon } from "@xterm/addon-fit";
    import "@xterm/xterm/css/xterm.css";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import { Button } from "$lib/components/ui/button/index.js";

    const rgId: string = $page.params.id;

    let group = $state<ResourceGroup | null>(null);
    let workloads = $state<Workload[]>([]);
    let volumes = $state<Volume[]>([]);
    let loading = $state(true);
    let error = $state<string | null>(null);

    let activeTab = $state<"all" | "container" | "volume">("all");
    let filterText = $state("");

    let deployDialog = $state<HTMLDialogElement | null>(null);
    let volumeDialog = $state<HTMLDialogElement | null>(null);
    let deploying = $state(false);
    let creatingVolume = $state(false);
    let deployError = $state<string | null>(null);
    let volumeError = $state<string | null>(null);
    let downloadingVpn = $state(false);

    let formImage = $state("");
    let formName = $state("");
    let formCpu = $state("500");
    let formMemory = $state("512");
    let formDisk = $state("1024");
    let formEnv = $state("");
    let formPorts = $state("");
    let formVolumeMounts = $state("");

    let volFormName = $state("");
    let volFormSize = $state("10");

    let logsDialog = $state<HTMLDialogElement | null>(null);
    let logsWorkloadName = $state("");
    let logsLines = $state<string[]>([]);
    let logsError = $state<string | null>(null);
    let logsAbort: AbortController | null = null;

    let execDialog = $state<HTMLDialogElement | null>(null);
    let execWorkloadName = $state("");
    let execError = $state<string | null>(null);
    let execTerminalEl = $state<HTMLDivElement | null>(null);
    let execTerminal: Terminal | null = null;
    let execFitAddon: FitAddon | null = null;
    let execSocket: WebSocket | null = null;

    async function load() {
        if (!auth.token) return;
        try {
            [group, workloads, volumes] = await Promise.all([
                getResourceGroup(auth.token, rgId),
                listResourceGroupWorkloads(auth.token, rgId),
                listResourceGroupVolumes(auth.token, rgId),
            ]);
        } catch (e) {
            error = e instanceof Error ? e.message : "Failed to load";
        } finally {
            loading = false;
        }
    }

    function parseEnvVars(raw: string): Record<string, string> | null {
        if (!raw.trim()) return null;
        const result: Record<string, string> = {};
        for (const line of raw.trim().split("\n")) {
            const eq = line.indexOf("=");
            if (eq === -1) continue;
            result[line.slice(0, eq).trim()] = line.slice(eq + 1).trim();
        }
        return Object.keys(result).length ? result : null;
    }

    function parsePorts(raw: string): PortMapping[] | null {
        if (!raw.trim()) return null;
        const result: PortMapping[] = [];
        for (const part of raw.trim().split(",")) {
            const t = part.trim();
            const nodePort = t.match(/^(\d+):(\d+)(?:\/(tcp|udp))?$/);
            if (nodePort) {
                result.push({
                    container_port: parseInt(nodePort[2]),
                    protocol: nodePort[3] ?? null,
                    node_port: parseInt(nodePort[1]),
                });
                continue;
            }
            const internal = t.match(/^(\d+)(?:\/(tcp|udp))?$/);
            if (internal) {
                result.push({
                    container_port: parseInt(internal[1]),
                    protocol: internal[2] ?? null,
                    node_port: null,
                });
            }
        }
        return result.length ? result : null;
    }

    function parseVolumeMounts(raw: string): VolumeMount[] | null {
        if (!raw.trim()) return null;
        const result: VolumeMount[] = [];
        for (const line of raw.trim().split("\n")) {
            const parts = line.trim().split(":");
            if (parts.length < 2) continue;
            const volumeName = parts[0].trim();
            const mountPath = parts.slice(1).join(":").trim();
            const vol = volumes.find((v) => v.name === volumeName || v.id === volumeName);
            if (!vol || !mountPath) continue;
            result.push({ volume_id: vol.id, mount_path: mountPath });
        }
        return result.length ? result : null;
    }

    async function handleDeploy() {
        if (!auth.token || !formImage || !formName) return;
        deploying = true;
        deployError = null;
        try {
            await createWorkload(auth.token, {
                name: formName,
                image: formImage,
                cpu_millicores: parseInt(formCpu),
                memory_bytes: parseInt(formMemory) * 1024 * 1024,
                disk_bytes: parseInt(formDisk) * 1024 * 1024,
                env_vars: parseEnvVars(formEnv),
                ports: parsePorts(formPorts),
                volume_mounts: parseVolumeMounts(formVolumeMounts),
                resource_group_id: rgId,
            });
            deployDialog?.close();
            resetDeployForm();
            workloads = await listResourceGroupWorkloads(auth.token, rgId);
        } catch (e) {
            deployError = e instanceof Error ? e.message : "Failed to deploy";
        } finally {
            deploying = false;
        }
    }

    async function handleCreateVolume() {
        if (!auth.token || !volFormName) return;
        creatingVolume = true;
        volumeError = null;
        try {
            await createVolume(auth.token, {
                name: volFormName,
                size_gb: parseInt(volFormSize),
                resource_group_id: rgId,
            });
            volumeDialog?.close();
            volFormName = "";
            volFormSize = "10";
            volumes = await listResourceGroupVolumes(auth.token, rgId);
        } catch (e) {
            volumeError = e instanceof Error ? e.message : "Failed to create volume";
        } finally {
            creatingVolume = false;
        }
    }

    async function handleDeleteWorkload(id: string) {
        if (!auth.token) return;
        try {
            await deleteWorkload(auth.token, id);
            workloads = workloads.filter((w) => w.id !== id);
        } catch (e) {
            error = e instanceof Error ? e.message : "Failed to stop workload";
        }
    }

    async function openLogs(workload: Workload) {
        if (!auth.token) return;

        logsWorkloadName = workload.name;
        logsLines = [];
        logsError = null;
        logsDialog?.showModal();

        logsAbort = new AbortController();
        const decoder = new TextDecoder();

        try {
            const body = await streamWorkloadLogs(auth.token, workload.id, logsAbort.signal);
            const reader = body.getReader();

            while (true) {
                const { done, value } = await reader.read();
                if (done) break;
                const chunk = decoder.decode(value, { stream: true });
                if (chunk) {
                    logsLines = [...logsLines, ...chunk.split("\n").filter((l) => l.length > 0)];
                }
            }
        } catch (e) {
            if (e instanceof Error && e.name !== "AbortError") {
                logsError = e.message;
            }
        }
    }

    function closeLogs() {
        logsAbort?.abort();
        logsAbort = null;
        logsDialog?.close();
    }

    async function openExec(workload: Workload) {
        if (!auth.token) return;

        execWorkloadName = workload.name;
        execError = null;
        execDialog?.showModal();

        await tick();

        if (!execTerminalEl) return;

        const { Terminal } = await import("@xterm/xterm");
        const { FitAddon } = await import("@xterm/addon-fit");

        execTerminal = new Terminal({ convertEol: true, cursorBlink: true });
        execFitAddon = new FitAddon();
        execTerminal.loadAddon(execFitAddon);
        execTerminal.open(execTerminalEl);
        execFitAddon.fit();

        try {
            execSocket = await openWorkloadExecSocket(auth.token, workload.id);
            execSocket.binaryType = "arraybuffer";

            execSocket.onmessage = (event) => {
                if (execTerminal) {
                    const data =
                        event.data instanceof ArrayBuffer
                            ? new Uint8Array(event.data)
                            : event.data;
                    execTerminal.write(data);
                }
            };
            execSocket.onerror = () => {
                execError = "Exec socket error";
            };
            execSocket.onclose = () => {
                execTerminal?.write("\r\n[session closed]\r\n");
            };

            execTerminal.onData((data) => {
                execSocket?.send(data);
            });
        } catch (e) {
            execError = e instanceof Error ? e.message : "Failed to start exec session";
        }
    }

    function closeExec() {
        execSocket?.close();
        execSocket = null;
        execTerminal?.dispose();
        execTerminal = null;
        execFitAddon = null;
        execDialog?.close();
    }

    async function handleDeleteVolume(id: string) {
        if (!auth.token) return;
        try {
            await deleteVolume(auth.token, id);
            volumes = volumes.filter((v) => v.id !== id);
        } catch (e) {
            error = e instanceof Error ? e.message : "Failed to delete volume";
        }
    }

    async function downloadVpnConfig() {
        if (!auth.token) return;
        downloadingVpn = true;
        try {
            const resp = await fetch(`/api/resource-groups/${rgId}/vpn-config`, {
                headers: { Authorization: `Bearer ${auth.token}` },
            });
            if (!resp.ok) {
                const body = await resp.json().catch(() => ({}));
                throw new Error(body.error ?? `HTTP ${resp.status}`);
            }
            const blob = await resp.blob();
            const url = URL.createObjectURL(blob);
            const a = document.createElement("a");
            const cd = resp.headers.get("content-disposition") ?? "";
            const fn = cd.match(/filename="([^"]+)"/)?.[1] ?? `csfx-vpn.conf`;
            a.href = url;
            a.download = fn;
            a.click();
            URL.revokeObjectURL(url);
        } catch (e) {
            error = e instanceof Error ? e.message : "VPN config download failed";
        } finally {
            downloadingVpn = false;
        }
    }

    function resetDeployForm() {
        formImage = "";
        formName = "";
        formCpu = "500";
        formMemory = "512";
        formDisk = "1024";
        formEnv = "";
        formPorts = "";
        formVolumeMounts = "";
        deployError = null;
    }

    type ResourceItem =
        | { kind: "container"; data: Workload }
        | { kind: "volume"; data: Volume };

    let allResources = $derived<ResourceItem[]>([
        ...workloads.map((w): ResourceItem => ({ kind: "container", data: w })),
        ...volumes.map((v): ResourceItem => ({ kind: "volume", data: v })),
    ]);

    let filteredResources = $derived(
        allResources.filter((r) => {
            if (activeTab === "container" && r.kind !== "container") return false;
            if (activeTab === "volume" && r.kind !== "volume") return false;
            if (!filterText) return true;
            const q = filterText.toLowerCase();
            if (r.kind === "container") {
                return r.data.name.toLowerCase().includes(q) || r.data.image.toLowerCase().includes(q);
            }
            return r.data.name.toLowerCase().includes(q);
        })
    );

    function statusClass(status: string): string {
        switch (status.toLowerCase()) {
            case "running":
            case "active":
            case "available": return "bg-green-500/15 text-green-600 dark:text-green-400";
            case "failed":
            case "error": return "bg-red-500/15 text-red-600 dark:text-red-400";
            case "scheduled":
            case "attaching": return "bg-blue-500/15 text-blue-600 dark:text-blue-400";
            case "pulling": return "bg-blue-500/15 text-blue-600 dark:text-blue-400 animate-pulse";
            case "creating":
            case "starting": return "bg-yellow-500/15 text-yellow-600 dark:text-yellow-400 animate-pulse";
            case "stopped":
            case "deleting": return "bg-muted text-muted-foreground";
            default: return "bg-yellow-500/15 text-yellow-600 dark:text-yellow-400";
        }
    }

    function statusLabel(status: string): string {
        switch (status.toLowerCase()) {
            case "pulling": return "Pulling image...";
            case "creating": return "Creating container...";
            case "starting": return "Starting...";
            default: return status;
        }
    }

    function fmtBytes(bytes: number): string {
        if (bytes >= 1073741824) return `${(bytes / 1073741824).toFixed(1)} GB`;
        if (bytes >= 1048576) return `${(bytes / 1048576).toFixed(0)} MB`;
        return `${bytes} B`;
    }

    function initials(name: string): string {
        return name.slice(0, 2).toUpperCase();
    }

    let totalCpu = $derived(workloads.reduce((s, w) => s + w.cpu_millicores, 0));
    let totalMem = $derived(workloads.reduce((s, w) => s + w.memory_bytes, 0));
    let totalDisk = $derived(volumes.reduce((s, v) => s + v.size_gb, 0));

    onMount(load);
</script>

<dialog
    bind:this={deployDialog}
    class="fixed inset-0 z-50 m-auto w-full max-w-lg rounded-xl border bg-background shadow-xl p-0 backdrop:bg-black/40"
    onclose={() => resetDeployForm()}
>
    <div class="flex flex-col gap-5 p-6">
        <div class="flex items-center justify-between">
            <h2 class="text-base font-semibold">Deploy Container</h2>
            <button
                class="text-muted-foreground hover:text-foreground"
                onclick={() => deployDialog?.close()}
                aria-label="Close"
            >
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
        </div>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="d-name">Name</label>
                <input id="d-name" class="border rounded px-3 py-1.5 text-sm bg-background" placeholder="my-app" bind:value={formName} />
            </div>
            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="d-image">Image</label>
                <input id="d-image" class="border rounded px-3 py-1.5 text-sm bg-background font-mono" placeholder="nginx:latest" bind:value={formImage} />
            </div>
            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="d-cpu">CPU (millicores)</label>
                <input id="d-cpu" type="number" class="border rounded px-3 py-1.5 text-sm bg-background" placeholder="500" bind:value={formCpu} />
            </div>
            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="d-mem">Memory (MB)</label>
                <input id="d-mem" type="number" class="border rounded px-3 py-1.5 text-sm bg-background" placeholder="512" bind:value={formMemory} />
            </div>
            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="d-disk">Disk (MB)</label>
                <input id="d-disk" type="number" class="border rounded px-3 py-1.5 text-sm bg-background" placeholder="1024" bind:value={formDisk} />
            </div>
            <div class="flex flex-col gap-1 sm:col-span-2">
                <label class="text-xs text-muted-foreground" for="d-ports">
                    Ports — <span class="font-mono">nodePort:containerPort</span> to expose externally,
                    <span class="font-mono">containerPort</span> for internal mesh only
                </label>
                <input id="d-ports" class="border rounded px-3 py-1.5 text-sm bg-background font-mono" placeholder="8080:80, 443 (internal only)" bind:value={formPorts} />
            </div>
        </div>
        {#if volumes.length > 0}
            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="d-vols">
                    Volume Mounts (name-or-id:/mount/path, one per line)
                </label>
                <textarea
                    id="d-vols"
                    class="border rounded px-3 py-1.5 text-sm bg-background font-mono resize-none"
                    rows={Math.min(4, volumes.length + 1)}
                    placeholder={volumes.map(v => `${v.name}:/data`).slice(0, 2).join("\n")}
                    bind:value={formVolumeMounts}
                ></textarea>
                <p class="text-xs text-muted-foreground">Available: {volumes.map(v => v.name).join(", ")}</p>
            </div>
        {/if}
        <div class="flex flex-col gap-1">
            <label class="text-xs text-muted-foreground" for="d-env">Environment Variables (KEY=VALUE, one per line)</label>
            <textarea id="d-env" class="border rounded px-3 py-1.5 text-sm bg-background font-mono resize-none" rows={3} placeholder={"NODE_ENV=production\nPORT=3000"} bind:value={formEnv}></textarea>
        </div>
        {#if deployError}
            <p class="text-xs text-destructive">{deployError}</p>
        {/if}
        <div class="flex gap-2 justify-end">
            <Button size="sm" variant="outline" onclick={() => deployDialog?.close()}>Cancel</Button>
            <Button size="sm" onclick={handleDeploy} disabled={deploying || !formName || !formImage}>
                {deploying ? "Deploying..." : "Deploy"}
            </Button>
        </div>
    </div>
</dialog>

<dialog
    bind:this={volumeDialog}
    class="fixed inset-0 z-50 m-auto w-full max-w-sm rounded-xl border bg-background shadow-xl p-0 backdrop:bg-black/40"
    onclose={() => { volumeError = null; }}
>
    <div class="flex flex-col gap-5 p-6">
        <div class="flex items-center justify-between">
            <h2 class="text-base font-semibold">Create Volume</h2>
            <button
                class="text-muted-foreground hover:text-foreground"
                onclick={() => volumeDialog?.close()}
                aria-label="Close"
            >
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
        </div>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="v-name">Name</label>
                <input id="v-name" class="border rounded px-3 py-1.5 text-sm bg-background" placeholder="postgres-data" bind:value={volFormName} />
            </div>
            <div class="flex flex-col gap-1">
                <label class="text-xs text-muted-foreground" for="v-size">Size (GB)</label>
                <input id="v-size" type="number" class="border rounded px-3 py-1.5 text-sm bg-background" placeholder="10" bind:value={volFormSize} />
            </div>
        </div>
        {#if volumeError}
            <p class="text-xs text-destructive">{volumeError}</p>
        {/if}
        <div class="flex gap-2 justify-end">
            <Button size="sm" variant="outline" onclick={() => volumeDialog?.close()}>Cancel</Button>
            <Button size="sm" onclick={handleCreateVolume} disabled={creatingVolume || !volFormName}>
                {creatingVolume ? "Creating..." : "Create"}
            </Button>
        </div>
    </div>
</dialog>

<dialog
    bind:this={logsDialog}
    class="fixed inset-0 z-50 m-auto w-full max-w-3xl h-[70vh] rounded-xl border bg-background shadow-xl p-0 backdrop:bg-black/40"
    onclose={() => closeLogs()}
>
    <div class="flex flex-col h-full">
        <div class="flex items-center justify-between px-6 py-4 border-b shrink-0">
            <h2 class="text-base font-semibold font-mono">{logsWorkloadName}</h2>
            <button
                class="text-muted-foreground hover:text-foreground"
                onclick={() => logsDialog?.close()}
                aria-label="Close"
            >
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
        </div>
        <div class="flex-1 overflow-y-auto bg-black p-4 font-mono text-xs text-green-400">
            {#if logsError}
                <p class="text-red-400">{logsError}</p>
            {/if}
            {#each logsLines as line}
                <p class="whitespace-pre-wrap">{line}</p>
            {/each}
        </div>
    </div>
</dialog>

<dialog
    bind:this={execDialog}
    class="fixed inset-0 z-50 m-auto w-full max-w-4xl h-[80vh] rounded-xl border bg-background shadow-xl p-0 backdrop:bg-black/40"
    onclose={() => closeExec()}
>
    <div class="flex flex-col h-full">
        <div class="flex items-center justify-between px-6 py-4 border-b shrink-0">
            <h2 class="text-base font-semibold font-mono">{execWorkloadName}</h2>
            <button
                class="text-muted-foreground hover:text-foreground"
                onclick={() => execDialog?.close()}
                aria-label="Close"
            >
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
        </div>
        {#if execError}
            <p class="px-6 py-2 text-xs text-destructive shrink-0">{execError}</p>
        {/if}
        <div class="flex-1 overflow-hidden bg-black p-2" bind:this={execTerminalEl}></div>
    </div>
</dialog>

<header class="flex h-16 shrink-0 items-center gap-2 px-4 border-b">
    <Sidebar.Trigger class="-ms-1" />
    <span class="text-sm text-muted-foreground">/</span>
    <button
        class="text-sm text-muted-foreground hover:text-foreground transition-colors"
        onclick={() => goto("/resource-groups")}
    >
        Resource Groups
    </button>
    <span class="text-sm text-muted-foreground">/</span>
    <span class="text-sm font-medium">{group?.name ?? rgId.slice(0, 8)}</span>
</header>

<div class="flex flex-col gap-6 p-6">
    {#if loading}
        <p class="text-sm text-muted-foreground">Loading...</p>
    {:else if error && !group}
        <p class="text-sm text-destructive">{error}</p>
    {:else if group}
        <div class="flex items-start gap-4">
            <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-lg border bg-muted font-semibold text-sm">
                {initials(group.name)}
            </div>
            <div class="flex-1 min-w-0">
                <div class="flex items-center gap-3 flex-wrap">
                    <h1 class="text-xl font-semibold tracking-tight">{group.name}</h1>
                    <span class="text-xs px-2 py-0.5 rounded-full font-medium {statusClass(group.status)}">{group.status}</span>
                </div>
                {#if group.description}
                    <p class="text-sm text-muted-foreground mt-0.5">{group.description}</p>
                {/if}
                <p class="text-xs text-muted-foreground font-mono mt-1">{group.internal_cidr}</p>
            </div>
            <div class="flex items-center gap-2 shrink-0 flex-wrap justify-end">
                <Button size="sm" variant="outline" onclick={downloadVpnConfig} disabled={downloadingVpn}>
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
                    {downloadingVpn ? "Generating..." : "Connect VPN"}
                </Button>
                <Button size="sm" variant="outline" onclick={() => volumeDialog?.showModal()}>
                    Add Volume
                </Button>
                <Button size="sm" onclick={() => deployDialog?.showModal()}>
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
                    Deploy Container
                </Button>
            </div>
        </div>

        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
            <div class="border rounded-lg p-4">
                <p class="text-xs text-muted-foreground">Containers</p>
                <p class="text-2xl font-semibold mt-1">{workloads.length}</p>
                <p class="text-xs text-muted-foreground mt-0.5">{workloads.filter(w => w.status === 'running').length} running</p>
            </div>
            <div class="border rounded-lg p-4">
                <p class="text-xs text-muted-foreground">Volumes</p>
                <p class="text-2xl font-semibold mt-1">{volumes.length}</p>
                <p class="text-xs text-muted-foreground mt-0.5">{totalDisk} GB total</p>
            </div>
            <div class="border rounded-lg p-4">
                <p class="text-xs text-muted-foreground">CPU Requested</p>
                <p class="text-2xl font-semibold mt-1">{(totalCpu / 1000).toFixed(1)}</p>
                <p class="text-xs text-muted-foreground mt-0.5">vCPU</p>
            </div>
            <div class="border rounded-lg p-4">
                <p class="text-xs text-muted-foreground">Memory Requested</p>
                <p class="text-2xl font-semibold mt-1">{fmtBytes(totalMem)}</p>
                <p class="text-xs text-muted-foreground mt-0.5">across containers</p>
            </div>
        </div>

        {#if error}
            <p class="text-xs text-destructive">{error}</p>
        {/if}

        <div class="border rounded-lg overflow-hidden">
            <div class="px-4 py-3 border-b flex items-center justify-between gap-4 flex-wrap">
                <div class="flex items-center gap-1">
                    {#each [["all", `All ${allResources.length}`], ["container", `Container ${workloads.length}`], ["volume", `Volume ${volumes.length}`]] as [tab, label]}
                        <button
                            class="px-3 py-1 rounded text-sm font-medium transition-colors {activeTab === tab ? 'bg-foreground text-background' : 'text-muted-foreground hover:text-foreground'}"
                            onclick={() => (activeTab = tab as typeof activeTab)}
                        >
                            {label}
                        </button>
                    {/each}
                </div>
                <div class="flex items-center gap-2 border rounded px-3 py-1.5 text-sm min-w-[180px]">
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-muted-foreground shrink-0"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
                    <input class="bg-transparent outline-none flex-1 text-sm" placeholder="Filter resources..." bind:value={filterText} />
                </div>
            </div>

            <table class="w-full text-sm">
                <thead>
                    <tr class="bg-muted/30 border-b">
                        <th class="text-left px-4 py-2.5 font-medium text-muted-foreground text-xs">Resource</th>
                        <th class="text-left px-4 py-2.5 font-medium text-muted-foreground text-xs">Kind</th>
                        <th class="text-left px-4 py-2.5 font-medium text-muted-foreground text-xs">Host / Size</th>
                        <th class="text-left px-4 py-2.5 font-medium text-muted-foreground text-xs">Load</th>
                        <th class="text-left px-4 py-2.5 font-medium text-muted-foreground text-xs">Status</th>
                        <th class="px-4 py-2.5"></th>
                    </tr>
                </thead>
                <tbody>
                    {#if filteredResources.length === 0}
                        <tr>
                            <td colspan="6" class="px-4 py-10 text-center text-muted-foreground text-sm">
                                {allResources.length === 0 ? "No resources yet. Deploy a container or create a volume." : "No resources match filter."}
                            </td>
                        </tr>
                    {:else}
                        {#each filteredResources as item (item.kind + item.data.id)}
                            {#if item.kind === "container"}
                                {@const w = item.data}
                                <tr class="border-t hover:bg-muted/20 transition-colors">
                                    <td class="px-4 py-3">
                                        <div class="flex items-center gap-2.5">
                                            <div class="flex h-7 w-7 shrink-0 items-center justify-center rounded border bg-muted/50">
                                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="7" width="20" height="14" rx="2"/><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"/></svg>
                                            </div>
                                            <div>
                                                <p class="font-medium leading-tight">{w.name}</p>
                                                <p class="text-xs text-muted-foreground font-mono">{w.image}</p>
                                            </div>
                                        </div>
                                    </td>
                                    <td class="px-4 py-3">
                                        <span class="text-xs px-2 py-0.5 rounded border font-medium">Container</span>
                                    </td>
                                    <td class="px-4 py-3 text-xs text-muted-foreground font-mono">
                                        {w.assigned_agent_id ? w.assigned_agent_id.slice(0, 8) : "-"}
                                    </td>
                                    <td class="px-4 py-3">
                                        <div class="flex items-center gap-2 min-w-[120px]">
                                            <span class="text-xs text-muted-foreground w-8">{w.cpu_millicores}m</span>
                                            <div class="flex-1 h-1 rounded-full bg-muted overflow-hidden">
                                                <div class="h-full rounded-full bg-foreground/60" style="width: {Math.min(100, w.cpu_millicores / 10)}%"></div>
                                            </div>
                                        </div>
                                    </td>
                                    <td class="px-4 py-3">
                                        <span class="text-xs px-2 py-0.5 rounded-full font-medium {statusClass(w.status)}">
                                            {statusLabel(w.status)}
                                        </span>
                                    </td>
                                    <td class="px-4 py-3 text-right">
                                        <Button
                                            size="sm"
                                            variant="ghost"
                                            class="h-7 px-2 text-xs"
                                            onclick={() => openLogs(w)}
                                        >
                                            Logs
                                        </Button>
                                        <Button
                                            size="sm"
                                            variant="ghost"
                                            class="h-7 px-2 text-xs"
                                            onclick={() => openExec(w)}
                                            disabled={w.status !== "running"}
                                        >
                                            Shell
                                        </Button>
                                        <Button
                                            size="sm"
                                            variant="ghost"
                                            class="text-destructive hover:text-destructive h-7 px-2 text-xs"
                                            onclick={() => handleDeleteWorkload(w.id)}
                                        >
                                            Stop
                                        </Button>
                                    </td>
                                </tr>
                            {:else}
                                {@const v = item.data}
                                <tr class="border-t hover:bg-muted/20 transition-colors">
                                    <td class="px-4 py-3">
                                        <div class="flex items-center gap-2.5">
                                            <div class="flex h-7 w-7 shrink-0 items-center justify-center rounded border bg-muted/50">
                                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>
                                            </div>
                                            <div>
                                                <p class="font-medium leading-tight">{v.name}</p>
                                                <p class="text-xs text-muted-foreground">{v.size_gb} GB · {v.pool}</p>
                                            </div>
                                        </div>
                                    </td>
                                    <td class="px-4 py-3">
                                        <span class="text-xs px-2 py-0.5 rounded border font-medium">Volume</span>
                                    </td>
                                    <td class="px-4 py-3 text-xs text-muted-foreground">
                                        {v.size_gb} GB
                                    </td>
                                    <td class="px-4 py-3">
                                        <div class="flex items-center gap-2 min-w-[120px]">
                                            <span class="text-xs text-muted-foreground w-8">{v.size_gb}G</span>
                                            <div class="flex-1 h-1 rounded-full bg-muted overflow-hidden">
                                                <div class="h-full rounded-full bg-foreground/30" style="width: {Math.min(100, v.size_gb)}%"></div>
                                            </div>
                                        </div>
                                    </td>
                                    <td class="px-4 py-3">
                                        <span class="text-xs px-2 py-0.5 rounded-full font-medium {statusClass(v.status)}">
                                            {v.status}
                                        </span>
                                    </td>
                                    <td class="px-4 py-3 text-right">
                                        <Button
                                            size="sm"
                                            variant="ghost"
                                            class="text-destructive hover:text-destructive h-7 px-2 text-xs"
                                            onclick={() => handleDeleteVolume(v.id)}
                                            disabled={v.status === "in_use"}
                                        >
                                            Delete
                                        </Button>
                                    </td>
                                </tr>
                            {/if}
                        {/each}
                    {/if}
                </tbody>
            </table>
        </div>
    {/if}
</div>
