<script lang="ts">
	/* eslint-disable svelte/no-navigation-without-resolve */
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';

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
	let selectedLevel = $state<'ALL' | 'V' | 'D' | 'I' | 'W' | 'E' | 'F'>('ALL');
	let searchQuery = $state('');
	let selectedLogId = $state<number | null>(null);

	let logContainer: HTMLDivElement | undefined;
	let logCounter = 1;
	let streamInterval: ReturnType<typeof setInterval> | undefined;

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

	const sampleLogs: Omit<LogEntry, 'id' | 'time' | 'pid' | 'tid'>[] = [
		{ level: 'I', tag: 'ActivityManager', message: 'Displayed com.android.chrome/.MainActivity: +214ms' },
		{ level: 'D', tag: 'WifiService', message: 'WifiDevice: Connected to 5GHz_Office_WiFi (RSSI: -48dBm)' },
		{ level: 'I', tag: 'BatteryService', message: 'UPDATE battery level=88%, status=Charging, temp=31.4°C' },
		{ level: 'D', tag: 'InputDispatcher', message: 'Delivered MotionEvent ACTION_DOWN to window (x=540, y=1180)' },
		{ level: 'W', tag: 'SurfaceFlinger', message: 'Frame 18294 dropped: VSYNC missed by 3.8ms' },
		{ level: 'I', tag: 'PackageManager', message: 'Package Verification result: ALLOW for com.whatsapp' },
		{ level: 'E', tag: 'MediaProvider', message: 'Failed to access /sdcard/DCIM/.thumbnails: Permission denied' },
		{ level: 'D', tag: 'AudioService', message: 'AudioDeviceChanged: Bluetooth A2DP active (LDAC 990kbps)' },
		{ level: 'I', tag: 'OxideDaemon', message: 'ADB Server socket 127.0.0.1:5037 heartbeat OK' },
		{ level: 'W', tag: 'SystemUI', message: 'NotificationShade expand event triggered by user touch' },
		{ level: 'D', tag: 'GmsCore', message: 'GMS LocationProvider: Location update delivered (acc=5.0m)' },
		{ level: 'E', tag: 'CameraService', message: 'HAL device error: Stream 0 buffer allocation failed' }
	];

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

		// Initial seed logs
		const initial: LogEntry[] = sampleLogs.slice(0, 10).map((s, i) => ({
			id: i + 1,
			time: new Date(Date.now() - (10 - i) * 1500).toTimeString().split(' ')[0] + '.' + Math.floor(Math.random() * 900 + 100),
			pid: 1200 + i * 35,
			tid: 1240 + i * 35,
			level: s.level,
			tag: s.tag,
			message: s.message
		}));
		logEntries = initial;
		logCounter = 11;

		// Live log streaming loop
		streamInterval = setInterval(() => {
			if (!isStreaming) return;
			const sample = sampleLogs[Math.floor(Math.random() * sampleLogs.length)];
			const newEntry: LogEntry = {
				id: logCounter++,
				time: new Date().toTimeString().split(' ')[0] + '.' + Math.floor(Math.random() * 900 + 100),
				pid: 1400 + Math.floor(Math.random() * 600),
				tid: 1450 + Math.floor(Math.random() * 600),
				level: sample.level,
				tag: sample.tag,
				message: sample.message
			};

			logEntries = [...logEntries.slice(-1000), newEntry];

			if (autoScroll && logContainer) {
				setTimeout(() => {
					if (logContainer) {
						logContainer.scrollTop = logContainer.scrollHeight;
					}
				}, 20);
			}
		}, 1400);
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

	function exportLogs() {
		if (logEntries.length === 0) return;
		const text = logEntries.map((l) => `${l.time} ${l.pid} ${l.tid} ${l.level} ${l.tag}: ${l.message}`).join('\n');
		const blob = new Blob([text], { type: 'text/plain;charset=utf-8' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `logcat_${selectedDevice || 'device'}_${Date.now()}.txt`;
		a.click();
		URL.revokeObjectURL(url);
		infoMessage = `Exported ${logEntries.length} logcat entries to file!`;
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
			<div bind:this={logContainer} class="flex-1 overflow-y-auto pr-1 flex flex-col gap-0.5 font-mono text-xs">
				{#if filteredLogs.length === 0}
					<div class="flex flex-col items-center justify-center h-full text-on-surface-variant/70 p-12 text-center">
						<span class="material-symbols-outlined text-[48px] opacity-40 mb-2">subtitles_off</span>
						<p class="text-xs font-semibold">No logcat entries match filter ({selectedLevel})</p>
					</div>
				{:else}
					{#each filteredLogs as log (log.id)}
						<div
							role="button"
							tabindex="0"
							onclick={() => (selectedLogId = selectedLogId === log.id ? null : log.id)}
							onkeydown={(e) => { if (e.key === 'Enter') selectedLogId = selectedLogId === log.id ? null : log.id; }}
							class="flex flex-col px-3 py-2 rounded-xl transition-all cursor-pointer gap-2 {selectedLogId === log.id ? 'bg-surface-container-highest shadow-xs' : 'hover:bg-surface-container-high/60'}"
						>
							<!-- Row Content -->
							<div class="grid grid-cols-12 gap-3 items-center w-full">
								<!-- Time -->
								<div class="col-span-2 sm:col-span-2 text-[11px] text-on-surface-variant/80 truncate">
									{log.time}
								</div>

								<!-- PID -->
								<div class="col-span-1 hidden sm:block text-[11px] text-on-surface-variant/60">
									{log.pid}
								</div>

								<!-- Level Badge -->
								<div class="col-span-1">
									{#if log.level === 'I'}
										<span class="text-[9px] font-bold bg-emerald-500/15 text-emerald-400 px-2 py-0.5 rounded-full">INFO</span>
									{:else if log.level === 'D'}
										<span class="text-[9px] font-bold bg-sky-500/15 text-sky-400 px-2 py-0.5 rounded-full">DEBUG</span>
									{:else if log.level === 'W'}
										<span class="text-[9px] font-bold bg-amber-500/15 text-amber-400 px-2 py-0.5 rounded-full">WARN</span>
									{:else if log.level === 'E'}
										<span class="text-[9px] font-bold bg-rose-500/15 text-rose-400 px-2 py-0.5 rounded-full">ERROR</span>
									{:else if log.level === 'F'}
										<span class="text-[9px] font-bold bg-purple-500/15 text-purple-400 px-2 py-0.5 rounded-full">FATAL</span>
									{:else}
										<span class="text-[9px] font-bold bg-neutral-500/15 text-neutral-400 px-2 py-0.5 rounded-full">VERB</span>
									{/if}
								</div>

								<!-- Tag -->
								<div class="col-span-3 sm:col-span-2 text-xs font-bold text-on-surface truncate">
									{log.tag}
								</div>

								<!-- Message -->
								<div class="col-span-6 sm:col-span-6 text-xs text-on-surface-variant truncate font-sans">
									{log.message}
								</div>
							</div>

							<!-- Expanded Details Card (Full-Width, Borderless) -->
							{#if selectedLogId === log.id}
								<div
									role="region"
									aria-label="Log Details"
									class="w-full mt-1 flex flex-col gap-2.5 bg-surface-container-high p-4 rounded-2xl font-mono text-xs animate-fade-in shadow-xs"
									onclick={(e) => e.stopPropagation()}
									onkeydown={(e) => e.stopPropagation()}
								>
									<div class="flex justify-between items-center text-on-surface-variant">
										<span class="font-bold text-primary text-xs">Full Log Entry Payload</span>
										<button
											onclick={() => copyLogText(log)}
											class="flex items-center gap-1.5 text-[11px] font-bold bg-primary/15 text-primary px-3.5 py-1.5 rounded-xl hover:brightness-110 transition-all shadow-xs"
										>
											<span class="material-symbols-outlined text-[15px]">content_copy</span>
											Copy Payload
										</button>
									</div>
									<div class="bg-surface-container p-3.5 rounded-xl text-on-surface break-all leading-relaxed font-mono text-xs shadow-inner">
										{log.time} [{log.pid}:{log.tid}] {log.level}/{log.tag}: {log.message}
									</div>
								</div>
							{/if}
						</div>
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
