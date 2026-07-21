<script lang="ts">
	/* eslint-disable svelte/no-navigation-without-resolve */
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import ShapeBadge from '$lib/components/ShapeBadge.svelte';
	import LogcatItem from './components/LogcatItem.svelte';

	let invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | undefined;
	let isTauri = $state(false);

	interface LogEntry {
		id: number;
		time: string;
		pid: number;
		tid: number;
		level: 'V' | 'D' | 'I' | 'W' | 'E' | 'F';
		tag: string;
		message: string;
	}

	interface Device {
		id: string;
		name: string;
	}

	let devices = $state<Device[]>([]);
	let selectedDevice = $state('');
	let loading = $state(false);
	let error = $state('');
	let infoMessage = $state('');

	// Logcat streaming state
	let logEntries = $state<LogEntry[]>([]);
	let isStreaming = $state(true);
	let autoScroll = $state(true);
	let userPausedScroll = $state(false); // true when user clicked a log or scrolled up
	let selectedLevel = $state<'ALL' | 'V' | 'D' | 'I' | 'W' | 'E' | 'F'>('ALL');
	let searchQuery = $state('');
	let selectedLogId = $state<number | null>(null);

	let logContainer: HTMLDivElement | undefined;
	let logCounter = 1;
	let streamInterval: ReturnType<typeof setInterval> | undefined;

	// Detect user scroll position: if scrolled back to bottom, resume autoScroll
	function handleLogScroll() {
		if (!logContainer) return;
		const { scrollTop, scrollHeight, clientHeight } = logContainer;
		const atBottom = scrollHeight - scrollTop - clientHeight < 40;
		if (atBottom && userPausedScroll) {
			userPausedScroll = false;
			autoScroll = true;
		} else if (!atBottom && !userPausedScroll) {
			userPausedScroll = true;
			autoScroll = false;
		}
	}

	// When user selects a log entry, pause autoScroll
	function selectLog(logId: number) {
		if (selectedLogId === logId) {
			selectedLogId = null;
		} else {
			selectedLogId = logId;
			autoScroll = false;
			userPausedScroll = true;
		}
	}

	// Derived filtered logs
	let filteredLogs = $derived(
		logEntries.filter((log) => {
			const levelMatch = selectedLevel === 'ALL' || log.level === selectedLevel;
			const searchLower = searchQuery.toLowerCase();
			const searchMatch =
				!searchQuery ||
				log.tag.toLowerCase().includes(searchLower) ||
				log.message.toLowerCase().includes(searchLower) ||
				String(log.pid).includes(searchLower);
			return levelMatch && searchMatch;
		})
	);

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

		// Real logcat stream polling
		streamInterval = setInterval(async () => {
			if (!isStreaming || !selectedDevice || !isTauri) return;
			try {
				const output = await safeInvoke<string>('device_shell', {
					serial: selectedDevice,
					command: 'logcat -d -t 20'
				});
				if (output) {
					const lines = output.split('\n');
					const newEntries: LogEntry[] = [];
					for (const line of lines) {
						if (!line.trim() || line.startsWith('------')) continue;
						const match = line.match(/^(\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}\.\d{3})\s+(\d+)\s+(\d+)\s+([VDIWEF])\s+([^:]+):\s+(.*)$/);
						if (match) {
							newEntries.push({
								id: logCounter++,
								time: match[1],
								pid: parseInt(match[2], 10),
								tid: parseInt(match[3], 10),
								level: match[4] as LogEntry['level'],
								tag: match[5].trim(),
								message: match[6]
							});
						}
					}
					if (newEntries.length > 0) {
						logEntries = [...logEntries.slice(-1000), ...newEntries];
						if (autoScroll && logContainer) {
							setTimeout(() => {
								if (logContainer) {
									logContainer.scrollTop = logContainer.scrollHeight;
								}
							}, 20);
						}
					}
				}
			} catch (e) {
				console.warn('Logcat stream error:', e);
			}
		}, 2000);
	});

	onDestroy(() => {
		if (streamInterval) clearInterval(streamInterval);
	});

	async function loadDevices() {
		loading = true;
		try {
			if (isTauri && invoke) {
				const rustDevices = await safeInvoke<Array<{ serial: string; model?: string }>>('get_devices');
				if (rustDevices && rustDevices.length > 0) {
					devices = rustDevices.map((d) => ({ id: d.serial, name: d.model || d.serial }));
				} else {
					devices = [];
				}
			} else {
				devices = [
					{ id: 'fastboot-pixel-8', name: 'Google Pixel 8 Pro' },
					{ id: 'adb-galaxy-s24', name: 'Samsung Galaxy S24 Ultra' }
				];
			}
			if (devices.length > 0) selectedDevice = devices[0].id;
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	function clearLogs() {
		logEntries = [];
		selectedLogId = null;
	}

	function getFormattedTimestamp(): string {
		const now = new Date();
		const dd = String(now.getDate()).padStart(2, '0');
		const mm = String(now.getMonth() + 1).padStart(2, '0');
		const yyyy = now.getFullYear();
		const hh = String(now.getHours()).padStart(2, '0');
		const min = String(now.getMinutes()).padStart(2, '0');
		const ss = String(now.getSeconds()).padStart(2, '0');
		return `${dd}-${mm}-${yyyy}_${hh}-${min}-${ss}`;
	}

	function exportLogs() {
		if (logEntries.length === 0) return;
		const ext = typeof localStorage !== 'undefined'
			? (localStorage.getItem('oxide:logcatExportFormat') || 'log')
			: 'log';
		const timestamp = getFormattedTimestamp();
		const fileName = `logcat_${selectedDevice || 'device'}_${timestamp}.${ext}`;
		const text = logEntries.map((l) => `${l.time} ${l.pid} ${l.tid} ${l.level} ${l.tag}: ${l.message}`).join('\n');
		const blob = new Blob([text], { type: 'text/plain;charset=utf-8' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = fileName;
		a.click();
		URL.revokeObjectURL(url);
		infoMessage = `Exported ${logEntries.length} logcat entries to Downloads/Oxide/Logcat (${fileName})!`;
	}

	function copyLogText(log: LogEntry) {
		const str = `${log.time} [${log.pid}:${log.tid}] ${log.level}/${log.tag}: ${log.message}`;
		navigator.clipboard.writeText(str);
		infoMessage = `Copied log entry to clipboard!`;
	}
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
						<ShapeBadge icon="subtitles" shape="ghostish" size={40} iconSize={20} />
						<h2 class="text-2xl font-bold tracking-tight text-on-surface">Logcat System Trace Studio</h2>
						{#if !isTauri}
							<span class="text-[10px] bg-warning/15 text-warning px-2.5 py-0.5 rounded-full font-bold uppercase tracking-wider">MOCK MODE</span>
						{/if}
					</div>
					<p class="text-xs text-on-surface-variant/80 font-medium mt-0.5">Stream, filter, & inspect real-time Android kernel, framework, & app logs</p>
				</div>
			</div>

			<div class="flex items-center gap-3">
				<button
					onclick={() => (isStreaming = !isStreaming)}
					class="flex items-center gap-2.5 rounded-full bg-surface-container hover:bg-surface-container-high px-4 py-2 text-xs font-bold text-on-surface transition-all shadow-xs"
				>
					<span class="relative flex h-2.5 w-2.5">
						{#if isStreaming}
							<span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
							<span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500"></span>
						{:else}
							<span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-amber-500"></span>
						{/if}
					</span>
					<span class="material-symbols-outlined text-[16px] text-on-surface-variant">{isStreaming ? 'pause' : 'play_arrow'}</span>
					{isStreaming ? 'Streaming' : 'Paused'}
				</button>

				<button
					onclick={exportLogs}
					class="flex items-center gap-2 rounded-full bg-primary px-4 py-1.5 text-xs font-bold text-on-primary hover:brightness-110 transition-all shadow-sm"
					title="Export Logcat File"
				>
					<span class="material-symbols-outlined text-[16px]">download</span>
					Export Log
				</button>
			</div>
		</header>

		<!-- Notification Banner -->
		{#if infoMessage}
			<div class="bg-primary/10 text-primary p-3 rounded-2xl font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[18px]">check_circle</span>
				<div class="flex-1 font-semibold">{infoMessage}</div>
				<button onclick={() => (infoMessage = '')} class="hover:opacity-80 text-[10px] font-bold uppercase">Dismiss</button>
			</div>
		{/if}

		<!-- Controls Toolbar (Level Filters & Search) -->
		<section class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 shrink-0">
			<!-- Level Filter Pills -->
			<div class="flex rounded-full bg-surface-container p-1 shadow-xs">
				{#each ['ALL', 'V', 'D', 'I', 'W', 'E', 'F'] as lvl (lvl)}
					<button
						onclick={() => (selectedLevel = lvl as 'ALL' | 'V' | 'D' | 'I' | 'W' | 'E' | 'F')}
						class="px-3.5 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider transition-all {selectedLevel === lvl ? 'bg-secondary-container text-on-secondary-container shadow-xs' : 'text-on-surface-variant hover:text-on-surface'}"
					>
						{lvl === 'V' ? 'VERBOSE' : lvl === 'D' ? 'DEBUG' : lvl === 'I' ? 'INFO' : lvl === 'W' ? 'WARN' : lvl === 'E' ? 'ERROR' : lvl === 'F' ? 'FATAL' : 'ALL'}
					</button>
				{/each}
			</div>

			<!-- Search Bar & Action Buttons -->
			<div class="flex items-center gap-3 w-full sm:w-auto">
				<div class="relative flex-1 sm:flex-none">
					<span class="material-symbols-outlined absolute left-3.5 top-1/2 -translate-y-1/2 text-on-surface-variant text-[18px]">search</span>
					<input
						type="text"
						bind:value={searchQuery}
						placeholder="Search tag, PID, message..."
						class="bg-surface-container rounded-full pl-10 pr-4 py-2 text-xs font-medium text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 w-full sm:w-64 shadow-xs"
					/>
				</div>

				<button
					onclick={() => (autoScroll = !autoScroll)}
					class="flex h-9 w-9 items-center justify-center rounded-full bg-surface-container hover:bg-surface-container-high transition-all shrink-0 shadow-xs {autoScroll ? 'text-primary' : 'text-on-surface-variant/60'}"
					title={autoScroll ? 'Auto-scroll Active' : 'Auto-scroll Paused'}
				>
					<span class="material-symbols-outlined text-[18px]">vertical_align_bottom</span>
				</button>

				<button
					onclick={clearLogs}
					class="flex h-9 w-9 items-center justify-center rounded-full bg-surface-container hover:bg-surface-container-high text-on-surface-variant hover:text-error transition-all shrink-0 shadow-xs"
					title="Clear Console"
				>
					<span class="material-symbols-outlined text-[18px]">delete</span>
				</button>
			</div>
		</section>

		<!-- Unified Borderless Seamless Log Stream Panel -->
		<div class="flex flex-1 flex-col overflow-hidden rounded-[32px] bg-surface-container p-6 shadow-sm min-h-0">
			<!-- Table Header -->
			<div class="grid grid-cols-12 gap-3 pb-3 mb-2 shrink-0 text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/70 px-3">
				<div class="col-span-2 sm:col-span-2">Timestamp</div>
				<div class="col-span-1 hidden sm:block">PID</div>
				<div class="col-span-1">Level</div>
				<div class="col-span-3 sm:col-span-2">Tag</div>
				<div class="col-span-6 sm:col-span-6">Message Payload</div>
			</div>

			<!-- Scrollable Seamless Log Stream List (Zero Outline Borders) -->
			<div bind:this={logContainer} onscroll={handleLogScroll} class="flex-1 overflow-y-auto pr-1 flex flex-col gap-0.5 font-mono text-xs">
				{#if filteredLogs.length === 0}
					<div class="flex flex-col items-center justify-center h-full text-on-surface-variant/70 p-12 text-center">
						<span class="material-symbols-outlined text-[48px] opacity-40 mb-2">subtitles_off</span>
						<p class="text-xs font-semibold">No logcat entries match filter ({selectedLevel})</p>
					</div>
				{:else}
					{#each filteredLogs as log (log.id)}
						<LogcatItem
							{log}
							isSelected={selectedLogId === log.id}
							onselect={() => selectLog(log.id)}
							oncopy={copyLogText}
						/>
					{/each}
				{/if}
			</div>
		</div>

		<!-- Footer Info Bar -->
		<footer class="flex justify-between items-center text-xs text-on-surface-variant font-medium shrink-0 px-2">
			<span>Showing {filteredLogs.length} of {logEntries.length} logcat entries</span>
			<span class="flex items-center gap-2">
				<span class="h-2 w-2 rounded-full {isStreaming ? 'bg-emerald-500 animate-pulse' : 'bg-amber-500'}"></span>
				{isStreaming ? 'Live Stream Active' : 'Stream Paused'}
			</span>
		</footer>

	</div>
</main>

<style>
	@keyframes fadeIn {
		from { opacity: 0; transform: scale(0.99); }
		to { opacity: 1; transform: scale(1); }
	}
	.animate-fade-in { animation: fadeIn 0.15s cubic-bezier(0.2, 0.8, 0.2, 1) forwards; }
</style>
