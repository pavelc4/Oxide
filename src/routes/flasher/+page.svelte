<script lang="ts">
	/* eslint-disable svelte/no-navigation-without-resolve */
	import { onMount, tick } from 'svelte';
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

	interface ConsoleHistoryItem {
		id: number;
		type: 'input' | 'stdout' | 'stderr' | 'info';
		content: string;
		timestamp?: string;
		durationMs?: number;
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
	let isDraggingRom = $state(false);
	let isDraggingImg = $state(false);
	let isDraggingZip = $state(false);
	let sideloadZipPath = $state('');
	let flashingManual = $state(false);
	let sideloading = $state(false);

	// File Input Refs for native File Manager Dialog
	let romFileInputRef = $state<HTMLInputElement | undefined>(undefined);
	let imgFileInputRef = $state<HTMLInputElement | undefined>(undefined);
	let zipFileInputRef = $state<HTMLInputElement | undefined>(undefined);

	// ROM Flash Plan State
	let romFolderPath = $state('');
	let scannedPlan = $state<FlashPlan | null>(null);
	let scanningRom = $state(false);
	let executingPlan = $state(false);
	let currentFlashStepIndex = $state(-1);
	let flashProgress = $state(0);

	// Fastboot CLI State (ADB Shell Terminal Style)
	let customCommand = $state('getvar all');
	let cliHistory = $state<ConsoleHistoryItem[]>([]);
	let executingCli = $state(false);
	let logCounter = 0;
	let outputEl = $state<HTMLDivElement | undefined>(undefined);

	// Popular partitions preset
	const partitionPresets = [
		'boot', 'init_boot', 'vendor_boot', 'recovery', 'dtbo', 'system', 'vendor', 'product', 'super', 'vbmeta'
	];

	// Fastboot CLI preset shortcuts
	const fastbootPresets = [
		'getvar all', 'getvar current-slot', 'oem device-info', 'getvar product', 'flashing lock_critical'
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

	function getNowTime(): string {
		const d = new Date();
		return d.toTimeString().split(' ')[0];
	}

	async function scrollToBottom() {
		await tick();
		if (outputEl) {
			outputEl.scrollTop = outputEl.scrollHeight;
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
		if (!confirm('FACTORY RESET WARNING!\n\nThis will completely WIPE all userdata and cache partitions on the device.\nAre you sure you want to proceed?')) return;
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

	async function scanRomFolder(pathOverride?: string) {
		const targetPath = (pathOverride || romFolderPath).trim();
		if (!targetPath) return;
		romFolderPath = targetPath;
		scanningRom = true;
		error = '';
		scannedPlan = null;
		try {
			if (isTauri && invoke) {
				scannedPlan = await safeInvoke<FlashPlan>('scan_flash_folder_cmd', {
					folderPath: targetPath
				});
			} else {
				await new Promise((resolve) => setTimeout(resolve, 600));
				scannedPlan = {
					folder: targetPath,
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

		if (!confirm(`Confirm Flashing ROM?\n\nThis will execute ${selectedSteps.length} partition flash operations on target device [${selectedDevice}].`)) return;

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
			infoMessage = `ROM Flashing Complete! ${selectedSteps.length} partitions updated successfully.`;
			scannedPlan = null;
		} catch (e) {
			error = `Flash plan failed at step ${currentFlashStepIndex + 1}: ${e}`;
		} finally {
			executingPlan = false;
			currentFlashStepIndex = -1;
			flashProgress = 0;
		}
	}

	async function executeCliCommand(cmdToRun?: string) {
		const cmd = (cmdToRun || customCommand).trim();
		if (!selectedDevice || !cmd) return;
		executingCli = true;
		error = '';
		
		const startTime = Date.now();
		cliHistory = [...cliHistory, { id: logCounter++, type: 'input', content: cmd, timestamp: getNowTime() }];
		if (!cmdToRun) customCommand = '';
		await scrollToBottom();

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
			const duration = Date.now() - startTime;
			cliHistory = [...cliHistory, { id: logCounter++, type: 'stdout', content: output, durationMs: duration, timestamp: getNowTime() }];
		} catch (e) {
			cliHistory = [...cliHistory, { id: logCounter++, type: 'stderr', content: String(e), timestamp: getNowTime() }];
		} finally {
			executingCli = false;
			await scrollToBottom();
		}
	}

	function copySingleOutput(text: string) {
		if (navigator.clipboard) {
			navigator.clipboard.writeText(text);
			infoMessage = 'Console output copied to clipboard.';
			setTimeout(() => {
				if (infoMessage.includes('copied')) infoMessage = '';
			}, 2000);
		}
	}

	function clearCliOutput() {
		cliHistory = [];
	}

	function handlePresetClick(p: string) {
		manualPartition = p;
	}

	function handleSampleRomPath() {
		scanRomFolder('/home/user/downloads/Pixel_8_Factory_ROM');
	}

	// File Picker Triggers
	function triggerRomPicker() {
		romFileInputRef?.click();
	}

	function triggerImgPicker() {
		imgFileInputRef?.click();
	}

	function triggerZipPicker() {
		zipFileInputRef?.click();
	}

	function handleRomFileSelect(e: Event) {
		const target = e.target as HTMLInputElement;
		if (target.files && target.files.length > 0) {
			const file = target.files[0];
			const folderPath = (file as unknown as { path?: string }).path || file.name;
			scanRomFolder(folderPath);
		}
	}

	function handleImgFileSelect(e: Event) {
		const target = e.target as HTMLInputElement;
		if (target.files && target.files.length > 0) {
			const file = target.files[0];
			manualImagePath = (file as unknown as { path?: string }).path || file.name;
		}
	}

	function handleZipFileSelect(e: Event) {
		const target = e.target as HTMLInputElement;
		if (target.files && target.files.length > 0) {
			const file = target.files[0];
			sideloadZipPath = (file as unknown as { path?: string }).path || file.name;
		}
	}

	function handleRomDrop(e: DragEvent) {
		e.preventDefault();
		isDraggingRom = false;
		if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
			const file = e.dataTransfer.files[0];
			const folderPath = (file as unknown as { path?: string }).path || file.name;
			scanRomFolder(folderPath);
		}
	}

	function handleImgDrop(e: DragEvent) {
		e.preventDefault();
		isDraggingImg = false;
		if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
			const file = e.dataTransfer.files[0];
			manualImagePath = (file as unknown as { path?: string }).path || file.name;
		}
	}

	function handleZipDrop(e: DragEvent) {
		e.preventDefault();
		isDraggingZip = false;
		if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
			const file = e.dataTransfer.files[0];
			sideloadZipPath = (file as unknown as { path?: string }).path || file.name;
		}
	}
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2 h-screen overflow-hidden">
	<!-- Hidden Native File Inputs -->
	<input bind:this={romFileInputRef} type="file" onchange={handleRomFileSelect} class="hidden" />
	<input bind:this={imgFileInputRef} type="file" accept=".img,.bin" onchange={handleImgFileSelect} class="hidden" />
	<input bind:this={zipFileInputRef} type="file" accept=".zip" onchange={handleZipFileSelect} class="hidden" />

	<div class="flex flex-1 flex-col overflow-hidden rounded-[32px] bg-surface-container-low p-6 lg:p-8 relative gap-5 shadow-sm">

		<!-- Top Header -->
		<header class="flex flex-col md:flex-row md:items-center justify-between gap-4 shrink-0 pb-2">
			<div class="flex items-center gap-4">
				<button
					onclick={() => goto('/')}
					class="flex h-10 w-10 items-center justify-center rounded-xl bg-surface-container hover:bg-surface-container-high text-on-surface-variant transition-all hover:scale-105 active:scale-95 shadow-xs"
					title="Back to dashboard"
				>
					<span class="material-symbols-outlined text-[20px]">arrow_back</span>
				</button>
				<div>
					<div class="flex items-center gap-3">
						<h2 class="text-2xl font-bold tracking-tight text-on-surface">Flasher Studio</h2>
						{#if !isTauri}
							<span class="text-[10px] bg-warning/15 text-warning px-2.5 py-0.5 rounded-full font-bold uppercase tracking-wider">MOCK MODE</span>
						{/if}
					</div>
					<p class="text-xs text-on-surface-variant/80 font-medium mt-0.5">Fastboot partitioning, ROM flasher planner & ADB sideload manager</p>
				</div>
			</div>

			<!-- Active Device Status & Refresh -->
			<div class="flex items-center gap-3">
				<div class="flex items-center gap-2 bg-surface-container px-4 py-2 rounded-full shadow-xs">
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
					class="flex h-9 w-9 items-center justify-center rounded-full bg-surface-container hover:bg-surface-container-high text-on-surface-variant transition-all hover:scale-105 active:scale-95 shadow-xs"
					title="Rescan Devices"
				>
					<span class="material-symbols-outlined text-[18px] {loading ? 'animate-spin' : ''}">refresh</span>
				</button>
			</div>
		</header>

		<!-- Alert Banners (No Emojis!) -->
		{#if error}
			<div class="bg-error/15 text-error p-3.5 rounded-2xl font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[20px]">error</span>
				<div class="flex-1 break-words font-semibold">{error}</div>
				<button onclick={() => (error = '')} class="hover:opacity-80 text-[10px] font-bold uppercase tracking-wider bg-error/20 px-2.5 py-1 rounded-lg">Dismiss</button>
			</div>
		{/if}

		{#if infoMessage}
			<div class="bg-primary/10 text-primary p-3.5 rounded-2xl font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[20px]">check_circle</span>
				<div class="flex-1 break-words font-semibold">{infoMessage}</div>
				<button onclick={() => (infoMessage = '')} class="hover:opacity-80 text-[10px] font-bold uppercase tracking-wider bg-primary/20 px-2.5 py-1 rounded-lg">Dismiss</button>
			</div>
		{/if}

		<!-- Main Studio Grid -->
		<div class="flex-1 grid grid-cols-1 lg:grid-cols-12 gap-6 overflow-hidden min-h-0">
			
			<!-- Left Column: Navigation Tabs & Workspace (Col 8) -->
			<div class="lg:col-span-8 flex flex-col gap-4 overflow-hidden min-h-0">
				
				<!-- Mode Navigation Tabs -->
				<nav class="flex gap-2 p-1.5 bg-surface-container rounded-2xl shrink-0 overflow-x-auto shadow-xs">
					<button
						onclick={() => (activeTab = 'rom_planner')}
						class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all whitespace-nowrap {activeTab === 'rom_planner' ? 'bg-primary text-on-primary shadow-sm' : 'text-on-surface-variant hover:text-on-surface hover:bg-surface-container-high'}"
					>
						<span class="material-symbols-outlined text-[18px]">folder_zip</span>
						ROM Flash Planner
					</button>

					<button
						onclick={() => (activeTab = 'manual_flasher')}
						class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all whitespace-nowrap {activeTab === 'manual_flasher' ? 'bg-primary text-on-primary shadow-sm' : 'text-on-surface-variant hover:text-on-surface hover:bg-surface-container-high'}"
					>
						<span class="material-symbols-outlined text-[18px]">flash_on</span>
						Single Partition
					</button>

					<button
						onclick={() => (activeTab = 'sideload')}
						class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all whitespace-nowrap {activeTab === 'sideload' ? 'bg-primary text-on-primary shadow-sm' : 'text-on-surface-variant hover:text-on-surface hover:bg-surface-container-high'}"
					>
						<span class="material-symbols-outlined text-[18px]">install_mobile</span>
						ADB Sideload
					</button>

					<button
						onclick={() => (activeTab = 'fastboot_console')}
						class="flex items-center gap-2 px-4 py-2.5 rounded-xl text-xs font-bold transition-all whitespace-nowrap {activeTab === 'fastboot_console' ? 'bg-primary text-on-primary shadow-sm' : 'text-on-surface-variant hover:text-on-surface hover:bg-surface-container-high'}"
					>
						<span class="material-symbols-outlined text-[18px]">terminal</span>
						Fastboot Terminal
					</button>
				</nav>

				<!-- Tab 1: ROM Flash Planner -->
				{#if activeTab === 'rom_planner'}
					<div class="flex-1 rounded-[32px] bg-surface-container p-6 flex flex-col gap-4 overflow-hidden min-h-0 shadow-sm">
						<div>
							<h3 class="text-base font-bold text-on-surface flex items-center gap-2">
								<span class="material-symbols-outlined text-primary">auto_mode</span>
								Batch ROM Flash Planner
							</h3>
							<p class="text-xs text-on-surface-variant mt-0.5">Click box to open File Manager or drop a factory ROM payload directory.</p>
						</div>

						<!-- Clean Interactive Click & Drag Dropzone -->
						<div
							role="button"
							tabindex="0"
							onclick={triggerRomPicker}
							onkeydown={(e) => { if (e.key === 'Enter') triggerRomPicker(); }}
							ondragover={(e) => { e.preventDefault(); isDraggingRom = true; }}
							ondragleave={() => (isDraggingRom = false)}
							ondrop={handleRomDrop}
							class="flex flex-col items-center justify-center p-8 rounded-[28px] transition-all cursor-pointer text-center gap-3 shrink-0 {isDraggingRom ? 'bg-primary/20 scale-[1.01]' : 'bg-surface-container-high hover:bg-surface-container-highest'}"
						>
							<div class="flex h-14 w-14 items-center justify-center rounded-2xl bg-primary/15 text-primary shadow-inner">
								<span class="material-symbols-outlined text-[30px]">{isDraggingRom ? 'downloading' : romFolderPath ? 'folder' : 'folder_open'}</span>
							</div>

							{#if romFolderPath}
								<div class="flex flex-col items-center gap-1.5 max-w-lg">
									<span class="text-xs font-bold text-on-surface">Selected Factory ROM Folder:</span>
									<span class="text-xs font-mono font-bold text-primary bg-primary/15 px-4 py-1.5 rounded-xl break-all truncate max-w-full">{romFolderPath}</span>
									<div class="flex items-center gap-2 mt-2">
										<button
											onclick={(e) => { e.stopPropagation(); scanRomFolder(); }}
											class="rounded-xl bg-primary text-on-primary hover:brightness-110 px-5 py-2 text-xs font-bold shadow-xs transition-all"
											disabled={scanningRom || executingPlan}
										>
											{scanningRom ? 'Scanning Directory...' : 'Scan ROM Directory'}
										</button>
										<button
											onclick={(e) => { e.stopPropagation(); romFolderPath = ''; scannedPlan = null; }}
											class="rounded-xl bg-surface-container hover:bg-surface-container-highest text-on-surface-variant px-3 py-2 text-xs font-bold transition-all"
										>
											Change Folder
										</button>
									</div>
								</div>
							{:else}
								<div class="flex flex-col items-center gap-1">
									<span class="text-sm font-bold text-on-surface">Click to Browse File Manager or Drag & Drop ROM Folder</span>
									<span class="text-xs text-on-surface-variant/80">Select factory ROM payload folder to generate partition execution plan</span>
									<button onclick={(e) => { e.stopPropagation(); handleSampleRomPath(); }} class="text-[11px] text-primary hover:underline font-bold mt-2">
										Click here to load sample ROM plan preview
									</button>
								</div>
							{/if}
						</div>

						<!-- Step List / Partitions Table -->
						{#if scannedPlan}
							<div class="flex-1 flex flex-col gap-3 min-h-0 overflow-hidden">
								<div class="flex items-center justify-between shrink-0">
									<span class="text-xs font-bold text-on-surface">Target Partition Queue ({scannedPlan.steps.length} files found)</span>
									<span class="text-[11px] text-on-surface-variant font-mono">Location: {scannedPlan.folder}</span>
								</div>

								<div class="flex-1 bg-surface-container-low rounded-2xl overflow-y-auto p-2 flex flex-col gap-1.5 shadow-inner">
									{#each scannedPlan.steps as step, idx (idx)}
										<div
											role="button"
											tabindex="0"
											onclick={() => { if (!executingPlan) step.selected = !step.selected; }}
											onkeydown={(e) => { if (e.key === 'Enter' && !executingPlan) step.selected = !step.selected; }}
											class="flex items-center justify-between p-3.5 rounded-2xl hover:bg-surface-container-high transition-all cursor-pointer {step.selected !== false ? 'bg-surface-container-high/60' : 'opacity-50'} {currentFlashStepIndex === idx ? 'bg-primary-container/30 font-bold' : ''}"
										>
											<div class="flex items-center gap-3.5">
												<div class="flex h-6 w-6 items-center justify-center rounded-lg {step.selected !== false ? 'bg-primary text-on-primary' : 'bg-surface-container-highest text-on-surface-variant/50'} shadow-xs">
													<span class="material-symbols-outlined text-[15px]">{step.selected !== false ? 'check' : ''}</span>
												</div>
												<div>
													<span class="text-xs font-bold font-mono text-on-surface block">{step.partition}</span>
													<span class="text-[10px] text-on-surface-variant/70 font-mono block">{step.image_file}</span>
												</div>
											</div>
											{#if step.size}
												<span class="text-[11px] font-mono text-on-surface-variant px-3 py-1 bg-surface-container rounded-xl font-bold shadow-xs">{step.size}</span>
											{/if}
										</div>
									{/each}
								</div>

								<!-- Progress Bar -->
								{#if executingPlan}
									<div class="flex flex-col gap-1.5 shrink-0 bg-primary/10 p-3.5 rounded-2xl">
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
									class="rounded-2xl bg-primary text-on-primary hover:brightness-110 px-6 py-3 text-xs font-bold transition-all shadow-md disabled:opacity-50 shrink-0 w-full"
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
					<div class="flex-1 rounded-[32px] bg-surface-container p-6 flex flex-col justify-between gap-5 overflow-y-auto shadow-sm">
						<div class="flex flex-col gap-5">
							<div>
								<h3 class="text-base font-bold text-on-surface flex items-center gap-2">
									<span class="material-symbols-outlined text-primary">flash_on</span>
									Single Partition Image Flasher
								</h3>
								<p class="text-xs text-on-surface-variant mt-0.5">Flash an individual partition image (.img / .bin) directly to the connected target device.</p>
							</div>

							<!-- Partition Quick Presets Chips -->
							<div class="flex flex-col gap-2">
								<span class="text-[11px] font-bold text-on-surface-variant uppercase tracking-wider">Target Partition Presets</span>
								<div class="flex flex-wrap gap-2">
									{#each partitionPresets as p}
										<button
											onclick={() => handlePresetClick(p)}
											class="px-3.5 py-1.5 rounded-xl text-xs font-mono font-bold transition-all {manualPartition === p ? 'bg-primary text-on-primary shadow-xs' : 'bg-surface-container-high text-on-surface-variant hover:bg-surface-container-highest hover:text-on-surface'}"
										>
											{p}
										</button>
									{/each}
								</div>
							</div>

							<!-- Clean Clickable & Drag Dropzone -->
							<div
								role="button"
								tabindex="0"
								onclick={triggerImgPicker}
								onkeydown={(e) => { if (e.key === 'Enter') triggerImgPicker(); }}
								ondragover={(e) => { e.preventDefault(); isDraggingImg = true; }}
								ondragleave={() => (isDraggingImg = false)}
								ondrop={handleImgDrop}
								class="flex flex-col items-center justify-center p-8 rounded-[28px] transition-all cursor-pointer text-center gap-3 {isDraggingImg ? 'bg-primary/20 scale-[1.01]' : 'bg-surface-container-high hover:bg-surface-container-highest'}"
							>
								<div class="flex h-14 w-14 items-center justify-center rounded-2xl bg-primary/15 text-primary shadow-inner">
									<span class="material-symbols-outlined text-[30px]">{isDraggingImg ? 'downloading' : manualImagePath ? 'image' : 'cloud_upload'}</span>
								</div>

								{#if manualImagePath}
									<div class="flex flex-col items-center gap-1.5 max-w-lg">
										<span class="text-xs font-bold text-on-surface">Selected Partition Image:</span>
										<span class="text-xs font-mono font-bold text-primary bg-primary/15 px-4 py-1.5 rounded-xl break-all truncate max-w-full">{manualImagePath}</span>
										<span class="text-[10px] text-on-surface-variant/70 mt-1">Click to choose different image file</span>
									</div>
								{:else}
									<div class="flex flex-col items-center gap-1">
										<span class="text-sm font-bold text-on-surface">Click to Browse File Manager or Drag & Drop .IMG File</span>
										<span class="text-xs text-on-surface-variant/80">Select image file (.img / .bin) to flash into partition [{manualPartition}]</span>
									</div>
								{/if}
							</div>
						</div>

						<button
							onclick={manualFlashPartition}
							class="rounded-2xl bg-primary text-on-primary hover:brightness-110 px-6 py-3 text-xs font-bold transition-all shadow-md disabled:opacity-50 w-full"
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
					<div class="flex-1 rounded-[32px] bg-surface-container p-6 flex flex-col justify-between gap-5 shadow-sm">
						<div class="flex flex-col gap-5">
							<div>
								<h3 class="text-base font-bold text-on-surface flex items-center gap-2">
									<span class="material-symbols-outlined text-primary">install_mobile</span>
									ADB Sideload Package Manager
								</h3>
								<p class="text-xs text-on-surface-variant mt-0.5">Sideload OTA updates, system zip patches, or custom recovery packages when device is in Sideload Mode.</p>
							</div>

							<div class="bg-primary/10 rounded-2xl p-4 flex items-start gap-3">
								<span class="material-symbols-outlined text-primary text-[22px] shrink-0 mt-0.5">info</span>
								<div class="text-xs text-on-surface-variant">
									<p class="font-bold text-on-surface">Entering Sideload Mode:</p>
									<p class="mt-1 leading-relaxed">Make sure your phone is in Recovery Mode and select <code class="bg-surface-container-high px-2 py-0.5 rounded text-primary font-mono font-bold">Apply update from ADB</code>.</p>
								</div>
							</div>

							<!-- Clean Clickable & Drag Dropzone -->
							<div
								role="button"
								tabindex="0"
								onclick={triggerZipPicker}
								onkeydown={(e) => { if (e.key === 'Enter') triggerZipPicker(); }}
								ondragover={(e) => { e.preventDefault(); isDraggingZip = true; }}
								ondragleave={() => (isDraggingZip = false)}
								ondrop={handleZipDrop}
								class="flex flex-col items-center justify-center p-8 rounded-[28px] transition-all cursor-pointer text-center gap-3 {isDraggingZip ? 'bg-primary/20 scale-[1.01]' : 'bg-surface-container-high hover:bg-surface-container-highest'}"
							>
								<div class="flex h-14 w-14 items-center justify-center rounded-2xl bg-primary/15 text-primary shadow-inner">
									<span class="material-symbols-outlined text-[30px]">{isDraggingZip ? 'downloading' : sideloadZipPath ? 'archive' : 'cloud_upload'}</span>
								</div>

								{#if sideloadZipPath}
									<div class="flex flex-col items-center gap-1.5 max-w-lg">
										<span class="text-xs font-bold text-on-surface">Selected OTA Package File:</span>
										<span class="text-xs font-mono font-bold text-primary bg-primary/15 px-4 py-1.5 rounded-xl break-all truncate max-w-full">{sideloadZipPath}</span>
										<span class="text-[10px] text-on-surface-variant/70 mt-1">Click to choose different ZIP package</span>
									</div>
								{:else}
									<div class="flex flex-col items-center gap-1">
										<span class="text-sm font-bold text-on-surface">Click to Browse File Manager or Drag & Drop .ZIP Package</span>
										<span class="text-xs text-on-surface-variant/80">Select OTA update zip package to sideload via ADB</span>
									</div>
								{/if}
							</div>
						</div>

						<button
							onclick={runSideload}
							class="rounded-2xl bg-primary text-on-primary hover:brightness-110 px-6 py-3 text-xs font-bold transition-all shadow-md disabled:opacity-50 w-full"
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

				<!-- Tab 4: Fastboot Terminal Console (Exact ADB Shell Studio Output Style!) -->
				{#if activeTab === 'fastboot_console'}
					<div class="flex-1 rounded-[32px] bg-surface-container p-6 flex flex-col justify-between gap-4 overflow-hidden shadow-sm min-h-0">
						
						<!-- Terminal Toolbar Header -->
						<div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 shrink-0">
							<div>
								<div class="flex items-center gap-3">
									<h3 class="text-base font-bold text-on-surface flex items-center gap-2">
										<span class="material-symbols-outlined text-primary">terminal</span>
										Fastboot Interactive Console
									</h3>
									<span class="flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-emerald-500/15 text-emerald-400 text-[10px] font-bold">
										<span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
										Ready
									</span>
								</div>
								<p class="text-xs text-on-surface-variant mt-0.5">Execute direct fastboot protocol sub-commands against connected target.</p>
							</div>

							<!-- Action Buttons -->
							<div class="flex items-center gap-2">
								<button
									onclick={() => {
										if (navigator.clipboard) {
											const text = cliHistory.map(h => h.content).join('\n');
											navigator.clipboard.writeText(text);
											infoMessage = 'Entire console transcript copied.';
											setTimeout(() => { if (infoMessage.includes('transcript')) infoMessage = ''; }, 2000);
										}
									}}
									class="flex items-center gap-1.5 rounded-full bg-surface-container-high hover:bg-surface-container-highest px-3.5 py-1.5 text-xs font-bold text-on-surface transition-all shadow-xs"
									title="Copy Entire Terminal Transcript"
								>
									<span class="material-symbols-outlined text-[15px]">content_copy</span>
									Copy All
								</button>

								<button
									onclick={clearCliOutput}
									class="flex items-center gap-1.5 rounded-full bg-surface-container-high hover:bg-surface-container-highest px-3.5 py-1.5 text-xs font-bold text-on-surface hover:text-error transition-all shadow-xs"
									title="Clear Terminal Console Output"
								>
									<span class="material-symbols-outlined text-[15px]">delete_sweep</span>
									Clear
								</button>
							</div>
						</div>

						<!-- Preset Command Chips Bar -->
						<div class="flex items-center gap-2 overflow-x-auto pb-1 shrink-0">
							<span class="text-[11px] font-bold text-on-surface-variant uppercase tracking-wider shrink-0 mr-1">Presets:</span>
							{#each fastbootPresets as cmdPreset (cmdPreset)}
								<button
									onclick={() => executeCliCommand(cmdPreset)}
									class="px-3.5 py-1.5 rounded-full text-xs font-mono font-bold bg-surface-container-high hover:bg-primary hover:text-on-primary text-on-surface transition-all shadow-xs whitespace-nowrap"
								>
									{cmdPreset}
								</button>
							{/each}
						</div>

						<!-- ADB Shell Style Output Terminal Window -->
						<div
							bind:this={outputEl}
							class="flex-1 bg-surface-container-high/40 rounded-[28px] p-6 font-mono text-xs overflow-y-auto flex flex-col gap-3 shadow-inner leading-relaxed select-text min-h-0"
						>
							{#if cliHistory.length === 0}
								<div class="flex flex-col items-center justify-center h-full text-on-surface-variant/60 py-12">
									<span class="material-symbols-outlined text-[44px] opacity-40 mb-2">terminal</span>
									<p class="text-xs font-semibold">Fastboot console initialized. Click presets above or type commands below.</p>
								</div>
							{:else}
								{#each cliHistory as entry (entry.id)}
									{#if entry.type === 'input'}
										<div class="flex items-center justify-between mt-2 first:mt-0 font-mono">
											<div class="flex items-center gap-2">
												<span class="text-primary font-bold select-none">$ fastboot -s {selectedDevice || 'target'}</span>
												<span class="text-on-surface font-bold">{entry.content}</span>
											</div>
											{#if entry.timestamp}
												<span class="text-[10px] text-on-surface-variant/40 font-mono select-none">{entry.timestamp}</span>
											{/if}
										</div>
									{:else if entry.type === 'stdout'}
										<div class="pl-4">
											<div class="bg-surface-container-high/70 p-4 rounded-2xl flex flex-col gap-2 relative shadow-xs">
												<div class="flex items-center justify-between text-[10px] text-on-surface-variant/70 font-mono pb-1">
													<span>Fastboot Protocol Response {#if entry.durationMs !== undefined}({entry.durationMs}ms){/if}</span>
													<button
														onclick={() => copySingleOutput(entry.content)}
														class="flex items-center gap-1.5 text-[10px] font-bold bg-surface-container hover:bg-primary hover:text-on-primary text-on-surface px-2.5 py-1 rounded-lg transition-all shadow-xs"
														title="Copy command output"
													>
														<span class="material-symbols-outlined text-[13px]">content_copy</span>
														Copy Output
													</button>
												</div>
												<pre class="text-on-surface/90 font-mono text-xs whitespace-pre-wrap break-all leading-relaxed m-0">{entry.content}</pre>
											</div>
										</div>
									{:else if entry.type === 'stderr'}
										<div class="pl-4">
											<div class="bg-error/10 p-4 rounded-2xl flex flex-col gap-2 font-semibold shadow-xs">
												<div class="flex items-center justify-between text-[10px] text-error font-mono pb-1">
													<span>Fastboot Error Output</span>
													<button
														onclick={() => copySingleOutput(entry.content)}
														class="flex items-center gap-1.5 text-[10px] font-bold bg-error/20 hover:bg-error text-error hover:text-on-error px-2.5 py-1 rounded-lg transition-all shadow-xs"
														title="Copy error text"
													>
														<span class="material-symbols-outlined text-[13px]">content_copy</span>
														Copy Error
													</button>
												</div>
												<pre class="text-error font-mono text-xs whitespace-pre-wrap break-all leading-relaxed m-0">{entry.content}</pre>
											</div>
										</div>
									{/if}
								{/each}
							{/if}
						</div>

						<!-- Command Prompt Input Form -->
						<form onsubmit={(e) => { e.preventDefault(); executeCliCommand(); }} class="flex items-center gap-3 bg-surface-container-high rounded-full px-5 py-2.5 shadow-xs shrink-0">
							<span class="text-primary font-bold font-mono text-xs select-none">$ fastboot</span>
							<input
								type="text"
								bind:value={customCommand}
								placeholder="getvar all"
								class="flex-1 bg-transparent font-mono text-xs text-on-surface placeholder:text-on-surface-variant/50 focus:outline-none disabled:opacity-40"
								disabled={executingCli}
							/>
							<button
								type="submit"
								class="rounded-full bg-primary text-on-primary hover:brightness-110 px-5 py-1.5 text-xs font-bold transition-all disabled:opacity-50 shadow-xs shrink-0"
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
				<section class="rounded-[32px] bg-surface-container p-6 flex flex-col gap-4 shadow-sm">
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
							class="rounded-2xl py-2.5 px-3 text-xs font-bold transition-all {activeSlot === 'a' ? 'bg-primary text-on-primary shadow-sm' : 'bg-surface-container-high text-on-surface hover:bg-surface-container-highest'}"
							disabled={loadingSlot || !selectedDevice}
						>
							Switch Slot A
						</button>
						<button
							onclick={() => setActiveSlot('b')}
							class="rounded-2xl py-2.5 px-3 text-xs font-bold transition-all {activeSlot === 'b' ? 'bg-primary text-on-primary shadow-sm' : 'bg-surface-container-high text-on-surface hover:bg-surface-container-highest'}"
							disabled={loadingSlot || !selectedDevice}
						>
							Switch Slot B
						</button>
					</div>

					<button
						onclick={continueBoot}
						class="rounded-2xl bg-surface-container-high text-on-surface hover:bg-primary hover:text-on-primary transition-all text-xs font-bold py-2.5 w-full disabled:opacity-40 shadow-xs"
						disabled={!selectedDevice}
					>
						Continue Normal Booting
					</button>
				</section>

				<!-- Device State & Quick Reboot Actions -->
				<section class="rounded-[32px] bg-surface-container p-6 flex flex-col gap-3 shadow-sm">
					<h3 class="text-xs font-bold text-on-surface uppercase tracking-wider flex items-center gap-2 mb-1">
						<span class="material-symbols-outlined text-primary text-[18px]">power_settings_new</span>
						Reboot Controls
					</h3>

					<div class="grid grid-cols-2 gap-2">
						<button
							onclick={() => rebootDevice('normal')}
							class="rounded-2xl bg-surface-container-high hover:bg-surface-container-highest text-on-surface py-2.5 text-xs font-bold transition-all disabled:opacity-40 shadow-xs"
							disabled={!selectedDevice}
						>
							System
						</button>
						<button
							onclick={() => rebootDevice('bootloader')}
							class="rounded-2xl bg-surface-container-high hover:bg-surface-container-highest text-on-surface py-2.5 text-xs font-bold transition-all disabled:opacity-40 shadow-xs"
							disabled={!selectedDevice}
						>
							Bootloader
						</button>
					</div>

					<button
						onclick={() => rebootDevice('recovery')}
						class="rounded-2xl bg-surface-container-high hover:bg-surface-container-highest text-on-surface py-2.5 text-xs font-bold transition-all disabled:opacity-40 w-full shadow-xs"
						disabled={!selectedDevice}
					>
						Reboot Recovery (Sideload)
					</button>
				</section>

				<!-- Safety & Reset Action -->
				<section class="rounded-[32px] bg-error/10 p-6 flex flex-col gap-3 shadow-sm">
					<h3 class="text-xs font-bold text-error uppercase tracking-wider flex items-center gap-2">
						<span class="material-symbols-outlined text-error text-[18px]">warning</span>
						Dangerous Wipe Actions
					</h3>
					<p class="text-[11px] text-on-surface-variant leading-relaxed">Perform factory reset by clearing user data & cache partitions.</p>
					<button
						onclick={wipeUserdata}
						class="rounded-2xl bg-error/15 text-error hover:bg-error/25 transition-all text-xs font-bold py-2.5 w-full disabled:opacity-40 shadow-xs"
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
