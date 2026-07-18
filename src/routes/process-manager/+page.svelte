<script lang="ts">
	import { onMount } from 'svelte';
	let invoke: any;
	let isTauri = false;

	let devices: any[] = [];
	let selectedDevice = '';
	let processes: any[] = [];
	let showSystemProcesses = false;
	let error = '';

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
	});

	async function loadDevices() {
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
			}
		} catch (e) {
			error = String(e);
		}
	}
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2">
	<div
		class="flex flex-1 flex-col overflow-y-auto rounded-[32px] bg-surface-container-low p-6 lg:p-10 relative"
	>
		<header class="mb-8 flex justify-between items-center">
			<h2 class="text-2xl font-bold tracking-tight text-on-surface flex items-center gap-4">
				Process Manager
				{#if !isTauri}
					<span
						class="text-xs bg-error text-on-error px-3 py-1 rounded-full font-medium tracking-normal border border-error/30"
						>MOCK MODE</span
					>
				{/if}
			</h2>
		</header>

		{#if error}
			<div
				class="bg-error/20 text-error border border-error/50 p-4 rounded-xl mb-4 font-medium break-words whitespace-pre-wrap"
			>
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
					Please connect an Android device via USB and ensure USB Debugging is enabled in Developer
					Options.
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
			<section class="mb-10">
				<div class="flex flex-col rounded-[24px] bg-surface-container p-6">
					<header class="mb-6 flex items-center justify-between">
						<div class="flex items-center gap-4">
							<h3 class="text-lg font-bold tracking-tight text-on-surface flex items-center gap-3">
								<div
									class="flex h-10 w-10 items-center justify-center rounded-[14px] bg-secondary-container text-on-secondary-container"
								>
									<span class="material-symbols-outlined text-[20px]">memory</span>
								</div>
								{devices[0].name}
							</h3>

							<button
								class="text-xs font-medium px-3 py-1.5 rounded-full transition-colors flex items-center gap-2 {showSystemProcesses
									? 'bg-primary/10 text-primary'
									: 'bg-surface-variant text-on-surface-variant'}"
								onclick={() => {
									showSystemProcesses = !showSystemProcesses;
									loadPackages();
								}}
							>
								<span class="material-symbols-outlined text-[14px]">
									{showSystemProcesses ? 'toggle_on' : 'toggle_off'}
								</span>
								Show System
							</button>
						</div>

						<div class="relative">
							<span
								class="material-symbols-outlined absolute left-4 top-1/2 -translate-y-1/2 text-on-surface-variant text-[18px]"
								>search</span
							>
							<input
								type="text"
								placeholder="Search processes..."
								class="bg-surface-container-high rounded-full pl-10 pr-4 py-2.5 text-sm text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all w-64"
							/>
						</div>
					</header>

					<div
						class="flex items-center justify-between px-5 py-2 text-xs font-semibold uppercase tracking-wider text-on-surface-variant mb-2"
					>
						<div class="w-1/3">Package Name</div>
						<div class="flex w-1/2 items-center justify-between pr-10">
							<span class="w-16">PID</span><span class="w-20">CPU</span><span
								class="w-24 text-right">Memory</span
							>
						</div>
					</div>

					<div class="flex flex-col gap-2">
						{#each processes as proc}
							<div
								class="group flex items-center justify-between rounded-[16px] bg-surface-container-high px-4 py-3 transition-colors hover:bg-surface-container-highest cursor-pointer"
							>
								<div class="flex w-1/3 items-center gap-4 pr-4 pl-1">
									<div
										class="flex h-9 w-9 shrink-0 items-center justify-center rounded-[12px] {proc.isSys
											? 'bg-tertiary-container text-on-tertiary-container'
											: 'bg-surface-variant text-on-surface'}"
									>
										<span class="material-symbols-outlined text-[18px]"
											>{proc.isSys ? 'settings_applications' : 'apps'}</span
										>
									</div>
									<span class="truncate text-[13px] font-medium text-on-surface">{proc.name}</span>
								</div>
								<div class="flex w-1/2 items-center justify-between text-[13px]">
									<span class="w-16 text-on-surface-variant">{proc.pid}</span>
									<span
										class="w-20 {parseFloat(proc.cpu) > 10
											? 'text-error font-semibold'
											: 'text-on-surface-variant'}">{proc.cpu}%</span
									>
									<span class="w-24 text-right text-on-surface-variant">{proc.mem}</span>
								</div>
							</div>
						{/each}
					</div>
				</div>
			</section>
		{/if}
	</div>
</main>
