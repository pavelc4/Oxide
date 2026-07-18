<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import Sparkline from '$lib/components/Sparkline.svelte';
	let invoke: any;
	let isTauri = $state(false);

	let devices: any[] = $state([]);
	let loading = $state(true);
	let error = $state('');
	let selectedDevice = $state('');

	let dataCpu = $state(Array(40).fill(0));
	let dataMem = $state(Array(40).fill(0));
	let dataFps = $state(Array(40).fill(0));
	let dataCores = $state(Array(8).fill(0).map(() => Array(25).fill(0)));

	let coreUsages = Array(8).fill(0);
	let coreSpeeds = Array(8).fill(0);
	let topPackageName = $state('Browser');
	let currentFps = $state(0);
	let currentCpu = $state(0);
	let currentMem = $state(0);
	let memStr = $state('0MB');
	let uptimeStr = $state('0:00:00:00');
	let lastFpsData: any = null;
	let lastCpuData: any = null;

	let packages: string[] = $state([]);
	let processes: any[] = $state([]);
	let showSystemProcesses = $state(false);

	let intervalId: any;
	let isPageVisible = true;

	function onVisibilityChange() {
		isPageVisible = !document.hidden;
	}

	onMount(async () => {
		try {
			const tauriApi = await import('@tauri-apps/api/core');
			invoke = tauriApi.invoke;
			isTauri = true;
		} catch (e) {
			console.warn('Tauri not available, using mock mode');
			isTauri = false;
		}

		await loadDevices();

		dataCpu = Array(40).fill(0).map(() => Math.floor(Math.random() * 20 + 20));
		dataMem = Array(40).fill(0).map(() => Math.floor(Math.random() * 20 + 50));
		dataFps = Array(40).fill(0).map(() => Math.floor(Math.random() * 10));

		document.addEventListener('visibilitychange', onVisibilityChange);

		let isPolling = false;

		intervalId = setInterval(async () => {
			if (!isPageVisible || !isTauri || !invoke || !selectedDevice || isPolling) return;

			isPolling = true;
			try {
				const [topPkg, perf] = await Promise.all([
					invoke('get_top_package', { serial: selectedDevice }),
					invoke('get_performance_profile', { serial: selectedDevice })
				]);

				if (topPkg && topPkg.name) {
					topPackageName = topPkg.name;
				}

				if (perf.memory && perf.memory.Ok) {
					const memInfo = perf.memory.Ok;
					currentMem = Math.round(
						((memInfo.total_kb - memInfo.available_kb) / memInfo.total_kb) * 100
					);
					memStr = `${Math.round((memInfo.total_kb - memInfo.available_kb) / 1024)}MB`;
					dataMem = [...dataMem.slice(1), currentMem];
				}

				if (perf.uptime && perf.uptime.Ok) {
					let ut = perf.uptime.Ok;
					let days = Math.floor(ut / 86400);
					let hours = Math.floor((ut % 86400) / 3600);
					let minutes = Math.floor((ut % 3600) / 60);
					let seconds = Math.floor(ut % 60);
					uptimeStr = `${hours}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
					if (days > 0) uptimeStr = `${days}d ` + uptimeStr;
				}

				if (perf.cpu && perf.cpu.Ok && perf.cpu.Ok.length > 0) {
					const currentCpus = perf.cpu.Ok;

					if (lastCpuData !== null && currentCpus.length > 0 && lastCpuData.length > 0) {
						const curr = currentCpus[0].times;
						const prev = lastCpuData[0].times;

						const c_total = curr.user + curr.nice + curr.sys + curr.idle + curr.iowait + curr.irq + curr.softirq;
						const p_total = prev.user + prev.nice + prev.sys + prev.idle + prev.iowait + prev.irq + prev.softirq;

						const totalDelta = c_total - p_total;
						const idleDelta = curr.idle + curr.iowait - (prev.idle + prev.iowait);

						if (totalDelta > 0) {
							currentCpu = Math.round(100 * (1 - idleDelta / totalDelta));
						}

						for (let i = 1; i < currentCpus.length; i++) {
							let cpuName = currentCpus[i].name;
							if (!cpuName || !cpuName.startsWith('cpu')) continue;

							let idxStr = cpuName.replace('cpu', '');
							let idx = parseInt(idxStr);
							if (isNaN(idx) || idx < 0 || idx >= 8) continue;

							let c_prv_cpu = lastCpuData.find((c: any) => c.name === cpuName);

							if (c_prv_cpu) {
								let c_cur = currentCpus[i].times;
								let c_prv = c_prv_cpu.times;

								let core_tot =
									c_cur.user + c_cur.nice + c_cur.sys + c_cur.idle + c_cur.iowait + c_cur.irq + c_cur.softirq -
									(c_prv.user + c_prv.nice + c_prv.sys + c_prv.idle + c_prv.iowait + c_prv.irq + c_prv.softirq);
								let core_idle = c_cur.idle + c_cur.iowait - (c_prv.idle + c_prv.iowait);

								let u = 0;
								if (core_tot > 0) {
									u = Math.round(100 * (1 - core_idle / core_tot));
								}

								coreUsages[idx] = u;
								coreSpeeds[idx] = currentCpus[i].speed_mhz || 0;

								let newCores = [...dataCores];
								newCores[idx] = [...newCores[idx].slice(1), u];
								dataCores = newCores;
							}
						}
					}

					lastCpuData = currentCpus;
					dataCpu = [...dataCpu.slice(1), currentCpu];
				}

				if (perf.fps && perf.fps.Ok) {
					const fpsData = perf.fps.Ok;
					if (fpsData.flips !== null && lastFpsData !== null && fpsData.flips > lastFpsData.flips) {
						const deltaFlips = fpsData.flips - lastFpsData.flips;
						const deltaTime = fpsData.timestamp_ms - lastFpsData.timestamp_ms;
						if (deltaTime > 0) {
							currentFps = Math.round((deltaFlips * 1000) / deltaTime);
						}
					} else if (lastFpsData !== null && fpsData.flips === lastFpsData.flips) {
						currentFps = 0;
					}
					lastFpsData = fpsData;

					dataFps = [...dataFps.slice(1), currentFps];
				}
			} catch (e) {
				console.error('Polling error', e);
				let errStr = String(e);
				if (errStr.includes('device') && errStr.includes('not found')) {
					await loadDevices();
				} else {
					error = errStr;
				}
			} finally {
				isPolling = false;
			}
		}, 1000);
	});

	onDestroy(() => {
		document.removeEventListener('visibilitychange', onVisibilityChange);
		if (intervalId) {
			clearInterval(intervalId);
		}
	});

	async function loadDevices() {
		loading = true;
		error = '';
		try {
			if (isTauri && invoke) {
				const rustDevices = await invoke('get_devices');
				if (rustDevices) {
					devices = rustDevices.map((d: any) => ({
						id: d.serial,
						name: d.model || d.serial,
						status: 'Online',
						connection: 'USB',
						isPixel: false
					}));
				} else {
					devices = [];
				}
			} else {
				devices = [];
			}

			if (devices.length > 0) {
				selectedDevice = devices[0].id;
				await loadPackages();
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

	async function loadPackages() {
		if (!selectedDevice) return;
		try {
			if (isTauri && invoke) {
				const rustProcs: any = await invoke('list_processes', {
					serial: selectedDevice,
					appsOnly: !showSystemProcesses
				});
				if (rustProcs && rustProcs.length > 0) {
					processes = rustProcs.slice(0, 50).map((p: any) => ({
						name: p.name,
						pid: p.pid.toString(),
						cpu: p.cpu,
						mem: p.mem,
						isSys: p.user === 'root' || p.user === 'system'
					}));
				}
			} else {
				packages = ['com.example.app1', 'com.example.app2'];
			}
		} catch (e) {
			error = String(e);
		}
	}
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2">
	<div
		class="flex flex-1 flex-col overflow-y-auto rounded-[32px] bg-surface-container-low p-6 lg:p-10 relative will-change-transform"
	>
		<header class="mb-8 flex justify-between items-center">
			<h2 class="text-2xl font-bold tracking-tight text-on-surface flex items-center gap-4">
				KIRA
				{#if !isTauri}
					<span class="text-xs bg-error text-on-error px-3 py-1 rounded-full font-medium tracking-normal"
						>MOCK MODE</span
					>
				{/if}
			</h2>
			<button
				onclick={() => {
					const theme =
						document.documentElement.getAttribute('data-theme') === 'light' ? 'dark' : 'light';
					document.documentElement.setAttribute('data-theme', theme);
				}}
				class="text-xs bg-surface-variant px-4 py-2 rounded-full text-on-surface font-medium transition-colors hover:brightness-110"
			>
				<span class="material-symbols-outlined text-[16px] align-middle mr-1">palette</span> Theme
			</button>
		</header>

		{#if error}
			<div class="bg-error/20 text-error p-4 rounded-xl mb-4 font-medium break-words whitespace-pre-wrap">
				{error}
			</div>
		{/if}

		{#if devices.length === 0}
			<div
				class="flex flex-col items-center justify-center p-12 mt-10 rounded-[28px] bg-surface-container text-center"
			>
				<div
					class="flex h-20 w-20 items-center justify-center rounded-full bg-surface-container-highest text-on-surface-variant mb-6"
				>
					<span class="material-symbols-outlined text-[40px] opacity-80">usb_off</span>
				</div>
				<h3 class="text-xl font-bold tracking-tight text-on-surface mb-2">No Devices Connected</h3>
				<p class="text-sm text-on-surface-variant max-w-md mx-auto mb-6">
					Please connect an Android device via USB and enable USB Debugging in Developer Options.
				</p>
				<button
					class="flex items-center gap-2 rounded-full bg-primary px-6 py-3 text-sm font-medium text-on-primary transition-all hover:brightness-110"
					onclick={loadDevices}
				>
					<span class="material-symbols-outlined text-[18px]">refresh</span> Refresh Devices
				</button>
			</div>
		{/if}

		{#if devices.length > 0}
			<section class="relative mb-8 flex flex-col rounded-[28px] bg-surface-container p-8">
				<div class="mb-6">
					<h1 class="text-[28px] font-bold tracking-tight text-on-surface leading-tight">
						{devices[0].name}
					</h1>
					<p class="mt-2 text-sm text-on-surface-variant font-medium">
						Device connected via {devices[0].connection}
					</p>
				</div>
				<div class="z-10 flex gap-4">
					<button
						class="flex items-center gap-2 rounded-2xl bg-primary px-6 py-3 text-sm font-medium text-on-primary transition-all hover:brightness-110"
					>
						<span class="material-symbols-outlined text-[18px]">restart_alt</span> Reboot
					</button>
					<a
						href="/shell?serial={encodeURIComponent(selectedDevice)}&name={encodeURIComponent(
							devices[0]?.name || ''
						)}"
						class="flex items-center gap-2 rounded-2xl px-6 py-3 text-sm font-medium text-on-surface transition-all hover:bg-surface-variant no-underline"
					>
						<span class="material-symbols-outlined text-[18px]">terminal</span> Shell
					</a>
				</div>
			</section>
		{/if}

		<section id="device-list" class="flex flex-col gap-3 mb-12">
			{#each devices as d}
				<div
					class="group flex items-center justify-between rounded-[24px] bg-surface-container p-5 transition-all hover:bg-surface-container-high"
				>
					<div class="flex items-center gap-5">
						<div
							class="flex h-14 w-14 items-center justify-center rounded-[16px] bg-surface-container-high text-on-surface transition-colors group-hover:bg-surface-container-highest"
						>
							<span class="material-symbols-outlined text-[28px] opacity-80"
								>{d.isPixel ? 'smartphone' : 'phone_android'}</span
							>
						</div>
						<div class="flex flex-col gap-1">
							<h3 class="text-base font-semibold text-on-surface tracking-wide">{d.name}</h3>
							<div class="flex items-center gap-2 text-[13px] text-on-surface-variant font-medium">
								<span
									class={d.status === 'Fastboot'
										? 'text-secondary'
										: d.status === 'Offline'
											? 'text-error'
											: 'text-primary'}>{d.status}</span
								><span class="opacity-50">•</span><span>{d.connection}</span>
							</div>
						</div>
					</div>
				</div>
			{/each}
		</section>

		<section class="mb-10">
			<div class="flex flex-col rounded-[24px] bg-surface-container p-6">
				<header class="mb-6 flex items-center justify-between">
					<h3 class="text-lg font-bold tracking-tight text-on-surface flex items-center gap-3">
						<div
							class="flex h-10 w-10 items-center justify-center rounded-[14px] bg-primary-container text-on-primary-container"
						>
							<span class="material-symbols-outlined text-[20px]">monitoring</span>
						</div>
						Performance
					</h3>
					<div class="flex items-center gap-2 rounded-full bg-surface-container-high px-4 py-2">
						<span class="h-2 w-2 rounded-full bg-primary"></span>
						<span class="text-xs font-medium text-on-surface">Uptime {uptimeStr}</span>
					</div>
				</header>

				<div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
					<div
						class="rounded-[20px] bg-surface-container-high p-5 transition-colors hover:bg-surface-container-highest contain-layout"
					>
						<div class="flex justify-between items-center mb-1">
							<span class="text-xs font-medium text-on-surface-variant">CPU Overall</span>
							<span class="text-sm font-bold text-primary">{currentCpu}%</span>
						</div>
						<Sparkline data={dataCpu} color="#4ADE80" height={60} />
					</div>

					<div
						class="rounded-[20px] bg-surface-container-high p-5 transition-colors hover:bg-surface-container-highest contain-layout"
					>
						<div class="flex justify-between items-center mb-1">
							<span class="text-xs font-medium text-on-surface-variant">Memory ({currentMem}%)</span
							>
							<span class="text-sm font-bold text-tertiary">{memStr}</span>
						</div>
						<Sparkline data={dataMem} color="#A78BFA" height={60} />
					</div>

					<div
						class="rounded-[20px] bg-surface-container-high p-5 transition-colors hover:bg-surface-container-highest contain-layout"
					>
						<div class="flex justify-between items-center mb-1">
							<span class="text-xs font-medium text-on-surface-variant">FPS {topPackageName}</span>
							<span class="text-sm font-bold text-secondary"
								>{currentFps > 0 ? currentFps : '-'}</span
							>
						</div>
						<Sparkline data={dataFps} color="#F97316" height={60} />
					</div>
				</div>

				<div class="grid grid-cols-2 lg:grid-cols-4 gap-4 mt-4">
					{#each Array(8) as _, i}
						<div
							class="rounded-[16px] bg-surface-container-high p-4 transition-colors hover:bg-surface-container-highest contain-layout"
						>
							<div class="flex justify-between items-center mb-2">
								<span class="text-[11px] font-medium text-on-surface-variant"
									>CPU{i}
									<span class="ml-1">{coreSpeeds[i] ? coreSpeeds[i] + 'MHz' : '~'}</span></span
								>
								<span class="text-[12px] font-bold text-primary">{coreUsages[i]}%</span>
							</div>
							<Sparkline data={dataCores[i]} color="#4ADE80" height={35} />
						</div>
					{/each}
				</div>
			</div>
		</section>
	</div>
</main>
