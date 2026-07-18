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
		connection: string;
	}

	interface FlashStep {
		partition: string;
		image_file: string;
	}

	interface FlashPlan {
		folder: string;
		steps: FlashStep[];
	}

	// State
	let devices = $state<Device[]>([]);
	let selectedDevice = $state<string>('');
	let loading = $state(true);
	let error = $state('');
	let infoMessage = $state('');

	// Slot
	let activeSlot = $state('Unknown');
	let loadingSlot = $state(false);

	// Manual Flash & Sideload
	let manualPartition = $state('boot');
	let manualImagePath = $state('');
	let sideloadZipPath = $state('');
	let flashingManual = $state(false);
	let sideloading = $state(false);

	// ROM Scan Plan
	let romFolderPath = $state('');
	let scannedPlan = $state<FlashPlan | null>(null);
	let scanningRom = $state(false);
	let executingPlan = $state(false);
	let currentFlashStepIndex = $state(-1);

	// CLI Fastboot
	let customCommand = $state('getvar all');
	let cliLogs = $state<string[]>([]);
	let executingCli = $state(false);

	onMount(async () => {
		try {
			const tauriApi = await import('@tauri-apps/api/core');
			invoke = tauriApi.invoke;
			isTauri = true;
		} catch {
			console.warn('Tauri not available, using mock mode');
			isTauri = false;
		}

		await loadDevices();
	});

	async function loadDevices() {
		loading = true;
		error = '';
		try {
			if (isTauri && invoke) {
				const rustDevices = (await invoke('get_devices')) as Array<{ serial: string; model?: string }>;
				if (rustDevices) {
					devices = rustDevices.map((d: { serial: string; model?: string }) => ({
						id: d.serial,
						name: d.model || d.serial,
						status: 'Online',
						connection: 'USB'
					}));
				} else {
					devices = [];
				}
			} else {
				devices = [
					{ id: 'fastboot-device-789', name: 'Mock Pixel 8 (Bootloader)', status: 'Online', connection: 'USB' },
					{ id: 'mock-device-123', name: 'Mock Galaxy S24 Ultra', status: 'Online', connection: 'USB' }
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
				const slot = (await invoke('fastboot_get_active_slot', { serial: selectedDevice })) as string;
				activeSlot = slot || 'Unknown';
			} else {
				activeSlot = 'a';
			}
		} catch {
			// Fail silently or set to Unknown if device is not in fastboot mode
			activeSlot = 'Unknown / ADB Mode';
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
				await invoke('fastboot_set_active_slot', { serial: selectedDevice, slot });
			}
			activeSlot = slot;
			infoMessage = `Active slot switched to Slot ${slot.toUpperCase()} successfully.`;
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
				await invoke('fastboot_continue_boot', { serial: selectedDevice });
			}
			infoMessage = 'Continue boot command sent. Device should boot up now.';
		} catch (e) {
			error = `Failed to boot: ${e}`;
		}
	}

	async function wipeUserdata() {
		if (!selectedDevice) return;
		if (!confirm('DANGER! This will perform a factory reset and WIPE all user data on the device. Are you sure you want to proceed?')) return;
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				await invoke('fastboot_wipe_userdata', { serial: selectedDevice });
			}
			infoMessage = 'User data partition wiped successfully (Factory Reset completed).';
		} catch (e) {
			error = `Failed to wipe data: ${e}`;
		}
	}

	async function manualFlashPartition() {
		if (!selectedDevice || !manualImagePath.trim()) return;
		flashingManual = true;
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				await invoke('fastboot_flash_partition', {
					serial: selectedDevice,
					partition: manualPartition.trim(),
					filePath: manualImagePath.trim()
				});
			} else {
				await new Promise((resolve) => setTimeout(resolve, 2000));
			}
			infoMessage = `Successfully flashed ${manualPartition} with image: ${manualImagePath}`;
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
				await invoke('adb_sideload_cmd', {
					serial: selectedDevice,
					zipPath: sideloadZipPath.trim()
				});
			} else {
				await new Promise((resolve) => setTimeout(resolve, 3000));
			}
			infoMessage = `ZIP Sideload completed successfully: ${sideloadZipPath}`;
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
				scannedPlan = (await invoke('scan_flash_folder_cmd', {
					folderPath: romFolderPath.trim()
				})) as FlashPlan;
			} else {
				await new Promise((resolve) => setTimeout(resolve, 800));
				scannedPlan = {
					folder: romFolderPath.trim(),
					steps: [
						{ partition: 'boot', image_file: 'boot.img' },
						{ partition: 'init_boot', image_file: 'init_boot.img' },
						{ partition: 'vendor_boot', image_file: 'vendor_boot.img' },
						{ partition: 'system', image_file: 'system.img' },
						{ partition: 'vendor', image_file: 'vendor.img' }
					]
				};
			}
		} catch (e) {
			error = `Failed to scan ROM folder: ${e}`;
		} finally {
			scanningRom = false;
		}
	}

	async function executeFlashPlan() {
		if (!scannedPlan || scannedPlan.steps.length === 0 || !selectedDevice) return;
		if (!confirm(`Confirm ROM Flashing Plan? This will execute ${scannedPlan.steps.length} sequential flash operations.`)) return;

		executingPlan = true;
		error = '';
		infoMessage = '';
		currentFlashStepIndex = 0;

		try {
			for (let i = 0; i < scannedPlan.steps.length; i++) {
				currentFlashStepIndex = i;
				const step = scannedPlan.steps[i];
				
				// Construct absolute path to image file
				const imgPath = `${scannedPlan.folder}/${step.image_file}`;
				
				if (isTauri && invoke) {
					await invoke('fastboot_flash_partition', {
						serial: selectedDevice,
						partition: step.partition,
						filePath: imgPath
					});
				} else {
					await new Promise((resolve) => setTimeout(resolve, 1500)); // simulates each flash
				}
			}
			infoMessage = `ROM flashed successfully! Total ${scannedPlan.steps.length} partitions updated.`;
			scannedPlan = null;
		} catch (e) {
			error = `Flash plan failed at step ${currentFlashStepIndex + 1} (${scannedPlan?.steps[currentFlashStepIndex]?.partition || 'unknown'}): ${e}`;
		} finally {
			executingPlan = false;
			currentFlashStepIndex = -1;
		}
	}

	async function executeCliCommand() {
		if (!selectedDevice || !customCommand.trim()) return;
		executingCli = true;
		error = '';
		const cmd = customCommand.trim();
		cliLogs = [...cliLogs, `> fastboot -s ${selectedDevice} ${cmd}`];
		
		try {
			let output = '';
			if (isTauri && invoke) {
				output = (await invoke('fastboot_custom_command', {
					serial: selectedDevice,
					command: cmd
				})) as string;
			} else {
				await new Promise((resolve) => setTimeout(resolve, 500));
				if (cmd.includes('getvar')) {
					output = 'active-slot: a\nsecure: yes\nproduct: mock_device\nfinished. total time: 0.05s';
				} else {
					output = 'finished. total time: 0.12s';
				}
			}
			cliLogs = [...cliLogs, output];
			customCommand = '';
		} catch (e) {
			cliLogs = [...cliLogs, `Error: ${e}`];
		} finally {
			executingCli = false;
		}
	}
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2 h-screen overflow-hidden">
	<div
		class="flex flex-1 flex-col overflow-hidden rounded-[32px] bg-surface-container-low p-6 lg:p-10 relative"
	>
		<!-- Page Header -->
		<header class="mb-6 flex justify-between items-center shrink-0">
			<div class="flex items-center gap-4">
				<button
					onclick={() => goto('/')}
					class="flex h-10 w-10 items-center justify-center rounded-full text-on-surface-variant transition-colors hover:bg-surface-variant hover:text-on-surface"
					title="Back to dashboard"
				>
					<span class="material-symbols-outlined text-[22px]">arrow_back</span>
				</button>
				<h2 class="text-2xl font-bold tracking-tight text-on-surface flex items-center gap-4">
					Flasher & Fastboot Utility
					{#if !isTauri}
						<span class="text-xs bg-error text-on-error px-3 py-1 rounded-full font-medium tracking-normal border border-error/30">MOCK MODE</span>
					{/if}
				</h2>
			</div>

			<!-- Device Selector -->
			<div class="flex items-center gap-3">
				<span class="text-xs font-semibold text-on-surface-variant">Device:</span>
				<select
					value={selectedDevice}
					onchange={(e) => selectDeviceChanged(e.currentTarget.value)}
					class="bg-surface-container-high rounded-full border border-outline-variant px-4 py-1.5 text-sm text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all font-medium"
				>
					{#each devices as dev (dev.id)}
						<option value={dev.id}>{dev.name} ({dev.id.slice(0, 8)})</option>
					{/each}
				</select>
				<button
					onclick={loadDevices}
					class="flex h-8 w-8 items-center justify-center rounded-full bg-surface-container-highest text-on-surface-variant hover:text-on-surface transition-all hover:scale-105 active:scale-95"
					title="Refresh Device List"
				>
					<span class="material-symbols-outlined text-[18px] {loading ? 'animate-spin' : ''}">refresh</span>
				</button>
			</div>
		</header>

		<!-- Alert Messages -->
		{#if error}
			<div
				class="bg-error/15 text-error border border-error/30 p-4 rounded-2xl mb-4 font-medium flex items-center gap-3 text-sm shrink-0 animate-fade-in"
			>
				<span class="material-symbols-outlined text-[20px]">error</span>
				<div class="flex-1 break-words">{error}</div>
				<button onclick={() => (error = '')} class="hover:opacity-85 text-xs font-semibold uppercase">Dismiss</button>
			</div>
		{/if}

		{#if infoMessage}
			<div
				class="bg-primary/10 text-primary p-4 rounded-2xl mb-4 font-medium flex items-center gap-3 text-sm shrink-0 animate-fade-in"
			>
				<span class="material-symbols-outlined text-[20px]">check_circle</span>
				<div class="flex-1 break-words">{infoMessage}</div>
				<button onclick={() => (infoMessage = '')} class="hover:opacity-85 text-xs font-semibold uppercase">Dismiss</button>
			</div>
		{/if}

		<!-- Grid Workspace -->
		<div class="flex-1 grid grid-cols-1 xl:grid-cols-3 gap-6 overflow-y-auto pr-1">
			<!-- Col 1: Slots, Wipes, Sideloads -->
			<div class="flex flex-col gap-6">
				<!-- Slot Controller -->
				<section class="rounded-[24px] bg-surface-container p-5 flex flex-col gap-4">
					<header class="flex items-center gap-2 pb-2.5">
						<span class="material-symbols-outlined text-primary text-[22px]">swap_horiz</span>
						<h3 class="text-sm font-bold text-on-surface">Slot Controller (A/B)</h3>
					</header>

					<div class="flex items-center justify-between">
						<span class="text-xs text-on-surface-variant font-semibold">Active boot slot:</span>
						<span class="text-xs font-bold bg-primary/10 text-primary px-3 py-1 rounded-full uppercase tracking-wider">
							{#if loadingSlot}
								<span class="animate-pulse">Checking...</span>
							{:else}
								Slot {activeSlot.toUpperCase()}
							{/if}
						</span>
					</div>

					<div class="flex gap-2">
						<button
							onclick={() => setActiveSlot('a')}
							class="flex-1 rounded-xl bg-surface-container-high hover:bg-surface-container-highest transition-all py-2 text-xs font-bold text-on-surface disabled:opacity-40"
							disabled={loadingSlot || !selectedDevice}
						>
							Set Slot A
						</button>
						<button
							onclick={() => setActiveSlot('b')}
							class="flex-1 rounded-xl bg-surface-container-high hover:bg-surface-container-highest transition-all py-2 text-xs font-bold text-on-surface disabled:opacity-40"
							disabled={loadingSlot || !selectedDevice}
						>
							Set Slot B
						</button>
					</div>

					<button
						onclick={continueBoot}
						class="rounded-xl bg-primary text-on-primary hover:brightness-105 transition-all text-xs font-bold py-2 disabled:opacity-40 w-full"
						disabled={!selectedDevice}
					>
						Continue Booting System
					</button>
				</section>

				<!-- Dangerous Wipes & Reboot Actions -->
				<section class="rounded-[24px] bg-surface-container p-5 flex flex-col gap-3">
					<header class="flex items-center gap-2 pb-2.5">
						<span class="material-symbols-outlined text-primary text-[22px]">delete_sweep</span>
						<h3 class="text-sm font-bold text-on-surface">Factory Reset & Reboot</h3>
					</header>

					<button
						onclick={wipeUserdata}
						class="rounded-xl bg-error/10 text-error hover:bg-error/15 transition-all text-xs font-bold py-2.5 w-full disabled:opacity-40"
						disabled={!selectedDevice}
					>
						Wipe Userdata (Factory Reset)
					</button>

					<div class="h-px bg-outline-variant/30 my-1"></div>

					<div class="grid grid-cols-2 gap-2 text-xs font-bold">
						<button
							onclick={async () => {
								if (isTauri && invoke) {
									await invoke('reboot_device', { serial: selectedDevice, mode: 'normal' });
									infoMessage = 'Rebooting device to Android System...';
								}
							}}
							class="rounded-xl bg-surface-container-high py-2 hover:brightness-105 text-on-surface disabled:opacity-40"
							disabled={!selectedDevice}
						>
							Reboot System
						</button>
						<button
							onclick={async () => {
								if (isTauri && invoke) {
									await invoke('reboot_device', { serial: selectedDevice, mode: 'bootloader' });
									infoMessage = 'Rebooting device to Bootloader...';
								}
							}}
							class="rounded-xl bg-surface-container-high py-2 hover:brightness-105 text-on-surface disabled:opacity-40"
							disabled={!selectedDevice}
						>
							Reboot Bootloader
						</button>
					</div>
				</section>
			</div>

			<!-- Col 2: Partition Flashing and Sideload -->
			<div class="flex flex-col gap-6">
				<!-- Partition Flasher -->
				<section class="rounded-[24px] bg-surface-container p-5 flex flex-col justify-between h-[230px]">
					<div>
						<header class="flex items-center gap-2 pb-2.5 mb-3">
							<span class="material-symbols-outlined text-primary text-[22px]">flash_on</span>
							<h3 class="text-sm font-bold text-on-surface">Manual Partition Flasher</h3>
						</header>

						<div class="grid grid-cols-3 gap-3 text-xs font-medium">
							<div class="flex flex-col gap-1">
								<label for="manual-p" class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant pl-0.5">Partition</label>
								<input
									id="manual-p"
									type="text"
									bind:value={manualPartition}
									class="bg-surface-container-high border border-outline-variant/40 rounded-xl px-3 py-1.5 font-semibold text-xs"
									disabled={flashingManual}
								/>
							</div>
							<div class="col-span-2 flex flex-col gap-1">
								<label for="manual-i" class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant pl-0.5">Host Image File Path</label>
								<input
									id="manual-i"
									type="text"
									bind:value={manualImagePath}
									placeholder="/path/to/boot.img"
									class="bg-surface-container-high border border-outline-variant/40 rounded-xl px-3 py-1.5 text-xs text-on-surface"
									disabled={flashingManual}
								/>
							</div>
						</div>
					</div>

					<button
						onclick={manualFlashPartition}
						class="rounded-xl bg-primary text-on-primary hover:brightness-105 transition-all text-xs font-bold py-2 disabled:opacity-50 w-full"
						disabled={flashingManual || !selectedDevice || !manualImagePath.trim()}
					>
						{#if flashingManual}
							<span class="animate-spin h-3.5 w-3.5 border-2 border-on-primary border-t-transparent rounded-full"></span>
							Flashing Partition...
						{:else}
							Flash Partition
						{/if}
					</button>
				</section>

				<!-- ADB Sideload -->
				<section class="rounded-[24px] bg-surface-container p-5 flex flex-col justify-between h-[210px]">
					<div>
						<header class="flex items-center gap-2 pb-2.5 mb-3">
							<span class="material-symbols-outlined text-primary text-[22px]">install_mobile</span>
							<h3 class="text-sm font-bold text-on-surface">ADB Sideload ZIP</h3>
						</header>

						<div class="flex flex-col gap-1.5 text-xs font-medium">
							<label for="sideload-z" class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant pl-0.5">OTA / Recovery ZIP File Path</label>
							<input
								id="sideload-z"
								type="text"
								bind:value={sideloadZipPath}
								placeholder="/path/to/update.zip"
								class="bg-surface-container-high border border-outline-variant/40 rounded-xl px-3 py-1.5 text-xs text-on-surface"
								disabled={sideloading}
							/>
						</div>
					</div>

					<button
						onclick={runSideload}
						class="rounded-xl bg-surface-container-highest hover:brightness-105 transition-all text-xs font-bold py-2 text-on-surface disabled:opacity-50 w-full"
						disabled={sideloading || !selectedDevice || !sideloadZipPath.trim()}
					>
						{#if sideloading}
							<span class="animate-spin h-3.5 w-3.5 border-2 border-on-surface border-t-transparent rounded-full"></span>
							Sideloading ZIP...
						{:else}
							Sideload Update package
						{/if}
					</button>
				</section>
			</div>

			<!-- Col 3: ROM Flash Plan & CLI Console -->
			<div class="flex flex-col gap-6">
				<!-- ROM Flash Folder scanner -->
				<section class="rounded-[24px] bg-surface-container p-5 flex flex-col justify-between min-h-[250px]">
					<div class="flex flex-col gap-3">
						<header class="flex items-center gap-2 pb-2.5">
							<span class="material-symbols-outlined text-primary text-[22px]">folder_zip</span>
							<h3 class="text-sm font-bold text-on-surface">ROM Flash Folder Planner</h3>
						</header>

						<div class="flex items-center gap-2 w-full">
							<input
								type="text"
								bind:value={romFolderPath}
								placeholder="Absolute path to ROM folder"
								class="bg-surface-container-high border border-outline-variant/40 rounded-xl px-3 py-1.5 text-xs flex-1"
								disabled={scanningRom || executingPlan}
							/>
							<button
								onclick={scanRomFolder}
								class="rounded-xl bg-surface-container-highest hover:brightness-105 py-1.5 px-3 text-xs font-bold text-on-surface disabled:opacity-40"
								disabled={scanningRom || executingPlan || !romFolderPath.trim()}
							>
								{#if scanningRom}
									Scan...
								{:else}
									Scan
								{/if}
							</button>
						</div>

						<!-- Parsed ROM steps list -->
						{#if scannedPlan}
							<div class="bg-surface-container-low p-3 rounded-xl max-h-36 overflow-y-auto text-[11px] font-medium flex flex-col gap-1.5">
								<p class="font-bold text-on-surface-variant text-[9px] uppercase tracking-wider mb-0.5">Identified Flash Files:</p>
								{#each scannedPlan.steps as step, idx (idx)}
									<div class="flex justify-between items-center px-2 py-1 rounded {currentFlashStepIndex === idx ? 'bg-primary/10 text-primary font-bold' : 'text-on-surface-variant'}">
										<span>{step.partition}</span>
										<span class="font-mono text-[9px] opacity-80">{step.image_file}</span>
									</div>
								{/each}
							</div>
						{/if}
					</div>

					{#if scannedPlan}
						<button
							onclick={executeFlashPlan}
							class="mt-4 rounded-xl bg-primary text-on-primary hover:brightness-105 transition-all text-xs font-bold py-2 w-full disabled:opacity-50"
							disabled={executingPlan || !selectedDevice}
						>
							{#if executingPlan}
								Flashing Step {currentFlashStepIndex + 1} of {scannedPlan.steps.length}...
							{:else}
								Execute Flash Plan
							{/if}
						</button>
					{/if}
				</section>

				<!-- Custom Fastboot CLI Console -->
				<section class="rounded-[24px] bg-surface-container p-5 flex-1 flex flex-col justify-between min-h-[220px]">
					<div class="flex-1 flex flex-col overflow-hidden min-h-0">
						<header class="flex items-center gap-2 pb-2.5 mb-3 shrink-0">
							<span class="material-symbols-outlined text-primary text-[22px]">terminal</span>
							<h3 class="text-sm font-bold text-on-surface">Fastboot Custom Command Console</h3>
						</header>

						<!-- Console display -->
						<div class="flex-1 bg-black rounded-2xl p-3 font-mono text-[10px] text-green-400 overflow-y-auto max-h-[140px] mb-3 leading-normal select-all">
							{#if cliLogs.length === 0}
								<span class="text-gray-500">// Console ready. Type custom fastboot arguments below (e.g. getvar all).</span>
							{:else}
								{#each cliLogs as log, i (i)}
									<div class="whitespace-pre-wrap">{log}</div>
								{/each}
							{/if}
						</div>
					</div>

					<form onsubmit={(e) => { e.preventDefault(); executeCliCommand(); }} class="flex gap-2 shrink-0">
						<div class="relative flex-1">
							<span class="absolute left-3 top-1/2 -translate-y-1/2 text-on-surface-variant font-mono text-xs select-none">$ fastboot</span>
							<input
								type="text"
								bind:value={customCommand}
								placeholder="getvar all"
								class="bg-surface-container-high border border-outline-variant/40 rounded-xl pl-20 pr-3 py-1.5 text-xs text-on-surface font-mono focus:outline-none w-full"
								disabled={executingCli}
							/>
						</div>
						<button
							type="submit"
							class="rounded-xl bg-primary text-on-primary hover:brightness-105 py-1.5 px-4 text-xs font-bold disabled:opacity-50"
							disabled={executingCli || !selectedDevice || !customCommand.trim()}
						>
							Run
						</button>
					</form>
				</section>
			</div>
		</div>
	</div>
</main>

<style>
	@keyframes fadeIn {
		from {
			opacity: 0;
			transform: scale(0.98);
		}
		to {
			opacity: 1;
			transform: scale(1);
		}
	}
	.animate-fade-in {
		animation: fadeIn 0.18s cubic-bezier(0.2, 0.8, 0.2, 1) forwards;
	}
</style>
