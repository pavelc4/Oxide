<script lang="ts">
	/* eslint-disable svelte/no-navigation-without-resolve */
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | undefined;
	let isTauri = $state(false);

	interface Device {
		id: string;
		name: string;
		status: string;
		mode: 'fastboot' | 'adb' | 'recovery';
		connection: string;
	}

	interface FlashStep {
		partition: string;
		image_file: string;
		selected?: boolean;
		size?: string;
	}

	interface FlashPlan {
		folder: string;
		steps: FlashStep[];
	}

	// Active Tab state
	type TabType = 'rom_planner' | 'manual_flasher' | 'sideload' | 'fastboot_console';
	let activeTab = $state<TabType>('rom_planner');

	// Devices State
	let devices = $state<Device[]>([]);
	let selectedDevice = $state<string>('');
	let loading = $state(true);
	let error = $state('');
	let infoMessage = $state('');

	// Slot Controller State
	let activeSlot = $state('a');
	let loadingSlot = $state(false);

	// Manual Flash & Sideload State
	let manualPartition = $state('boot');
	let manualImagePath = $state('');
	let targetSlot = $state<'active' | 'a' | 'b' | 'both'>('active');
	let sideloadZipPath = $state('');
	let flashingManual = $state(false);
	let sideloading = $state(false);

	// ROM Flash Plan State
	let romFolderPath = $state('');
	let scannedPlan = $state<FlashPlan | null>(null);
	let scanningRom = $state(false);
	let executingPlan = $state(false);
	let currentFlashStepIndex = $state(-1);
	let flashProgress = $state(0);

	// Fastboot CLI State
	let customCommand = $state('getvar all');
	let cliLogs = $state<Array<{ type: 'cmd' | 'output' | 'error' | 'info'; text: string }>>([]);
	let executingCli = $state(false);

	// Popular partitions preset
	const partitionPresets = [
		'boot', 'init_boot', 'vendor_boot', 'recovery', 'dtbo', 'system', 'vendor', 'product', 'super', 'vbmeta'
	];

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
				const rustDevices = await safeInvoke<Array<{ serial: string; model?: string; mode?: string }>>('get_devices');
				if (rustDevices && rustDevices.length > 0) {
					devices = rustDevices.map((d) => ({
						id: d.serial,
						name: d.model || d.serial,
						status: 'Online',
						mode: (d.mode === 'bootloader' || d.mode === 'fastboot') ? 'fastboot' : 'adb',
						connection: 'USB'
					}));
				} else {
					devices = [];
				}
			} else {
				devices = [
					{ id: 'fastboot-pixel-8', name: 'Google Pixel 8 Pro', status: 'Online', mode: 'fastboot', connection: 'USB' },
					{ id: 'adb-galaxy-s24', name: 'Samsung Galaxy S24 Ultra', status: 'Online', mode: 'adb', connection: 'USB' }
				];
			}

			if (devices.length > 0) {
				selectedDevice = devices[0].id;
				await fetchActiveSlot();
			} else {
				selectedDevice = '';
				activeSlot = 'Unknown';
			}
		} catch (e) {
			error = String(e);
			devices = [];
		} finally {
			loading = false;
		}
	}

	async function selectDeviceChanged(id: string) {
		selectedDevice = id;
		await fetchActiveSlot();
	}

	async function fetchActiveSlot() {
		if (!selectedDevice) return;
		loadingSlot = true;
		error = '';
		try {
			if (isTauri && invoke) {
				const slot = await safeInvoke<string>('fastboot_get_active_slot', { serial: selectedDevice });
				activeSlot = slot || 'a';
			} else {
				activeSlot = 'a';
			}
		} catch {
			activeSlot = 'a';
		} finally {
			loadingSlot = false;
		}
	}

	async function setActiveSlot(slot: string) {
		if (!selectedDevice) return;
		loadingSlot = true;
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				await safeInvoke('fastboot_set_active_slot', { serial: selectedDevice, slot });
			}
			activeSlot = slot;
			infoMessage = `Slot ${slot.toUpperCase()} set as active boot partition.`;
		} catch (e) {
			error = `Failed to set active slot: ${e}`;
		} finally {
			loadingSlot = false;
		}
	}

	async function continueBoot() {
		if (!selectedDevice) return;
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				await safeInvoke('fastboot_continue_boot', { serial: selectedDevice });
			}
			infoMessage = 'Boot command sent. Device is initializing system startup.';
		} catch (e) {
			error = `Failed to continue boot: ${e}`;
		}
	}

	async function wipeUserdata() {
		if (!selectedDevice) return;
		if (!confirm('⚠️ FACTORY RESET WARNING!\n\nThis will completely WIPE all userdata and cache partitions on the device.\nAre you sure you want to proceed?')) return;
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				await safeInvoke('fastboot_wipe_userdata', { serial: selectedDevice });
			}
			infoMessage = 'User data partition wiped successfully (Factory Reset Complete).';
		} catch (e) {
			error = `Failed to wipe data: ${e}`;
		}
	}

	async function rebootDevice(mode: 'normal' | 'bootloader' | 'recovery') {
		if (!selectedDevice) return;
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				await safeInvoke('reboot_device', { serial: selectedDevice, mode });
			}
			infoMessage = `Rebooting device to ${mode.toUpperCase()} mode...`;
		} catch (e) {
			error = `Failed to reboot: ${e}`;
		}
	}

	async function manualFlashPartition() {
		if (!selectedDevice || !manualImagePath.trim()) return;
		flashingManual = true;
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				await safeInvoke('fastboot_flash_partition', {
					serial: selectedDevice,
					partition: manualPartition.trim(),
					filePath: manualImagePath.trim()
				});
			} else {
				await new Promise((resolve) => setTimeout(resolve, 1500));
			}
			infoMessage = `Successfully flashed [${manualPartition}] partition from file ${manualImagePath.split('/').pop() || manualImagePath}`;
			manualImagePath = '';
		} catch (e) {
			error = `Flash failed: ${e}`;
		} finally {
			flashingManual = false;
		}
	}

	async function runSideload() {
		if (!selectedDevice || !sideloadZipPath.trim()) return;
		sideloading = true;
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				await safeInvoke('adb_sideload_cmd', {
					serial: selectedDevice,
					zipPath: sideloadZipPath.trim()
				});
			} else {
				await new Promise((resolve) => setTimeout(resolve, 2500));
			}
			infoMessage = `OTA ZIP Package sideloaded successfully: ${sideloadZipPath.split('/').pop() || sideloadZipPath}`;
			sideloadZipPath = '';
		} catch (e) {
			error = `Sideload failed: ${e}`;
		} finally {
			sideloading = false;
		}
	}

	async function scanRomFolder() {
		if (!romFolderPath.trim()) return;
		scanningRom = true;
		error = '';
		scannedPlan = null;
		try {
			if (isTauri && invoke) {
				scannedPlan = await safeInvoke<FlashPlan>('scan_flash_folder_cmd', {
					folderPath: romFolderPath.trim()
				});
			} else {
				await new Promise((resolve) => setTimeout(resolve, 600));
				scannedPlan = {
					folder: romFolderPath.trim(),
					steps: [
						{ partition: 'boot', image_file: 'boot.img', selected: true, size: '64 MB' },
						{ partition: 'init_boot', image_file: 'init_boot.img', selected: true, size: '8 MB' },
						{ partition: 'vendor_boot', image_file: 'vendor_boot.img', selected: true, size: '64 MB' },
						{ partition: 'dtbo', image_file: 'dtbo.img', selected: true, size: '16 MB' },
						{ partition: 'vbmeta', image_file: 'vbmeta.img', selected: true, size: '8 KB' },
						{ partition: 'super', image_file: 'super.img', selected: true, size: '4.2 GB' },
						{ partition: 'system', image_file: 'system.img', selected: true, size: '2.1 GB' },
						{ partition: 'vendor', image_file: 'vendor.img', selected: true, size: '850 MB' }
					]
				};
			}
			if (scannedPlan && scannedPlan.steps) {
				scannedPlan.steps = scannedPlan.steps.map((s) => ({ ...s, selected: true }));
			}
		} catch (e) {
			error = `Failed to scan ROM folder: ${e}`;
		} finally {
			scanningRom = false;
		}
	}

	async function executeFlashPlan() {
		if (!scannedPlan || !selectedDevice) return;
		const selectedSteps = scannedPlan.steps.filter((s) => s.selected !== false);
		if (selectedSteps.length === 0) {
			error = 'Please select at least one partition to flash.';
			return;
		}

		if (!confirm(`🚀 Confirm Flashing ROM?\n\nThis will execute ${selectedSteps.length} partition flash operations on target device [${selectedDevice}].`)) return;

		executingPlan = true;
		error = '';
		infoMessage = '';
		currentFlashStepIndex = 0;
		flashProgress = 0;

		try {
			for (let i = 0; i < selectedSteps.length; i++) {
				currentFlashStepIndex = i;
				flashProgress = Math.round(((i + 1) / selectedSteps.length) * 100);
				const step = selectedSteps[i];
				const imgPath = `${scannedPlan.folder}/${step.image_file}`;

				if (isTauri && invoke) {
					await safeInvoke('fastboot_flash_partition', {
						serial: selectedDevice,
						partition: step.partition,
						filePath: imgPath
					});
				} else {
					await new Promise((resolve) => setTimeout(resolve, 1200));
				}
			}
			infoMessage = `🎉 ROM Flashing Complete! ${selectedSteps.length} partitions updated successfully.`;
			scannedPlan = null;
		} catch (e) {
			error = `Flash plan failed at step ${currentFlashStepIndex + 1}: ${e}`;
		} finally {
			executingPlan = false;
			currentFlashStepIndex = -1;
			flashProgress = 0;
		}
	}

	async function executeCliCommand() {
		if (!selectedDevice || !customCommand.trim()) return;
		executingCli = true;
		error = '';
		const cmd = customCommand.trim();
		cliLogs = [...cliLogs, { type: 'cmd', text: `fastboot -s ${selectedDevice} ${cmd}` }];

		try {
			let output = '';
			if (isTauri && invoke) {
				output = await safeInvoke<string>('fastboot_custom_command', {
					serial: selectedDevice,
					command: cmd
				});
			} else {
				await new Promise((resolve) => setTimeout(resolve, 400));
				if (cmd.includes('getvar')) {
					output = 'version-baseband: g5300g-123456\nversion-bootloader: rip-0.1\nactive-slot: a\nsecure: yes\nproduct: mock_device\nfinished. total time: 0.042s';
				} else if (cmd.includes('devices')) {
					output = `${selectedDevice}\tfastboot`;
				} else {
					output = 'OKAY [  0.082s ]\nfinished. total time: 0.085s';
				}
			}
			cliLogs = [...cliLogs, { type: 'output', text: output }];
			customCommand = '';
		} catch (e) {
			cliLogs = [...cliLogs, { type: 'error', text: `Error: ${e}` }];
		} finally {
			executingCli = false;
		}
	}

	function handlePresetClick(p: string) {
		manualPartition = p;
	}

	function handleSampleRomPath() {
		romFolderPath = '/home/user/downloads/Pixel_8_Factory_ROM';
		scanRomFolder();
	}
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2 h-screen overflow-hidden">
	<div class="flex flex-1 flex-col overflow-hidden rounded-[32px] bg-surface-container-low p-6 lg:p-8 relative gap-6">

		<!-- Top Navigation & Header -->
		<header class="flex flex-col md:flex-row md:items-center justify-between gap-4 shrink-0 pb-2 border-b border-outline-variant/15">
			<div class="flex items-center gap-4">
				<button
					onclick={() => goto('/')}
					class="flex h-10 w-10 items-center justify-center rounded-full bg-surface-container hover:bg-surface-container-high text-on-surface-variant transition-all hover:scale-105 active:scale-95"
					title="Back to dashboard"
				>
					<span class="material-symbols-outlined text-[20px]">arrow_back</span>
				</button>
				<div>
					<div class="flex items-center gap-3">
						<h2 class="text-2xl font-bold tracking-tight text-on-surface">Flasher Studio</h2>
						{#if !isTauri}
							<span class="text-[10px] bg-warning/15 text-warning border border-warning/30 px-2.5 py-0.5 rounded-full font-bold uppercase tracking-wider">MOCK PREVIEW</span>
						{/if}
					</div>
					<p class="text-xs text-on-surface-variant/80 font-medium mt-0.5">Fastboot partitioning, ROM flasher planner & ADB sideload manager</p>
				</div>
			</div>

			<!-- Right Control Group: Active Device Picker & Refresh -->
			<div class="flex items-center gap-3">
				<div class="flex items-center gap-2 bg-surface-container px-3.5 py-1.5 rounded-full border border-outline-variant/20 shadow-sm">
					<span class="relative flex h-2.5 w-2.5">
						<span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
						<span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500"></span>
					</span>
					<select
						value={selectedDevice}
						onchange={(e) => selectDeviceChanged(e.currentTarget.value)}
						class="bg-transparent border-none text-xs font-semibold text-on-surface focus:outline-none cursor-pointer pr-2"
					>
						{#if devices.length === 0}
							<option value="">No Device Detected</option>
						{:else}
							{#each devices as dev (dev.id)}
								<option value={dev.id}>{dev.name} ({dev.mode.toUpperCase()})</option>
							{/each}
						{/if}
					</select>
				</div>

				<button
					onclick={loadDevices}
					class="flex h-9 w-9 items-center justify-center rounded-full bg-surface-container hover:bg-surface-container-high text-on-surface-variant transition-all hover:scale-105 active:scale-95 border border-outline-variant/20"
					title="Rescan Devices"
				>
					<span class="material-symbols-outlined text-[18px] {loading ? 'animate-spin' : ''}">refresh</span>
				</button>
			</div>
		</header>

		<!-- Alert Banners -->
		{#if error}
			<div class="bg-error/15 text-error border border-error/30 p-3.5 rounded-2xl font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[20px]">error</span>
				<div class="flex-1 break-words font-semibold">{error}</div>
				<button onclick={() => (error = '')} class="hover:opacity-80 text-[10px] font-bold uppercase tracking-wider bg-error/20 px-2 py-1 rounded-lg">Dismiss</button>
			</div>
		{/if}

		{#if infoMessage}
			<div class="bg-primary/10 text-primary border border-primary/20 p-3.5 rounded-2xl font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[20px]">check_circle</span>
				<div class="flex-1 break-words font-semibold">{infoMessage}</div>
				<button onclick={() => (infoMessage = '')} class="hover:opacity-80 text-[10px] font-bold uppercase tracking-wider bg-primary/20 px-2 py-1 rounded-lg">Dismiss</button>
			</div>
		{/if}

		<!-- Main Studio Grid -->
		<div class="flex-1 grid grid-cols-1 lg:grid-cols-12 gap-6 overflow-hidden min-h-0">
			
			<!-- Left Column: Navigation Tabs & Main Workspace (Col 8) -->
			<div class="lg:col-span-8 flex flex-col gap-4 overflow-hidden min-h-0">
				
				<!-- Mode Navigation Tabs -->
				<nav class="flex gap-2 p-1.5 bg-surface-container rounded-2xl shrink-0 overflow-x-auto">
					<button
						onclick={() => (activeTab = 'rom_planner')}
						class="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-bold transition-all whitespace-nowrap {activeTab === 'rom_planner' ? 'bg-primary text-on-primary shadow-sm' : 'text-on-surface-variant hover:text-on-surface hover:bg-surface-container-high'}"
					>
						<span class="material-symbols-outlined text-[18px]">folder_zip</span>
						ROM Flash Planner
					</button>

					<button
						onclick={() => (activeTab = 'manual_flasher')}
						class="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-bold transition-all whitespace-nowrap {activeTab === 'manual_flasher' ? 'bg-primary text-on-primary shadow-sm' : 'text-on-surface-variant hover:text-on-surface hover:bg-surface-container-high'}"
					>
						<span class="material-symbols-outlined text-[18px]">flash_on</span>
						Single Partition
					</button>

					<button
						onclick={() => (activeTab = 'sideload')}
						class="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-bold transition-all whitespace-nowrap {activeTab === 'sideload' ? 'bg-primary text-on-primary shadow-sm' : 'text-on-surface-variant hover:text-on-surface hover:bg-surface-container-high'}"
					>
						<span class="material-symbols-outlined text-[18px]">install_mobile</span>
						ADB Sideload
					</button>

					<button
						onclick={() => (activeTab = 'fastboot_console')}
						class="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-bold transition-all whitespace-nowrap {activeTab === 'fastboot_console' ? 'bg-primary text-on-primary shadow-sm' : 'text-on-surface-variant hover:text-on-surface hover:bg-surface-container-high'}"
					>
						<span class="material-symbols-outlined text-[18px]">terminal</span>
						Fastboot Terminal
					</button>
				</nav>

				<!-- Tab 1: ROM Flash Planner -->
				{#if activeTab === 'rom_planner'}
					<div class="flex-1 rounded-[24px] bg-surface-container p-6 flex flex-col gap-5 overflow-hidden min-h-0 border border-outline-variant/10">
						<div>
							<h3 class="text-base font-bold text-on-surface flex items-center gap-2">
								<span class="material-symbols-outlined text-primary">auto_mode</span>
								Batch ROM Flash Planner
							</h3>
							<p class="text-xs text-on-surface-variant mt-1">Scan a factory ROM payload directory to automatically extract partitions and flash sequentially.</p>
						</div>

						<!-- Folder Path Scanner -->
						<div class="flex flex-col gap-2">
							<label for="rom-folder-input" class="text-[11px] font-bold text-on-surface-variant uppercase tracking-wider">ROM Directory Path</label>
							<div class="flex items-center gap-2">
								<div class="relative flex-1">
									<span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-on-surface-variant text-[18px]">folder</span>
									<input
										id="rom-folder-input"
										type="text"
										bind:value={romFolderPath}
										placeholder="e.g. /home/user/Downloads/Pixel8_Factory_ROM"
										class="w-full bg-surface-container-high border border-outline-variant/30 rounded-xl pl-10 pr-3 py-2 text-xs font-mono text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 transition-all"
										disabled={scanningRom || executingPlan}
									/>
								</div>
								<button
									onclick={scanRomFolder}
									class="rounded-xl bg-primary text-on-primary hover:brightness-110 px-5 py-2 text-xs font-bold transition-all disabled:opacity-40 shrink-0"
									disabled={scanningRom || executingPlan || !romFolderPath.trim()}
								>
									{#if scanningRom}
										<span class="flex items-center gap-1.5">
											<span class="animate-spin h-3.5 w-3.5 border-2 border-on-primary border-t-transparent rounded-full"></span>
											Scanning...
										</span>
									{:else}
										Scan Directory
									{/if}
								</button>
							</div>
							{#if !scannedPlan}
								<button onclick={handleSampleRomPath} class="text-[11px] text-primary hover:underline text-left font-medium w-fit mt-0.5">
									💡 Click here to load sample ROM plan preview
								</button>
							{/if}
						</div>

						<!-- Step List / Partitions Table -->
						{#if scannedPlan}
							<div class="flex-1 flex flex-col gap-3 min-h-0 overflow-hidden">
								<div class="flex items-center justify-between shrink-0">
									<span class="text-xs font-bold text-on-surface">Target Partition Queue ({scannedPlan.steps.length} files found)</span>
									<span class="text-[11px] text-on-surface-variant font-mono">Location: {scannedPlan.folder}</span>
								</div>

								<div class="flex-1 bg-surface-container-low rounded-2xl overflow-y-auto border border-outline-variant/20 p-2 flex flex-col gap-1.5">
									{#each scannedPlan.steps as step, idx (idx)}
										<label class="flex items-center justify-between p-2.5 rounded-xl hover:bg-surface-container-high/60 transition-all cursor-pointer border border-transparent {currentFlashStepIndex === idx ? 'bg-primary/15 border-primary/40 font-bold' : ''}">
											<div class="flex items-center gap-3">
												<input
													type="checkbox"
													bind:checked={step.selected}
													disabled={executingPlan}
													class="w-4 h-4 rounded text-primary focus:ring-primary/50"
												/>
												<div>
													<span class="text-xs font-bold font-mono text-on-surface">{step.partition}</span>
													<span class="text-[10px] text-on-surface-variant/70 font-mono block">{step.image_file}</span>
												</div>
											</div>
											{#if step.size}
												<span class="text-[11px] font-mono text-on-surface-variant px-2 py-0.5 bg-surface-container rounded-md">{step.size}</span>
											{/if}
										</label>
									{/each}
								</div>

								<!-- Progress Bar -->
								{#if executingPlan}
									<div class="flex flex-col gap-1.5 shrink-0 bg-primary/10 p-3 rounded-xl border border-primary/20">
										<div class="flex justify-between text-xs font-bold text-primary">
											<span>Flashing step {currentFlashStepIndex + 1} of {scannedPlan.steps.length}...</span>
											<span>{flashProgress}%</span>
										</div>
										<div class="w-full h-2 bg-surface-container-highest rounded-full overflow-hidden">
											<div class="h-full bg-primary transition-all duration-300 rounded-full" style="width: {flashProgress}%"></div>
										</div>
									</div>
								{/if}

								<!-- Action Execution -->
								<button
									onclick={executeFlashPlan}
									class="rounded-xl bg-primary text-on-primary hover:brightness-110 px-6 py-2.5 text-xs font-bold transition-all shadow-md disabled:opacity-50 shrink-0 w-full"
									disabled={executingPlan || !selectedDevice}
								>
									{#if executingPlan}
										Flashing Partition Sequence...
									{:else}
										Flash Selected ROM Partitions
									{/if}
								</button>
							</div>
						{/if}
					</div>
				{/if}

				<!-- Tab 2: Single Partition Flasher -->
				{#if activeTab === 'manual_flasher'}
					<div class="flex-1 rounded-[24px] bg-surface-container p-6 flex flex-col justify-between gap-5 overflow-y-auto border border-outline-variant/10">
						<div class="flex flex-col gap-5">
							<div>
								<h3 class="text-base font-bold text-on-surface flex items-center gap-2">
									<span class="material-symbols-outlined text-primary">flash_on</span>
									Single Partition Image Flasher
								</h3>
								<p class="text-xs text-on-surface-variant mt-1">Flash an individual partition image (.img / .bin) directly to the connected target device.</p>
							</div>

							<!-- Partition Quick Presets -->
							<div class="flex flex-col gap-2">
								<span class="text-[11px] font-bold text-on-surface-variant uppercase tracking-wider">Popular Partition Targets</span>
								<div class="flex flex-wrap gap-2">
									{#each partitionPresets as p}
										<button
											onclick={() => handlePresetClick(p)}
											class="px-3 py-1 rounded-xl text-xs font-mono font-bold transition-all border {manualPartition === p ? 'bg-primary text-on-primary border-primary' : 'bg-surface-container-high text-on-surface-variant border-outline-variant/20 hover:border-primary/50 hover:text-on-surface'}"
										>
											{p}
										</button>
									{/each}
								</div>
							</div>

							<!-- Custom Inputs -->
							<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
								<div class="flex flex-col gap-1.5">
									<label for="target-part-input" class="text-[11px] font-bold text-on-surface-variant uppercase tracking-wider">Partition Name</label>
									<input
										id="target-part-input"
										type="text"
										bind:value={manualPartition}
										placeholder="e.g. boot"
										class="bg-surface-container-high border border-outline-variant/30 rounded-xl px-3 py-2 text-xs font-mono font-bold text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40"
										disabled={flashingManual}
									/>
								</div>

								<div class="md:col-span-2 flex flex-col gap-1.5">
									<label for="img-file-path" class="text-[11px] font-bold text-on-surface-variant uppercase tracking-wider">Image File Path (.img)</label>
									<div class="relative">
										<span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-on-surface-variant text-[18px]">image</span>
										<input
											id="img-file-path"
											type="text"
											bind:value={manualImagePath}
											placeholder="/path/to/image.img"
											class="w-full bg-surface-container-high border border-outline-variant/30 rounded-xl pl-10 pr-3 py-2 text-xs font-mono text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40"
											disabled={flashingManual}
										/>
									</div>
								</div>
							</div>
						</div>

						<button
							onclick={manualFlashPartition}
							class="rounded-xl bg-primary text-on-primary hover:brightness-110 px-6 py-3 text-xs font-bold transition-all shadow-md disabled:opacity-50 w-full"
							disabled={flashingManual || !selectedDevice || !manualImagePath.trim()}
						>
							{#if flashingManual}
								<span class="flex items-center justify-center gap-2">
									<span class="animate-spin h-4 w-4 border-2 border-on-primary border-t-transparent rounded-full"></span>
									Flashing Partition [{manualPartition}]...
								</span>
							{:else}
								Flash Partition Image Now
							{/if}
						</button>
					</div>
				{/if}

				<!-- Tab 3: ADB Sideload -->
				{#if activeTab === 'sideload'}
					<div class="flex-1 rounded-[24px] bg-surface-container p-6 flex flex-col justify-between gap-5 border border-outline-variant/10">
						<div class="flex flex-col gap-5">
							<div>
								<h3 class="text-base font-bold text-on-surface flex items-center gap-2">
									<span class="material-symbols-outlined text-primary">install_mobile</span>
									ADB Sideload Package Manager
								</h3>
								<p class="text-xs text-on-surface-variant mt-1">Sideload OTA updates, system zip patches, or custom recovery packages when device is in Sideload Mode.</p>
							</div>

							<div class="bg-primary/5 border border-primary/20 rounded-2xl p-4 flex items-start gap-3">
								<span class="material-symbols-outlined text-primary text-[22px] shrink-0 mt-0.5">info</span>
								<div class="text-xs text-on-surface-variant">
									<p class="font-bold text-on-surface">Entering Sideload Mode:</p>
									<p class="mt-1">Make sure your phone is in Recovery Mode and select <code class="bg-surface-container-high px-1.5 py-0.5 rounded text-primary font-mono">Apply update from ADB</code>, or use the quick action button on the right side to reboot.</p>
								</div>
							</div>

							<div class="flex flex-col gap-1.5">
								<label for="zip-file-path" class="text-[11px] font-bold text-on-surface-variant uppercase tracking-wider">ZIP Package Path (.zip)</label>
								<div class="relative">
									<span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-on-surface-variant text-[18px]">archive</span>
									<input
										id="zip-file-path"
										type="text"
										bind:value={sideloadZipPath}
										placeholder="/path/to/update.zip"
										class="w-full bg-surface-container-high border border-outline-variant/30 rounded-xl pl-10 pr-3 py-2 text-xs font-mono text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40"
										disabled={sideloading}
									/>
								</div>
							</div>
						</div>

						<button
							onclick={runSideload}
							class="rounded-xl bg-primary text-on-primary hover:brightness-110 px-6 py-3 text-xs font-bold transition-all shadow-md disabled:opacity-50 w-full"
							disabled={sideloading || !selectedDevice || !sideloadZipPath.trim()}
						>
							{#if sideloading}
								<span class="flex items-center justify-center gap-2">
									<span class="animate-spin h-4 w-4 border-2 border-on-primary border-t-transparent rounded-full"></span>
									Sideloading ZIP Package...
								</span>
							{:else}
								Sideload Package Now
							{/if}
						</button>
					</div>
				{/if}

				<!-- Tab 4: Fastboot Terminal Console -->
				{#if activeTab === 'fastboot_console'}
					<div class="flex-1 rounded-[24px] bg-surface-container p-6 flex flex-col justify-between gap-4 overflow-hidden border border-outline-variant/10">
						<div>
							<h3 class="text-base font-bold text-on-surface flex items-center gap-2">
								<span class="material-symbols-outlined text-primary">terminal</span>
								Fastboot Interactive Console
							</h3>
							<p class="text-xs text-on-surface-variant mt-1">Execute direct fastboot protocol sub-commands against connected target.</p>
						</div>

						<!-- CLI Console Terminal Box -->
						<div class="flex-1 bg-neutral-950 rounded-2xl p-4 font-mono text-xs overflow-y-auto max-h-[320px] flex flex-col gap-1 border border-neutral-800 shadow-inner select-all">
							{#if cliLogs.length === 0}
								<span class="text-neutral-500">// Terminal ready. Type fastboot arguments below (e.g. getvar all).</span>
							{:else}
								{#each cliLogs as log, i (i)}
									{#if log.type === 'cmd'}
										<div class="text-emerald-400 font-bold">$ {log.text}</div>
									{:else if log.type === 'error'}
										<div class="text-rose-400">{log.text}</div>
									{:else}
										<div class="text-neutral-300 leading-relaxed whitespace-pre-wrap">{log.text}</div>
									{/if}
								{/each}
							{/if}
						</div>

						<form onsubmit={(e) => { e.preventDefault(); executeCliCommand(); }} class="flex gap-2 shrink-0">
							<div class="relative flex-1">
								<span class="absolute left-3 top-1/2 -translate-y-1/2 text-on-surface-variant/80 font-mono text-xs font-bold select-none">$ fastboot</span>
								<input
									type="text"
									bind:value={customCommand}
									placeholder="getvar all"
									class="bg-surface-container-high border border-outline-variant/40 rounded-xl pl-24 pr-3 py-2 text-xs font-mono text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 w-full"
									disabled={executingCli}
								/>
							</div>
							<button
								type="submit"
								class="rounded-xl bg-primary text-on-primary hover:brightness-110 px-5 py-2 text-xs font-bold transition-all disabled:opacity-50"
								disabled={executingCli || !selectedDevice || !customCommand.trim()}
							>
								Send Command
							</button>
						</form>
					</div>
				{/if}
			</div>

			<!-- Right Column: Device Quick Controls & Slot Management (Col 4) -->
			<div class="lg:col-span-4 flex flex-col gap-4 overflow-y-auto">
				
				<!-- A/B Slot Manager Panel -->
				<section class="rounded-[24px] bg-surface-container p-5 border border-outline-variant/10 flex flex-col gap-4">
					<div class="flex items-center justify-between">
						<h3 class="text-xs font-bold text-on-surface uppercase tracking-wider flex items-center gap-2">
							<span class="material-symbols-outlined text-primary text-[18px]">swap_horiz</span>
							Slot Manager (A/B)
						</h3>
						<span class="text-[10px] font-bold bg-primary/15 text-primary px-2.5 py-0.5 rounded-full uppercase font-mono">
							{#if loadingSlot}
								Updating...
							{:else}
								Slot {activeSlot.toUpperCase()}
							{/if}
						</span>
					</div>

					<div class="grid grid-cols-2 gap-2">
						<button
							onclick={() => setActiveSlot('a')}
							class="rounded-xl py-2 px-3 text-xs font-bold transition-all border {activeSlot === 'a' ? 'bg-primary text-on-primary border-primary shadow-sm' : 'bg-surface-container-high text-on-surface hover:bg-surface-container-highest border-outline-variant/20'}"
							disabled={loadingSlot || !selectedDevice}
						>
							Switch Slot A
						</button>
						<button
							onclick={() => setActiveSlot('b')}
							class="rounded-xl py-2 px-3 text-xs font-bold transition-all border {activeSlot === 'b' ? 'bg-primary text-on-primary border-primary shadow-sm' : 'bg-surface-container-high text-on-surface hover:bg-surface-container-highest border-outline-variant/20'}"
							disabled={loadingSlot || !selectedDevice}
						>
							Switch Slot B
						</button>
					</div>

					<button
						onclick={continueBoot}
						class="rounded-xl bg-surface-container-highest text-on-surface hover:bg-primary hover:text-on-primary transition-all text-xs font-bold py-2.5 w-full disabled:opacity-40 border border-outline-variant/20"
						disabled={!selectedDevice}
					>
						Continue Normal Booting
					</button>
				</section>

				<!-- Device State & Quick Reboot Actions -->
				<section class="rounded-[24px] bg-surface-container p-5 border border-outline-variant/10 flex flex-col gap-3">
					<h3 class="text-xs font-bold text-on-surface uppercase tracking-wider flex items-center gap-2 mb-1">
						<span class="material-symbols-outlined text-primary text-[18px]">power_settings_new</span>
						Reboot Controls
					</h3>

					<div class="grid grid-cols-2 gap-2">
						<button
							onclick={() => rebootDevice('normal')}
							class="rounded-xl bg-surface-container-high hover:bg-surface-container-highest text-on-surface py-2 text-xs font-bold transition-all border border-outline-variant/20 disabled:opacity-40"
							disabled={!selectedDevice}
						>
							System
						</button>
						<button
							onclick={() => rebootDevice('bootloader')}
							class="rounded-xl bg-surface-container-high hover:bg-surface-container-highest text-on-surface py-2 text-xs font-bold transition-all border border-outline-variant/20 disabled:opacity-40"
							disabled={!selectedDevice}
						>
							Bootloader
						</button>
					</div>

					<button
						onclick={() => rebootDevice('recovery')}
						class="rounded-xl bg-surface-container-high hover:bg-surface-container-highest text-on-surface py-2 text-xs font-bold transition-all border border-outline-variant/20 disabled:opacity-40 w-full"
						disabled={!selectedDevice}
					>
						Reboot Recovery (Sideload)
					</button>
				</section>

				<!-- Safety & Reset Action -->
				<section class="rounded-[24px] bg-error/5 p-5 border border-error/20 flex flex-col gap-3">
					<h3 class="text-xs font-bold text-error uppercase tracking-wider flex items-center gap-2">
						<span class="material-symbols-outlined text-error text-[18px]">warning</span>
						Dangerous Wipe Actions
					</h3>
					<p class="text-[11px] text-on-surface-variant">Perform factory reset by clearing user data & cache partitions.</p>
					<button
						onclick={wipeUserdata}
						class="rounded-xl bg-error/15 text-error hover:bg-error/25 border border-error/30 transition-all text-xs font-bold py-2.5 w-full disabled:opacity-40"
						disabled={!selectedDevice}
					>
						Wipe Userdata (Factory Reset)
					</button>
				</section>

			</div>

		</div>
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
