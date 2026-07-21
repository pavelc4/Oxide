<script lang="ts">
	/* eslint-disable svelte/no-navigation-without-resolve */
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import ShapeBadge from '$lib/components/ShapeBadge.svelte';
	import ProcessRow from './components/ProcessRow.svelte';
	import ProcessDetailsPanel from './components/ProcessDetailsPanel.svelte';

	let invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | undefined;
	let isTauri = $state(false);

	interface ProcessItem {
		pid: number;
		user: string;
		name: string;
		cpu: number;
		memMb: number;
		isSystem: boolean;
		status: 'running' | 'sleeping';
		threads?: number;
		stateStr?: string;
	}

	interface Device {
		id: string;
		name: string;
	}

	let devices = $state<Device[]>([]);
	let selectedDevice = $state('');
	let processes = $state<ProcessItem[]>([]);
	let loading = $state(false);
	let error = $state('');
	let infoMessage = $state('');

	// Filter & Selection states (App Studio style)
	let processCategory = $state<'ALL' | 'USER' | 'SYSTEM'>('ALL');
	let searchQuery = $state('');
	let activePid = $state<number | null>(null);

	async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
		if (isTauri && invoke) {
			return (await invoke(cmd, args)) as T;
		}
		throw new Error('Tauri API not active');
	}

	onMount(async () => {
		try {
			if (typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window)) {
				const tauriApi = await import('@tauri-apps/api/core');
				invoke = tauriApi.invoke;
				isTauri = true;
			} else {
				isTauri = false;
			}
		} catch {
			isTauri = false;
		}

		await loadDevices();
	});

	async function loadDevices() {
		loading = true;
		error = '';
		try {
			if (isTauri && invoke) {
				const rustDevices = await safeInvoke<Array<{ serial: string; model?: string }>>('get_devices');
				if (rustDevices && rustDevices.length > 0) {
					devices = rustDevices.map((d) => ({ id: d.serial, name: d.model || d.serial }));
				} else {
					devices = [];
				}
			} else {
				devices = [];
			}

			if (devices.length > 0) {
				selectedDevice = devices[0].id;
				await loadProcesses();
			} else {
				selectedDevice = '';
				processes = [];
			}
		} catch (e) {
			error = String(e);
			devices = [];
		} finally {
			loading = false;
		}
	}

	async function loadProcesses() {
		if (!selectedDevice) return;
		loading = true;
		try {
			if (isTauri && invoke) {
				const rustProcs = await safeInvoke<Array<{ pid: number; user: string; name: string; cpu: number; mem: string }>>('list_processes', {
					serial: selectedDevice,
					appsOnly: false
				});
				if (rustProcs && rustProcs.length > 0) {
					processes = rustProcs.map((p) => ({
						pid: p.pid,
						user: p.user || 'system',
						name: p.name,
						cpu: typeof p.cpu === 'number' ? p.cpu : parseFloat(String(p.cpu || 0)),
						memMb: parseInt(String(p.mem || '0').replace(/[^0-9]/g, '')) || 0,
						isSystem: p.user === 'root' || p.user === 'system',
						status: 'running',
						threads: 1,
						stateStr: 'S (sleeping/active)'
					}));
				} else {
					processes = [];
				}
			} else {
				processes = [];
			}

			if (processes.length > 0) {
				activePid = processes[0].pid;
			} else {
				activePid = null;
			}
		} catch (e) {
			error = `Failed to load processes: ${e}`;
			processes = [];
			activePid = null;
		} finally {
			loading = false;
		}
	}

	async function killProcess(proc: ProcessItem) {
		if (!confirm(`Are you sure you want to force stop "${proc.name}" (PID ${proc.pid})?`)) return;
		try {
			if (isTauri && invoke) {
				await safeInvoke('kill_process', { serial: selectedDevice, pid: proc.pid, packageName: proc.name });
			}
			processes = processes.filter((p) => p.pid !== proc.pid);
			if (activePid === proc.pid) {
				activePid = processes[0]?.pid || null;
			}
			infoMessage = `Process "${proc.name}" (PID ${proc.pid}) terminated successfully!`;
		} catch (e) {
			alert(`Failed to kill process: ${e}`);
		}
	}

	function copyProcDetails(proc: ProcessItem) {
		const text = `Process Name: ${proc.name}\nPID: ${proc.pid}\nUser: ${proc.user}\nCPU: ${proc.cpu}%\nRAM: ${proc.memMb} MB\nThreads: ${proc.threads || 12}\nStatus: ${proc.status.toUpperCase()}`;
		navigator.clipboard.writeText(text);
		infoMessage = `Process details copied to clipboard!`;
	}

	// Derived filtered process list
	let filteredProcesses = $derived(
		processes.filter((p) => {
			const categoryMatch = processCategory === 'ALL' || (processCategory === 'USER' ? !p.isSystem : p.isSystem);
			const searchLower = searchQuery.toLowerCase();
			const searchMatch =
				!searchQuery ||
				p.name.toLowerCase().includes(searchLower) ||
				p.user.toLowerCase().includes(searchLower) ||
				String(p.pid).includes(searchLower);
			return categoryMatch && searchMatch;
		})
	);

	// Derived active process item for right detail panel
	let activeProcess = $derived(processes.find((p) => p.pid === activePid) || filteredProcesses[0] || null);
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2 h-screen overflow-hidden">
	<div class="flex flex-1 flex-col overflow-hidden rounded-[32px] bg-surface-container-low p-6 lg:p-8 relative gap-5 shadow-sm">
		
		<!-- Page Header -->
		<header class="flex flex-col md:flex-row md:items-center justify-between gap-4 shrink-0 pb-2">
			<div class="flex items-center gap-3">
				<button
					onclick={() => goto('/')}
					class="flex h-10 w-10 items-center justify-center rounded-xl bg-surface-container hover:bg-surface-container-high text-on-surface-variant transition-all hover:scale-105 active:scale-95 shadow-xs"
					title="Back to dashboard"
				>
					<span class="material-symbols-outlined text-[20px]">arrow_back</span>
				</button>
				<div>
					<div class="flex items-center gap-3">
						<ShapeBadge icon="memory" shape="arch" size={40} iconSize={20} />
						<h2 class="text-2xl font-bold tracking-tight text-on-surface">Process Manager Studio</h2>
						{#if !isTauri}
							<span class="text-[10px] bg-warning/15 text-warning px-2.5 py-0.5 rounded-full font-bold uppercase tracking-wider">MOCK MODE</span>
						{/if}
					</div>
					<p class="text-xs text-on-surface-variant/80 font-medium mt-0.5">Monitor, inspect, & force-terminate active Android background processes</p>
				</div>
			</div>

			<div class="flex items-center gap-3">
				<button
					onclick={loadProcesses}
					class="flex items-center gap-2 rounded-full bg-surface-container hover:bg-surface-container-high px-4 py-2 text-xs font-bold text-on-surface transition-all shadow-xs"
					title="Refresh Process List"
				>
					<span class="material-symbols-outlined text-[16px] {loading ? 'animate-spin' : ''}">refresh</span>
					Refresh
				</button>
			</div>
		</header>

		<!-- Alert Banners -->
		{#if infoMessage}
			<div class="bg-primary/10 text-primary p-3.5 rounded-2xl font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[18px]">check_circle</span>
				<div class="flex-1 font-semibold">{infoMessage}</div>
				<button onclick={() => (infoMessage = '')} class="hover:opacity-80 text-[10px] font-bold uppercase">Dismiss</button>
			</div>
		{/if}

		{#if error}
			<div class="bg-error/15 text-error p-3.5 rounded-2xl font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[18px]">error</span>
				<div class="flex-1 font-semibold">{error}</div>
				<button onclick={() => (error = '')} class="hover:opacity-80 text-[10px] font-bold uppercase">Dismiss</button>
			</div>
		{/if}

		{#if devices.length === 0}
			<div class="flex flex-col items-center justify-center p-12 my-auto rounded-[32px] bg-surface-container text-center shadow-sm">
				<div class="flex h-20 w-20 items-center justify-center rounded-full bg-surface-container-high text-on-surface-variant mb-6 shadow-inner">
					<span class="material-symbols-outlined text-[44px] opacity-70">usb_off</span>
				</div>
				<h3 class="text-xl font-bold tracking-tight text-on-surface mb-2">No Devices Connected</h3>
				<p class="text-xs text-on-surface-variant max-w-md mx-auto mb-6 leading-relaxed">
					Please connect an Android device via USB or Wi-Fi and ensure USB Debugging is enabled.
				</p>
				<button
					class="flex items-center gap-2 rounded-xl bg-primary px-6 py-3 text-xs font-bold text-on-primary transition-all hover:brightness-110 shadow-sm"
					onclick={loadDevices}
				>
					<span class="material-symbols-outlined text-[18px]">refresh</span> Rescan Devices
				</button>
			</div>
		{:else}
			<!-- Category Tabs & Search Bar -->
			<section class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 shrink-0">
				<!-- Category Tabs -->
				<div class="flex rounded-full bg-surface-container p-1 shadow-xs">
					{#each ['ALL', 'USER', 'SYSTEM'] as cat (cat)}
						<button
							onclick={() => (processCategory = cat as 'ALL' | 'USER' | 'SYSTEM')}
							class="px-4 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider transition-all {processCategory === cat ? 'bg-secondary-container text-on-secondary-container shadow-xs' : 'text-on-surface-variant hover:text-on-surface'}"
						>
							{cat === 'USER' ? 'User Apps' : cat === 'SYSTEM' ? 'System Daemons' : 'All Processes'}
						</button>
					{/each}
				</div>

				<!-- Search Bar -->
				<div class="relative w-full sm:w-72">
					<span class="material-symbols-outlined absolute left-3.5 top-1/2 -translate-y-1/2 text-on-surface-variant text-[18px]">search</span>
					<input
						type="text"
						bind:value={searchQuery}
						placeholder="Search process name, PID..."
						class="bg-surface-container rounded-full pl-10 pr-4 py-2 text-xs font-medium text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 w-full shadow-xs"
					/>
				</div>
			</section>

			<!-- 2-Column Main Workspace (App Studio Style Layout) -->
			<div class="flex-1 grid grid-cols-1 lg:grid-cols-12 gap-6 overflow-hidden min-h-0">
				
				<!-- Left Column: Process List (Col 8) -->
				<div class="lg:col-span-8 flex flex-col overflow-hidden rounded-[32px] bg-surface-container p-5 shadow-sm min-h-0">
					<div class="flex items-center justify-between pb-3 mb-2 shrink-0 text-xs font-bold text-on-surface px-2">
						<span>Processes ({filteredProcesses.length})</span>
						<span class="text-[10px] text-on-surface-variant/70 font-mono">Click item to inspect details</span>
					</div>

					<div class="flex-1 overflow-y-auto pr-1 flex flex-col gap-1.5 font-mono text-xs">
						{#if filteredProcesses.length === 0}
							<div class="flex flex-col items-center justify-center h-full text-on-surface-variant/70 p-12 text-center">
								<span class="material-symbols-outlined text-[48px] opacity-40 mb-2">subtitles_off</span>
								<p class="text-xs font-semibold">No processes match filter ({processCategory})</p>
							</div>
						{:else}
							{#each filteredProcesses as proc (proc.pid)}
								<ProcessRow
									{proc}
									isActive={activePid === proc.pid}
									onselect={() => (activePid = proc.pid)}
									onkill={(e) => {
										e.stopPropagation();
										killProcess(proc);
									}}
								/>
							{/each}
						{/if}
					</div>
				</div>

				<!-- Right Column: Process Details Inspector Panel -->
				<ProcessDetailsPanel
					proc={activeProcess}
					onkill={() => { if (activeProcess) killProcess(activeProcess); }}
				/>

			</div>
		{/if}

	</div>
</main>

<style>
	@keyframes fadeIn {
		from { opacity: 0; transform: scale(0.99); }
		to { opacity: 1; transform: scale(1); }
	}
	.animate-fade-in { animation: fadeIn 0.15s cubic-bezier(0.2, 0.8, 0.2, 1) forwards; }
</style>
