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

	interface LogEntry {
		id: number;
		time: string;
		pid: number;
		tag: string;
		level: 'I' | 'D' | 'W' | 'E' | 'F';
		message: string;
	}

	// Devices State
	let devices = $state<Device[]>([]);
	let loading = $state(true);
	let error = $state('');
	let selectedDevice = $state('');
	let activeDevice = $derived(devices.find((d) => d.id === selectedDevice) || devices[0] || null);
	let hasWirelessDevice = $derived(devices.some((d) => d.connection === 'Wireless ADB'));

let deviceInfo: {
	model: string | null;
	brand: string | null;
	sdk_version: string | null;
	battery_level: number | null;
	storage_total: number | null;
	storage_used: number | null;
	ram_total: number | null;
	device: string | null;
	product: string | null;
	ip_address: string | null;
} | null = $state(null);

	// Performance Monitoring State
	let dataCpu = $state(Array(40).fill(0));
	let dataMem = $state(Array(40).fill(0));
	let currentCpu = $state(18);
	let currentMem = $state(45);
	let memStr = $state('3.8 GB / 8 GB');
	let uptimeStr = $state('0:00:00');
	let lastCpuData: unknown = null;

	// Recent ADB operations — empty until real ops happen
	// ponytail: hardcoded dummy data removed; wire to audit backend when ready
	interface AuditOp { time: string; icon: string; title: string; detail: string; }
	let auditOps = $state<AuditOp[]>([]);

	// Wireless ADB Quick Connect state — persist across navigation
	let wirelessIp = $state(localStorage.getItem('oxide:wirelessIp') ?? '192.168.1.100');
	let wirelessPort = $state(localStorage.getItem('oxide:wirelessPort') ?? '5555');
	let wirelessConnecting = $state(false);
	let wirelessStatus = $state('');
	let wirelessError = $state('');

	// TCP/IP enable state
	let tcpipPort = $state(localStorage.getItem('oxide:tcpipPort') ?? '5555');

	$effect(() => { localStorage.setItem('oxide:wirelessIp', wirelessIp); });
	$effect(() => { localStorage.setItem('oxide:wirelessPort', wirelessPort); });
	$effect(() => { localStorage.setItem('oxide:tcpipPort', tcpipPort); });
	let tcpipEnabling = $state(false);
	let tcpipStatus = $state('');
	let tcpipError = $state('');

	// Logcat System Trace State
	let logcatLines = $state<LogEntry[]>([]);
	let logcatFilter = $state<'ALL' | 'I' | 'D' | 'W' | 'E'>('ALL');
	let logcatSearch = $state('');
	let logcatStreaming = $state(true);
	let logcatContainer: HTMLDivElement | undefined;
	let logCounter = 1;

	let filteredLogcat = $derived(
		logcatLines.filter((l) => {
			const matchesLevel = logcatFilter === 'ALL' || l.level === logcatFilter;
			const matchesSearch =
				!logcatSearch ||
				l.tag.toLowerCase().includes(logcatSearch.toLowerCase()) ||
				l.message.toLowerCase().includes(logcatSearch.toLowerCase());
			return matchesLevel && matchesSearch;
		})
	);

	const sampleTags = [
		{ tag: 'ActivityManager', level: 'I', msg: 'Displayed com.android.chrome/.MainActivity: +240ms' },
		{ tag: 'BatteryService', level: 'I', msg: 'UPDATE battery level=92%, status=Charging, temp=31.2°C' },
		{ tag: 'WifiService', level: 'D', msg: 'WifiDevice connected to 5GHz_Home_Network (RSSI: -52dBm)' },
		{ tag: 'PackageManager', level: 'I', msg: 'Package Verification result: ALLOW for com.whatsapp' },
		{ tag: 'InputDispatcher', level: 'D', msg: 'Delivered MotionEvent ACTION_DOWN to window (x=540, y=1200)' },
		{ tag: 'SurfaceFlinger', level: 'W', msg: 'Frame 10294 dropped: VSYNC missed by 4.2ms' },
		{ tag: 'AudioService', level: 'D', msg: 'AudioDeviceChanged: Bluetooth A2DP active' },
		{ tag: 'OxideDaemon', level: 'I', msg: 'ADB Server socket 127.0.0.1:5037 heartbeat OK' },
		{ tag: 'SystemUI', level: 'W', msg: 'NotificationShade expand event triggered' },
		{ tag: 'MediaProvider', level: 'E', msg: 'Failed to access /sdcard/DCIM/.thumbnails: Permission denied' }
	];

	let intervalId: ReturnType<typeof setInterval> | undefined;
	let logcatIntervalId: ReturnType<typeof setInterval> | undefined;
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
		if (selectedDevice) await fetchDeviceInfo(selectedDevice);

		dataCpu = Array(40).fill(0).map(() => Math.floor(Math.random() * 20 + 15));
		dataMem = Array(40).fill(0).map(() => Math.floor(Math.random() * 15 + 40));

		// Seed initial logcat lines
		const initialLogs: LogEntry[] = sampleTags.slice(0, 6).map((s, idx) => ({
			id: idx + 1,
			time: new Date(Date.now() - (6 - idx) * 2000).toTimeString().split(' ')[0] + '.102',
			pid: 1200 + idx * 45,
			tag: s.tag,
			level: s.level as 'I' | 'D' | 'W' | 'E' | 'F',
			message: s.msg
		}));
		logcatLines = initialLogs;
		logCounter = 7;

		document.addEventListener('visibilitychange', onVisibilityChange);

		let isPolling = false;

		intervalId = setInterval(async () => {
			if (!isPageVisible || !isTauri || !invoke || !selectedDevice || isPolling) return;

			isPolling = true;
			try {
				const perf = await safeInvoke<{ cpu: { user: number; system: number; idle: number }; ram: { total: number; used: number; free: number }; battery: { level: number; status: string } | null } | null>('get_perf', { serial: selectedDevice }).catch(() => null);

				if (perf) {
					if (perf.ram) {
						const totalKb = perf.ram.total / 1024;
						const availKb = (perf.ram.total - perf.ram.used) / 1024;
						currentMem = Math.round(((totalKb - availKb) / totalKb) * 100);
						const usedGb = (perf.ram.used / (1024 * 1024 * 1024)).toFixed(1);
						const totalGb = (perf.ram.total / (1024 * 1024 * 1024)).toFixed(1);
						memStr = `${usedGb} GB / ${totalGb} GB`;
						dataMem = [...dataMem.slice(1), currentMem];
					}

					if (perf.cpu) {
						currentCpu = Math.round(perf.cpu.user + perf.cpu.system);
						dataCpu = [...dataCpu.slice(1), currentCpu];
					}
				}
			} catch (e) {
				console.warn('Polling status info:', e);
			} finally {
				isPolling = false;
			}
		}, 1500);

		// Live Logcat streaming interval
		logcatIntervalId = setInterval(() => {
			if (!logcatStreaming || !isPageVisible) return;
			const sample = sampleTags[Math.floor(Math.random() * sampleTags.length)];
			const newLog: LogEntry = {
				id: logCounter++,
				time: new Date().toTimeString().split(' ')[0] + '.' + Math.floor(Math.random() * 900 + 100),
				pid: 1400 + Math.floor(Math.random() * 800),
				tag: sample.tag,
				level: sample.level as 'I' | 'D' | 'W' | 'E' | 'F',
				message: sample.msg
			};
			logcatLines = [...logcatLines.slice(-150), newLog];

			if (logcatContainer) {
				logcatContainer.scrollTop = logcatContainer.scrollHeight;
			}
		}, 2000);
	});

	onDestroy(() => {
		if (typeof document !== 'undefined') {
			document.removeEventListener('visibilitychange', onVisibilityChange);
		}
		if (intervalId) clearInterval(intervalId);
		if (logcatIntervalId) clearInterval(logcatIntervalId);
	});

	$effect(() => {
		if (selectedDevice) fetchDeviceInfo(selectedDevice);
	});

	async function fetchDeviceInfo(serial: string) {
		try {
			if (isTauri && invoke) {
				deviceInfo = await safeInvoke<any>('get_device_info', { serial });
			}
		} catch (e) {
			console.warn('get_device_info:', e);
			deviceInfo = null;
		}
	}

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
						name: d.serial,
						status: 'Online',
						connection: d.serial.includes('.') ? 'Wireless ADB' : 'USB',
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

	async function handleWirelessConnect() {
		if (!wirelessIp.trim()) return;
		wirelessConnecting = true;
		wirelessStatus = '';
		wirelessError = '';
		try {
			if (isTauri && invoke) {
				await safeInvoke('wireless_connect', { host: `${wirelessIp.trim()}:${wirelessPort.trim() || '5555'}` });
			}
			wirelessStatus = `Connected to ${wirelessIp}:${wirelessPort}`;
			await loadDevices();
		} catch (e) {
			wirelessError = `Connection failed: ${e}`;
		} finally {
			wirelessConnecting = false;
		}
	}

	async function handleEnableTcpip() {
		if (!selectedDevice) return;
		tcpipEnabling = true;
		tcpipStatus = '';
		tcpipError = '';
		const port = parseInt(tcpipPort.trim() || '5555', 10);
		try {
			if (isTauri && invoke) {
				await safeInvoke('wireless_tcpip', { serial: selectedDevice, port });
			}
			tcpipStatus = `TCP/IP enabled on port ${port}. You can unplug USB now.`;
		} catch (e) {
			tcpipError = `Failed: ${e}`;
		} finally {
			tcpipEnabling = false;
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
			<!-- Main Device Overview Section (Compact Device Mockup + Specs Hub) -->
			<section class="grid grid-cols-1 lg:grid-cols-12 gap-5 items-stretch">
				
				<!-- Left Column: Compact Device Mockup Asset (Col 3) -->
				<div class="lg:col-span-3 flex flex-col items-center justify-center rounded-[32px] bg-surface-container p-5 relative overflow-hidden shadow-sm min-h-[340px]">
					<img
						src="/device.svg"
						alt="Connected Device Mockup"
						class="w-[170px] max-h-[290px] object-contain drop-shadow-xl transition-all duration-300 hover:scale-[1.03]"
					/>
					<div class="mt-3 flex items-center gap-2 bg-surface-container-high px-3.5 py-1 rounded-full shadow-xs">
						<span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
						<span class="text-xs font-bold text-on-surface truncate max-w-[120px]">{deviceInfo?.model || deviceInfo?.product || 'Unknown'}</span>
						<span class="text-[9px] font-mono text-on-surface-variant font-bold">({activeDevice?.connection || 'USB'})</span>
					</div>
				</div>

				<!-- Right Column: Extended Device Specs & Quick Action Buttons (Col 9) -->
				<div class="lg:col-span-9 flex flex-col justify-between gap-4">
					
					<!-- Device Info Card Grid -->
					<div class="rounded-[32px] bg-surface-container p-6 flex flex-col gap-4 shadow-sm">
						<div class="flex items-center justify-between">
							<div>
								<h3 class="text-lg font-bold text-on-surface">{deviceInfo?.model || deviceInfo?.product || activeDevice.name}</h3>
								<p class="text-xs text-on-surface-variant font-mono mt-0.5">{activeDevice.connection === 'Wireless ADB' ? `Wireless: ${activeDevice.id}` : `Serial: ${activeDevice.id}`}</p>
							</div>
							<span class="text-xs font-bold bg-primary/15 text-primary px-3 py-1 rounded-full uppercase tracking-wider">
								{activeDevice.status}
							</span>
						</div>

						<div class="grid grid-cols-2 sm:grid-cols-4 gap-3 text-xs">
							<div class="bg-surface-container-high p-3 rounded-2xl flex flex-col gap-1">
								<span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider flex items-center gap-1">
									<span class="material-symbols-outlined text-[14px] text-primary">android</span> OS Version
								</span>
								<span class="font-bold text-on-surface truncate">{deviceInfo?.sdk_version ? `Android ${deviceInfo.sdk_version}` : 'Android ?'}</span>
							</div>

							<div class="bg-surface-container-high p-3 rounded-2xl flex flex-col gap-1">
								<span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider flex items-center gap-1">
									<span class="material-symbols-outlined text-[14px] text-emerald-400">battery_charging_full</span> Battery
								</span>
								<span class="font-bold text-on-surface">{deviceInfo?.battery_level ?? '?'}%</span>
							</div>

							<div class="bg-surface-container-high p-3 rounded-2xl flex flex-col gap-1">
								<span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider flex items-center gap-1">
									<span class="material-symbols-outlined text-[14px] text-sky-400">aspect_ratio</span> Codename
								</span>
								<span class="font-bold text-on-surface truncate">{deviceInfo?.device || '-'}</span>
							</div>

							<div class="bg-surface-container-high p-3 rounded-2xl flex flex-col gap-1">
								<span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider flex items-center gap-1">
									<span class="material-symbols-outlined text-[14px] text-purple-400">memory</span> Product
								</span>
								<span class="font-bold font-mono text-on-surface">{deviceInfo?.product || '-'}</span>
							</div>
						</div>

						<!-- Storage Bar -->
						<div class="bg-surface-container-high p-3.5 rounded-2xl flex flex-col gap-2">
							<div class="flex justify-between items-center text-xs font-semibold">
								<span class="text-on-surface-variant flex items-center gap-1.5">
									<span class="material-symbols-outlined text-[16px] text-primary">sd_storage</span> Internal Storage
								</span>
								<span class="text-on-surface font-mono">{deviceInfo?.storage_used ? (deviceInfo.storage_used / 1073741824).toFixed(0) : '?'} GB / {deviceInfo?.storage_total ? (deviceInfo.storage_total / 1073741824).toFixed(0) : '?'} GB used</span>
							</div>
							<div class="w-full h-2 bg-surface-container-highest rounded-full overflow-hidden">
								<div class="h-full bg-primary rounded-full" style="width: {deviceInfo?.storage_total && deviceInfo?.storage_used ? Math.round((deviceInfo.storage_used / deviceInfo.storage_total) * 100) : 0}%"></div>
							</div>
						</div>
					</div>

					<!-- Quick Action Navigation Hub -->
					<div class="rounded-[32px] bg-surface-container p-4 grid grid-cols-2 sm:grid-cols-4 gap-3 shadow-sm">
						<button
							onclick={() => goto('/files')}
							class="flex items-center gap-3 p-3 rounded-2xl bg-surface-container-high hover:bg-surface-container-highest transition-all group text-left"
						>
							<div class="flex h-9 w-9 items-center justify-center rounded-xl bg-primary/15 text-primary shrink-0">
								<span class="material-symbols-outlined text-[18px]">folder_open</span>
							</div>
							<div class="min-w-0">
								<span class="text-xs font-bold text-on-surface block group-hover:text-primary transition-colors truncate">File Explorer</span>
								<span class="text-[10px] text-on-surface-variant block truncate">Browse storage</span>
							</div>
						</button>

						<button
							onclick={() => goto('/apps')}
							class="flex items-center gap-3 p-3 rounded-2xl bg-surface-container-high hover:bg-surface-container-highest transition-all group text-left"
						>
							<div class="flex h-9 w-9 items-center justify-center rounded-xl bg-primary/15 text-primary shrink-0">
								<span class="material-symbols-outlined text-[18px]">apps</span>
							</div>
							<div class="min-w-0">
								<span class="text-xs font-bold text-on-surface block group-hover:text-primary transition-colors truncate">App Manager</span>
								<span class="text-[10px] text-on-surface-variant block truncate">Install / Remove</span>
							</div>
						</button>

						<button
							onclick={() => goto('/shell')}
							class="flex items-center gap-3 p-3 rounded-2xl bg-surface-container-high hover:bg-surface-container-highest transition-all group text-left"
						>
							<div class="flex h-9 w-9 items-center justify-center rounded-xl bg-primary/15 text-primary shrink-0">
								<span class="material-symbols-outlined text-[18px]">terminal</span>
							</div>
							<div class="min-w-0">
								<span class="text-xs font-bold text-on-surface block group-hover:text-primary transition-colors truncate">ADB Shell</span>
								<span class="text-[10px] text-on-surface-variant block truncate">Interactive CLI</span>
							</div>
						</button>

						<button
							onclick={() => goto('/flasher')}
							class="flex items-center gap-3 p-3 rounded-2xl bg-surface-container-high hover:bg-surface-container-highest transition-all group text-left"
						>
							<div class="flex h-9 w-9 items-center justify-center rounded-xl bg-primary/15 text-primary shrink-0">
								<span class="material-symbols-outlined text-[18px]">bolt</span>
							</div>
							<div class="min-w-0">
								<span class="text-xs font-bold text-on-surface block group-hover:text-primary transition-colors truncate">Flasher Studio</span>
								<span class="text-[10px] text-on-surface-variant block truncate">ROM & Sideload</span>
							</div>
						</button>
					</div>

				</div>
			</section>
		{/if}

		{#if activeDevice}
		<!-- Performance Monitor Panel -->
		<section class="rounded-[32px] bg-surface-container p-6 flex flex-col gap-5 shadow-sm shrink-0">
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

			<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
				<div class="rounded-2xl bg-surface-container-high p-5 transition-colors hover:bg-surface-container-highest overflow-hidden">
					<div class="flex justify-between items-center mb-2">
						<span class="text-xs font-semibold text-on-surface-variant">CPU Usage</span>
						<span class="text-sm font-bold text-emerald-400 font-mono">{currentCpu}%</span>
					</div>
					<Sparkline data={dataCpu} color="#4ADE80" height={60} />
				</div>

				<div class="rounded-2xl bg-surface-container-high p-5 transition-colors hover:bg-surface-container-highest overflow-hidden">
					<div class="flex justify-between items-center mb-2">
						<span class="text-xs font-semibold text-on-surface-variant">RAM Memory ({currentMem}%)</span>
						<span class="text-sm font-bold text-purple-400 font-mono">{memStr}</span>
					</div>
					<Sparkline data={dataMem} color="#A78BFA" height={60} />
				</div>
			</div>
		</section>

		<!-- WIRELESS ADB SETUP HUB (Directly on Home Dashboard) -->
		<section class="grid grid-cols-1 lg:grid-cols-12 gap-5 shrink-0">
			<!-- Connect via IP Address (Col 6) -->
			<div class="lg:col-span-6 rounded-[32px] bg-surface-container p-6 flex flex-col justify-between gap-5 shadow-sm">
				<div class="flex items-start justify-between gap-3">
					<div>
						<h3 class="text-base font-bold tracking-tight text-on-surface flex items-center gap-2 mb-1">
							<div class="flex h-9 w-9 items-center justify-center rounded-xl bg-primary-container text-on-primary-container shrink-0">
								<span class="material-symbols-outlined text-[18px]">cell_tower</span>
							</div>
							Connect via IP Address
						</h3>
						<p class="text-xs text-on-surface-variant leading-relaxed mt-1">
							Connect directly over Wi-Fi network using target device IP and ADB port (usually 5555).
						</p>
					</div>
					{#if hasWirelessDevice}
						<span class="flex items-center gap-1.5 text-[10px] font-bold bg-emerald-500/15 text-emerald-400 px-2.5 py-1 rounded-full whitespace-nowrap shrink-0">
							<span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span> Connected
						</span>
					{:else}
						<span class="flex items-center gap-1.5 text-[10px] font-bold bg-on-surface-variant/10 text-on-surface-variant/60 px-2.5 py-1 rounded-full whitespace-nowrap shrink-0">
							<span class="w-1.5 h-1.5 rounded-full bg-on-surface-variant/40"></span> Disconnected
						</span>
					{/if}
				</div>

				<div class="grid grid-cols-3 sm:grid-cols-4 gap-3 text-xs">
					<div class="col-span-2 sm:col-span-3">
						<label for="home-ip-input" class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider block mb-1">Device IP Address</label>
						<input
							id="home-ip-input"
							type="text"
							bind:value={wirelessIp}
							placeholder="192.168.1.100"
							class="w-full bg-surface-container-high rounded-2xl px-4 py-2.5 text-xs font-mono text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 shadow-xs"
						/>
					</div>
					<div class="col-span-1">
						<label for="home-port-input" class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider block mb-1">Port</label>
						<input
							id="home-port-input"
							type="text"
							bind:value={wirelessPort}
							placeholder="5555"
							class="w-full bg-surface-container-high rounded-2xl px-3.5 py-2.5 text-xs font-mono text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 shadow-xs"
						/>
					</div>
				</div>

				<div class="flex items-center gap-3 pt-1">
					<button
						onclick={handleWirelessConnect}
						class="flex-1 flex items-center justify-center gap-2 rounded-2xl bg-primary py-3 text-xs font-bold text-on-primary hover:brightness-110 transition-all shadow-sm disabled:opacity-50"
						disabled={wirelessConnecting || !wirelessIp.trim()}
					>
						{#if wirelessConnecting}
							<span class="animate-spin h-3.5 w-3.5 border-2 border-on-primary border-t-transparent rounded-full"></span>
							Connecting...
						{:else}
							<span class="material-symbols-outlined text-[18px]">wifi</span>
							Connect Device
						{/if}
					</button>
				</div>
				{#if wirelessStatus}
					<p class="text-xs text-emerald-400 font-medium mt-1 flex items-center gap-1.5">
						<span class="material-symbols-outlined text-[14px]">check_circle</span> {wirelessStatus}
					</p>
				{/if}
				{#if wirelessError}
					<p class="text-xs text-error font-medium mt-1 flex items-center gap-1.5">
						<span class="material-symbols-outlined text-[14px]">error</span> {wirelessError}
					</p>
				{/if}
			</div>

			<!-- Enable TCP/IP Port (Requires USB) (Col 6) -->
			<div class="lg:col-span-6 rounded-[32px] bg-surface-container p-6 flex flex-col justify-between gap-5 shadow-sm">
				<div class="flex items-start justify-between gap-3">
					<div>
						<h3 class="text-base font-bold tracking-tight text-on-surface flex items-center gap-2 mb-1">
							<div class="flex h-9 w-9 items-center justify-center rounded-xl bg-primary-container text-on-primary-container shrink-0">
								<span class="material-symbols-outlined text-[18px]">usb</span>
							</div>
							Enable TCP/IP Port (Requires USB)
						</h3>
						<p class="text-xs text-on-surface-variant leading-relaxed mt-1">
							Connect device via USB first to enable wireless port 5555. Once activated, cable can be unplugged.
						</p>
					</div>
					{#if tcpipStatus}
						<span class="flex items-center gap-1.5 text-[10px] font-bold bg-emerald-500/15 text-emerald-400 px-2.5 py-1 rounded-full whitespace-nowrap shrink-0">
							<span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span> Enabled
						</span>
					{:else}
						<span class="flex items-center gap-1.5 text-[10px] font-bold bg-on-surface-variant/10 text-on-surface-variant/60 px-2.5 py-1 rounded-full whitespace-nowrap shrink-0">
							<span class="w-1.5 h-1.5 rounded-full bg-on-surface-variant/40"></span> Inactive
						</span>
					{/if}
				</div>

				<div class="grid grid-cols-3 sm:grid-cols-4 gap-3 text-xs">
					<div class="col-span-1">
						<label for="home-tcp-port" class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider block mb-1">Port</label>
						<input
							id="home-tcp-port"
							type="text"
							bind:value={tcpipPort}
							placeholder="5555"
							class="w-full bg-surface-container-high rounded-2xl px-3.5 py-2.5 text-xs font-mono text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 shadow-xs"
						/>
					</div>
				</div>

				<div class="flex items-center gap-3 pt-1">
					<button
						onclick={handleEnableTcpip}
						class="flex-1 flex items-center justify-center gap-2 rounded-2xl bg-surface-container-high hover:bg-surface-container-highest py-3 text-xs font-bold text-on-surface transition-all shadow-xs disabled:opacity-50"
						disabled={tcpipEnabling || !selectedDevice}
					>
						{#if tcpipEnabling}
							<span class="animate-spin h-3.5 w-3.5 border-2 border-on-surface border-t-transparent rounded-full"></span>
							Enabling...
						{:else}
							<span class="material-symbols-outlined text-[18px] text-amber-400">settings_input_antenna</span>
							Enable TCP/IP Port
						{/if}
					</button>
				</div>
				{#if !selectedDevice}
					<p class="text-[10px] text-on-surface-variant/60 font-medium flex items-center gap-1">
						<span class="material-symbols-outlined text-[12px]">info</span> Connect a USB device first
					</p>
				{/if}
				{#if tcpipStatus}
					<p class="text-xs text-emerald-400 font-medium mt-1 flex items-center gap-1.5">
						<span class="material-symbols-outlined text-[14px]">check_circle</span> {tcpipStatus}
					</p>
				{/if}
				{#if tcpipError}
					<p class="text-xs text-error font-medium mt-1 flex items-center gap-1.5">
						<span class="material-symbols-outlined text-[14px]">error</span> {tcpipError}
					</p>
				{/if}
			</div>
		</section>
		{/if}

		<!-- Bottom Section: Quick Power Actions & Audit Operations -->
		<section class="grid grid-cols-1 lg:grid-cols-12 gap-5 shrink-0">
			{#if activeDevice}
			<!-- Power & Reboot Controls (Col 5) -->
			<div class="lg:col-span-5 rounded-[32px] bg-surface-container p-6 flex flex-col justify-between gap-4 shadow-sm">
				<div>
					<h3 class="text-sm font-bold text-on-surface flex items-center gap-2 mb-1">
						<span class="material-symbols-outlined text-primary text-[20px]">power_settings_new</span>
						Quick Power & Reboot Control
					</h3>
					<p class="text-xs text-on-surface-variant">Send instant reboot commands to connected target</p>
				</div>

				<div class="grid grid-cols-2 gap-2 text-xs">
					<button
						onclick={() => rebootDevice('normal')}
						class="flex items-center justify-center gap-2 p-3 rounded-xl bg-surface-container-high hover:bg-surface-container-highest text-on-surface font-bold transition-all"
					>
						<span class="material-symbols-outlined text-[16px] text-emerald-400">restart_alt</span>
						Reboot System
					</button>
					<button
						onclick={() => rebootDevice('bootloader')}
						class="flex items-center justify-center gap-2 p-3 rounded-xl bg-surface-container-high hover:bg-surface-container-highest text-on-surface font-bold transition-all"
					>
						<span class="material-symbols-outlined text-[16px] text-amber-400">memory</span>
						Bootloader Mode
					</button>
				</div>

				<button
					onclick={() => rebootDevice('recovery')}
					class="flex items-center justify-center gap-2 p-3 rounded-xl bg-surface-container-high hover:bg-surface-container-highest text-on-surface font-bold text-xs transition-all w-full"
				>
					<span class="material-symbols-outlined text-[16px] text-sky-400">build</span>
					Reboot Recovery (Sideload)
				</button>
			</div>
			{/if}

			<!-- Recent Activity Stream -->
			<div class="{activeDevice ? 'lg:col-span-7' : 'lg:col-span-12'} rounded-[32px] bg-surface-container p-6 flex flex-col justify-between gap-3 shadow-sm">
				<div class="flex items-center justify-between">
					<h3 class="text-sm font-bold text-on-surface flex items-center gap-2">
						<span class="material-symbols-outlined text-primary text-[20px]">history</span>
						Recent ADB Operations
					</h3>
					{#if auditOps.length > 0}
					<button onclick={() => goto('/audit')} class="text-[11px] font-bold text-primary hover:underline">
						View Full Audit Log ↗
					</button>
					{/if}
				</div>

				{#if auditOps.length === 0}
				<div class="flex flex-col items-center justify-center py-8 text-center">
					<span class="material-symbols-outlined text-[32px] text-on-surface-variant/40 mb-2">inbox</span>
					<p class="text-xs text-on-surface-variant/60 font-medium">No operations yet</p>
				</div>
				{:else}
				<div class="flex flex-col gap-2">
					{#each auditOps as act, idx (idx)}
						<div class="flex items-center justify-between p-2.5 rounded-xl bg-surface-container-high/60 text-xs">
							<div class="flex items-center gap-2.5">
								<span class="material-symbols-outlined text-[16px] text-primary">{act.icon}</span>
								<div>
									<span class="font-bold text-on-surface">{act.title}</span>
									<span class="text-[10px] text-on-surface-variant font-mono block">{act.detail}</span>
								</div>
							</div>
							<span class="text-[10px] font-mono text-on-surface-variant/70">{act.time}</span>
						</div>
					{/each}
				</div>
				{/if}
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
