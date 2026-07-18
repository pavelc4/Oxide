<script lang="ts">
	/* eslint-disable svelte/no-navigation-without-resolve */
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import Sparkline from '$lib/components/Sparkline.svelte';

	let invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | undefined;
	let isTauri = $state(false);

	interface Device {
		id: string;
		name: string;
		status: string;
		connection: string;
		model?: string;
		androidVersion?: string;
		batteryLevel?: number;
		isCharging?: boolean;
		storageUsedGb?: number;
		storageTotalGb?: number;
		resolution?: string;
		abi?: string;
	}

	// Devices State
	let devices = $state<Device[]>([]);
	let loading = $state(true);
	let error = $state('');
	let selectedDevice = $state('');
	let activeDevice = $derived(devices.find((d) => d.id === selectedDevice) || devices[0] || null);

	// Performance Monitoring State
	let dataCpu = $state(Array(40).fill(0));
	let dataMem = $state(Array(40).fill(0));
	let dataFps = $state(Array(40).fill(0));
	let dataCores = $state(Array(8).fill(0).map(() => Array(25).fill(0)));

	let coreUsages = $state(Array(8).fill(0));
	let coreSpeeds = $state(Array(8).fill(0));
	let topPackageName = $state('System UI');
	let currentFps = $state(60);
	let currentCpu = $state(18);
	let currentMem = $state(45);
	let memStr = $state('3.8 GB / 8 GB');
	let uptimeStr = $state('0:00:00');
	let lastFpsData: unknown = null;
	let lastCpuData: unknown = null;

	let intervalId: ReturnType<typeof setInterval> | undefined;
	let isPageVisible = true;

	function onVisibilityChange() {
		isPageVisible = !document.hidden;
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

		// Initial mock graphs for aesthetic preview
		dataCpu = Array(40).fill(0).map(() => Math.floor(Math.random() * 20 + 15));
		dataMem = Array(40).fill(0).map(() => Math.floor(Math.random() * 15 + 40));
		dataFps = Array(40).fill(0).map(() => Math.floor(Math.random() * 10 + 52));

		document.addEventListener('visibilitychange', onVisibilityChange);

		let isPolling = false;

		intervalId = setInterval(async () => {
			if (!isPageVisible || !isTauri || !invoke || !selectedDevice || isPolling) return;

			isPolling = true;
			try {
				const [topPkg, perf] = await Promise.all([
					safeInvoke<{ name?: string }>('get_top_package', { serial: selectedDevice }).catch(() => null),
					safeInvoke<{ memory?: any; uptime?: any; cpu?: any; fps?: any }>('get_performance_profile', { serial: selectedDevice }).catch(() => null)
				]);

				if (topPkg && topPkg.name) {
					topPackageName = topPkg.name.split('.').pop() || topPkg.name;
				}

				if (perf) {
					if (perf.memory && perf.memory.Ok) {
						const memInfo = perf.memory.Ok;
						currentMem = Math.round(((memInfo.total_kb - memInfo.available_kb) / memInfo.total_kb) * 100);
						const usedGb = ((memInfo.total_kb - memInfo.available_kb) / (1024 * 1024)).toFixed(1);
						const totalGb = (memInfo.total_kb / (1024 * 1024)).toFixed(1);
						memStr = `${usedGb} GB / ${totalGb} GB`;
						dataMem = [...dataMem.slice(1), currentMem];
					}

					if (perf.uptime && perf.uptime.Ok) {
						const ut = perf.uptime.Ok;
						const days = Math.floor(ut / 86400);
						const hours = Math.floor((ut % 86400) / 3600);
						const minutes = Math.floor((ut % 3600) / 60);
						const seconds = Math.floor(ut % 60);
						uptimeStr = `${hours}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
						if (days > 0) uptimeStr = `${days}d ` + uptimeStr;
					}

					if (perf.cpu && perf.cpu.Ok && perf.cpu.Ok.length > 0) {
						const currentCpus = perf.cpu.Ok;
						if (lastCpuData !== null && currentCpus.length > 0 && Array.isArray(lastCpuData) && lastCpuData.length > 0) {
							const curr = currentCpus[0].times;
							const prev = (lastCpuData as any[])[0].times;

							const c_total = curr.user + curr.nice + curr.sys + curr.idle + curr.iowait + curr.irq + curr.softirq;
							const p_total = prev.user + prev.nice + prev.sys + prev.idle + prev.iowait + prev.irq + prev.softirq;

							const totalDelta = c_total - p_total;
							const idleDelta = curr.idle + curr.iowait - (prev.idle + prev.iowait);

							if (totalDelta > 0) {
								currentCpu = Math.round(100 * (1 - idleDelta / totalDelta));
							}
						}
						lastCpuData = currentCpus;
						dataCpu = [...dataCpu.slice(1), currentCpu];
					}

					if (perf.fps && perf.fps.Ok) {
						const fpsData = perf.fps.Ok;
						if (fpsData.flips !== null && lastFpsData !== null && fpsData.flips > (lastFpsData as any).flips) {
							const deltaFlips = fpsData.flips - (lastFpsData as any).flips;
							const deltaTime = fpsData.timestamp_ms - (lastFpsData as any).timestamp_ms;
							if (deltaTime > 0) {
								currentFps = Math.round((deltaFlips * 1000) / deltaTime);
							}
						}
						lastFpsData = fpsData;
						dataFps = [...dataFps.slice(1), currentFps];
					}
				}
			} catch (e) {
				console.warn('Polling status info:', e);
			} finally {
				isPolling = false;
			}
		}, 1500);
	});

	onDestroy(() => {
		if (typeof document !== 'undefined') {
			document.removeEventListener('visibilitychange', onVisibilityChange);
		}
		if (intervalId) {
			clearInterval(intervalId);
		}
	});

	async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
		if (isTauri && invoke) {
			return (await invoke(cmd, args)) as T;
		}
		throw new Error('Tauri API not active');
	}

	async function loadDevices() {
		loading = true;
		error = '';
		try {
			if (isTauri && invoke) {
				const rustDevices = await safeInvoke<Array<{ serial: string; model?: string }>>('get_devices');
				if (rustDevices && rustDevices.length > 0) {
					devices = rustDevices.map((d) => ({
						id: d.serial,
						name: d.model || d.serial,
						status: 'Online',
						connection: d.serial.includes('.') ? 'Wireless ADB' : 'USB',
						androidVersion: 'Android 14 (API 34)',
						batteryLevel: 88,
						isCharging: true,
						storageUsedGb: 64,
						storageTotalGb: 128,
						resolution: '1440 x 3120 (560 dpi)',
						abi: 'arm64-v8a'
					}));
				} else {
					devices = [];
				}
			} else {
				devices = [
					{
						id: 'fastboot-pixel-8',
						name: 'Google Pixel 8 Pro',
						status: 'Online',
						connection: 'USB 3.2 Gen2',
						androidVersion: 'Android 14 (API 34)',
						batteryLevel: 92,
						isCharging: true,
						storageUsedGb: 84,
						storageTotalGb: 256,
						resolution: '1440 x 3120 @ 120Hz',
						abi: 'arm64-v8a'
					},
					{
						id: 'adb-galaxy-s24',
						name: 'Samsung Galaxy S24 Ultra',
						status: 'Online',
						connection: 'Wireless ADB',
						androidVersion: 'Android 14 (OneUI 6.1)',
						batteryLevel: 76,
						isCharging: false,
						storageUsedGb: 112,
						storageTotalGb: 512,
						resolution: '1440 x 3120 @ 120Hz',
						abi: 'arm64-v8a'
					}
				];
			}

			if (devices.length > 0) {
				selectedDevice = devices[0].id;
			} else {
				selectedDevice = '';
			}
		} catch (e) {
			error = String(e);
			devices = [];
		} finally {
			loading = false;
		}
	}

	async function rebootDevice(mode: 'normal' | 'bootloader' | 'recovery') {
		if (!selectedDevice) return;
		try {
			if (isTauri && invoke) {
				await safeInvoke('reboot_device', { serial: selectedDevice, mode });
			}
			alert(`Rebooting ${selectedDevice} to ${mode.toUpperCase()} mode...`);
		} catch (e) {
			alert(`Reboot failed: ${e}`);
		}
	}
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2 h-screen overflow-hidden">
	<div class="flex flex-1 flex-col overflow-y-auto rounded-[32px] bg-surface-container-low p-6 lg:p-8 relative gap-6 shadow-sm">
		
		<!-- Top Dashboard Header -->
		<header class="flex flex-col md:flex-row md:items-center justify-between gap-4 shrink-0 pb-2">
			<div class="flex items-center gap-3">
				<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary-container text-on-primary-container shadow-inner">
					<span class="material-symbols-outlined text-[22px]">science</span>
				</div>
				<div>
					<div class="flex items-center gap-3">
						<h2 class="text-2xl font-bold tracking-tight text-on-surface">OXIDE</h2>
						{#if !isTauri}
							<span class="text-[10px] bg-warning/15 text-warning px-2.5 py-0.5 rounded-full font-bold uppercase tracking-wider">MOCK PREVIEW</span>
						{/if}
					</div>
					<p class="text-xs text-on-surface-variant/80 font-medium mt-0.5">High-performance cross-platform ADB device manager</p>
				</div>
			</div>

			<div class="flex items-center gap-3">
				{#if devices.length > 0}
					<div class="flex items-center gap-2 bg-surface-container px-4 py-2 rounded-full shadow-xs">
						<span class="relative flex h-2.5 w-2.5">
							<span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
							<span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500"></span>
						</span>
						<select
							bind:value={selectedDevice}
							class="bg-transparent border-none text-xs font-semibold text-on-surface focus:outline-none cursor-pointer pr-2"
						>
							{#each devices as dev (dev.id)}
								<option value={dev.id}>{dev.name} ({dev.connection})</option>
							{/each}
						</select>
					</div>
				{/if}

				<button
					onclick={loadDevices}
					class="flex h-9 w-9 items-center justify-center rounded-full bg-surface-container hover:bg-surface-container-high text-on-surface-variant transition-all hover:scale-105 active:scale-95 shadow-xs"
					title="Rescan Connected Devices"
				>
					<span class="material-symbols-outlined text-[18px] {loading ? 'animate-spin' : ''}">refresh</span>
				</button>
			</div>
		</header>

		{#if error}
			<div class="bg-error/15 text-error p-3.5 rounded-2xl font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[20px]">error</span>
				<div class="flex-1 break-words font-semibold">{error}</div>
				<button onclick={() => (error = '')} class="hover:opacity-80 text-[10px] font-bold uppercase tracking-wider bg-error/20 px-2.5 py-1 rounded-lg">Dismiss</button>
			</div>
		{/if}

		{#if devices.length === 0}
			<div class="flex flex-col items-center justify-center p-12 my-auto rounded-[32px] bg-surface-container text-center shadow-sm">
				<div class="flex h-20 w-20 items-center justify-center rounded-full bg-surface-container-high text-on-surface-variant mb-6 shadow-inner">
					<span class="material-symbols-outlined text-[44px] opacity-70">usb_off</span>
				</div>
				<h3 class="text-xl font-bold tracking-tight text-on-surface mb-2">No Devices Connected</h3>
				<p class="text-xs text-on-surface-variant max-w-md mx-auto mb-6 leading-relaxed">
					Please connect an Android smartphone via USB or Wi-Fi network and ensure USB Debugging is enabled in Developer Options.
				</p>
				<div class="flex gap-3">
					<button
						class="flex items-center gap-2 rounded-xl bg-primary px-6 py-3 text-xs font-bold text-on-primary transition-all hover:brightness-110 shadow-sm"
						onclick={loadDevices}
					>
						<span class="material-symbols-outlined text-[18px]">refresh</span> Rescan ADB Devices
					</button>
					<button
						class="flex items-center gap-2 rounded-xl bg-surface-container-high px-6 py-3 text-xs font-bold text-on-surface hover:bg-surface-container-highest transition-all"
						onclick={() => goto('/wireless')}
					>
						<span class="material-symbols-outlined text-[18px]">settings_input_antenna</span> Wireless ADB Setup
					</button>
				</div>
			</div>
		{:else if activeDevice}
			<!-- Main Device Overview Section (Mock Phone + Info Cards) -->
			<section class="grid grid-cols-1 lg:grid-cols-12 gap-6 items-stretch">
				
				<!-- Left Column: Modern Mock Smartphone Frame (Col 4) -->
				<div class="lg:col-span-4 flex flex-col items-center justify-center rounded-[32px] bg-surface-container p-6 relative overflow-hidden shadow-sm min-h-[380px]">
					<div class="relative w-52 h-[340px] rounded-[42px] bg-neutral-900 p-3 shadow-2xl flex flex-col justify-between border-4 border-neutral-800">
						<!-- Phone Notch / Camera Pill -->
						<div class="w-20 h-4 bg-neutral-950 rounded-full mx-auto flex items-center justify-center gap-2 px-2 z-20">
							<span class="w-1.5 h-1.5 rounded-full bg-neutral-700"></span>
							<span class="w-2.5 h-2.5 rounded-full bg-neutral-800"></span>
						</div>

						<!-- Smartphone Screen Content -->
						<div class="flex-1 rounded-[30px] bg-gradient-to-br from-neutral-900 via-neutral-950 to-primary/20 p-4 flex flex-col justify-between relative overflow-hidden my-1">
							<!-- Screen Top Bar -->
							<div class="flex justify-between items-center text-[10px] font-mono text-neutral-400 z-10">
								<span>10:00</span>
								<div class="flex items-center gap-1.5">
									<span class="material-symbols-outlined text-[12px]">wifi</span>
									<span class="material-symbols-outlined text-[12px] text-emerald-400">battery_5_bar</span>
								</div>
							</div>

							<!-- Center Screen Brand Watermark -->
							<div class="flex flex-col items-center justify-center gap-2 my-auto text-center z-10">
								<div class="w-12 h-12 rounded-2xl bg-primary/20 text-primary flex items-center justify-center shadow-lg backdrop-blur-md">
									<span class="material-symbols-outlined text-[26px]">smartphone</span>
								</div>
								<span class="text-xs font-bold text-on-surface tracking-wide">{activeDevice.name}</span>
								<span class="text-[9px] font-mono text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-full uppercase">Connected ({activeDevice.connection})</span>
							</div>

							<!-- Screen Bottom Indicator -->
							<div class="w-16 h-1 bg-neutral-600/60 rounded-full mx-auto z-10"></div>
						</div>
					</div>
				</div>

				<!-- Right Column: Extended Device Specs & Quick Action Buttons (Col 8) -->
				<div class="lg:col-span-8 flex flex-col justify-between gap-4">
					
					<!-- Device Info Card Grid -->
					<div class="rounded-[32px] bg-surface-container p-6 flex flex-col gap-4 shadow-sm">
						<div class="flex items-center justify-between">
							<div>
								<h3 class="text-lg font-bold text-on-surface">{activeDevice.name}</h3>
								<p class="text-xs text-on-surface-variant font-mono mt-0.5">Serial: {activeDevice.id}</p>
							</div>
							<span class="text-xs font-bold bg-primary/15 text-primary px-3 py-1 rounded-full uppercase tracking-wider">
								{activeDevice.status}
							</span>
						</div>

						<div class="grid grid-cols-2 sm:grid-cols-4 gap-3 text-xs">
							<div class="bg-surface-container-high p-3.5 rounded-2xl flex flex-col gap-1">
								<span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider flex items-center gap-1">
									<span class="material-symbols-outlined text-[14px] text-primary">android</span> OS Version
								</span>
								<span class="font-bold text-on-surface truncate">{activeDevice.androidVersion || 'Android 14'}</span>
							</div>

							<div class="bg-surface-container-high p-3.5 rounded-2xl flex flex-col gap-1">
								<span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider flex items-center gap-1">
									<span class="material-symbols-outlined text-[14px] text-emerald-400">battery_charging_full</span> Battery
								</span>
								<span class="font-bold text-on-surface">{activeDevice.batteryLevel || 90}% {activeDevice.isCharging ? '(Charging)' : ''}</span>
							</div>

							<div class="bg-surface-container-high p-3.5 rounded-2xl flex flex-col gap-1">
								<span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider flex items-center gap-1">
									<span class="material-symbols-outlined text-[14px] text-sky-400">aspect_ratio</span> Resolution
								</span>
								<span class="font-bold text-on-surface truncate">{activeDevice.resolution || '1440 x 3120'}</span>
							</div>

							<div class="bg-surface-container-high p-3.5 rounded-2xl flex flex-col gap-1">
								<span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider flex items-center gap-1">
									<span class="material-symbols-outlined text-[14px] text-purple-400">memory</span> Architecture
								</span>
								<span class="font-bold font-mono text-on-surface">{activeDevice.abi || 'arm64-v8a'}</span>
							</div>
						</div>

						<!-- Storage Bar -->
						<div class="bg-surface-container-high p-4 rounded-2xl flex flex-col gap-2">
							<div class="flex justify-between items-center text-xs font-semibold">
								<span class="text-on-surface-variant flex items-center gap-1.5">
									<span class="material-symbols-outlined text-[16px] text-primary">sd_storage</span> Internal Storage
								</span>
								<span class="text-on-surface font-mono">{activeDevice.storageUsedGb || 64} GB / {activeDevice.storageTotalGb || 128} GB used</span>
							</div>
							<div class="w-full h-2 bg-surface-container-highest rounded-full overflow-hidden">
								<div class="h-full bg-primary rounded-full" style="width: {Math.round(((activeDevice.storageUsedGb || 64) / (activeDevice.storageTotalGb || 128)) * 100)}%"></div>
							</div>
						</div>
					</div>

					<!-- Quick Action Navigation Hub -->
					<div class="rounded-[32px] bg-surface-container p-5 grid grid-cols-2 sm:grid-cols-4 gap-3 shadow-sm">
						<button
							onclick={() => goto('/files')}
							class="flex items-center gap-3 p-3.5 rounded-2xl bg-surface-container-high hover:bg-surface-container-highest transition-all group text-left"
						>
							<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/15 text-primary shrink-0">
								<span class="material-symbols-outlined text-[20px]">folder_open</span>
							</div>
							<div>
								<span class="text-xs font-bold text-on-surface block group-hover:text-primary transition-colors">File Explorer</span>
								<span class="text-[10px] text-on-surface-variant">Browse storage</span>
							</div>
						</button>

						<button
							onclick={() => goto('/apps')}
							class="flex items-center gap-3 p-3.5 rounded-2xl bg-surface-container-high hover:bg-surface-container-highest transition-all group text-left"
						>
							<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/15 text-primary shrink-0">
								<span class="material-symbols-outlined text-[20px]">apps</span>
							</div>
							<div>
								<span class="text-xs font-bold text-on-surface block group-hover:text-primary transition-colors">App Manager</span>
								<span class="text-[10px] text-on-surface-variant">Install / Remove</span>
							</div>
						</button>

						<button
							onclick={() => goto('/shell')}
							class="flex items-center gap-3 p-3.5 rounded-2xl bg-surface-container-high hover:bg-surface-container-highest transition-all group text-left"
						>
							<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/15 text-primary shrink-0">
								<span class="material-symbols-outlined text-[20px]">terminal</span>
							</div>
							<div>
								<span class="text-xs font-bold text-on-surface block group-hover:text-primary transition-colors">ADB Shell</span>
								<span class="text-[10px] text-on-surface-variant">Interactive CLI</span>
							</div>
						</button>

						<button
							onclick={() => goto('/flasher')}
							class="flex items-center gap-3 p-3.5 rounded-2xl bg-surface-container-high hover:bg-surface-container-highest transition-all group text-left"
						>
							<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/15 text-primary shrink-0">
								<span class="material-symbols-outlined text-[20px]">bolt</span>
							</div>
							<div>
								<span class="text-xs font-bold text-on-surface block group-hover:text-primary transition-colors">Flasher Studio</span>
								<span class="text-[10px] text-on-surface-variant">ROM & Sideload</span>
							</div>
						</button>
					</div>

				</div>
			</section>
		{/if}

		<!-- Performance Monitor Panel -->
		<section class="rounded-[32px] bg-surface-container p-6 flex flex-col gap-6 shadow-sm shrink-0">
			<header class="flex items-center justify-between">
				<h3 class="text-base font-bold tracking-tight text-on-surface flex items-center gap-3">
					<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary-container text-on-primary-container">
						<span class="material-symbols-outlined text-[20px]">monitoring</span>
					</div>
					Live Performance Dashboard
				</h3>
				<div class="flex items-center gap-2 rounded-full bg-surface-container-high px-4 py-1.5 shadow-xs">
					<span class="h-2 w-2 rounded-full bg-emerald-500 animate-pulse"></span>
					<span class="text-xs font-bold text-on-surface font-mono">Uptime {uptimeStr}</span>
				</div>
			</header>

			<div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
				<div class="rounded-2xl bg-surface-container-high p-5 transition-colors hover:bg-surface-container-highest">
					<div class="flex justify-between items-center mb-1">
						<span class="text-xs font-semibold text-on-surface-variant">CPU Usage</span>
						<span class="text-sm font-bold text-emerald-400 font-mono">{currentCpu}%</span>
					</div>
					<Sparkline data={dataCpu} color="#4ADE80" height={60} />
				</div>

				<div class="rounded-2xl bg-surface-container-high p-5 transition-colors hover:bg-surface-container-highest">
					<div class="flex justify-between items-center mb-1">
						<span class="text-xs font-semibold text-on-surface-variant">RAM Memory ({currentMem}%)</span>
						<span class="text-sm font-bold text-purple-400 font-mono">{memStr}</span>
					</div>
					<Sparkline data={dataMem} color="#A78BFA" height={60} />
				</div>

				<div class="rounded-2xl bg-surface-container-high p-5 transition-colors hover:bg-surface-container-highest">
					<div class="flex justify-between items-center mb-1">
						<span class="text-xs font-semibold text-on-surface-variant font-mono">FPS ({topPackageName})</span>
						<span class="text-sm font-bold text-amber-400 font-mono">{currentFps > 0 ? `${currentFps} FPS` : '60 FPS'}</span>
					</div>
					<Sparkline data={dataFps} color="#F97316" height={60} />
				</div>
			</div>
		</section>

	</div>
</main>

<style>
	@keyframes fadeIn {
		from {
			opacity: 0;
			transform: scale(0.99);
		}
		to {
			opacity: 1;
			transform: scale(1);
		}
	}
	.animate-fade-in {
		animation: fadeIn 0.15s cubic-bezier(0.2, 0.8, 0.2, 1) forwards;
	}
</style>
