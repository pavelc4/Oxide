<script lang="ts">
	/* eslint-disable svelte/no-navigation-without-resolve */
	import { onMount } from 'svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import { goto } from '$app/navigation';

	// Subcomponents
	import AppListItem from './components/AppListItem.svelte';
	import AppDetailsPanel from './components/AppDetailsPanel.svelte';
	import InstallModal from './components/InstallModal.svelte';
	import BatchActionBar from './components/BatchActionBar.svelte';
	import PaginationControls from './components/PaginationControls.svelte';

	let invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | undefined;
	let isTauri = $state(false);

	interface Device {
		id: string;
		name: string;
		status: string;
		connection: string;
	}

	interface AppDetails {
		package_name: string;
		version_name: string | null;
		version_code: number | null;
		label: string | null;
		install_location: string;
		flags: string[];
		first_install_time: string | null;
		last_update_time: string | null;
		apk_path: string | null;
		data_dir: string | null;
		is_system_app: boolean;
		is_enabled: boolean;
	}

	// State variables
	let devices = $state<Device[]>([]);
	let selectedDevice = $state<string>('');
	let loading = $state(true);
	let error = $state('');
	let infoMessage = $state('');
	let searchQuery = $state('');
	let currentFilter = $state<'all' | 'user' | 'system' | 'enabled' | 'disabled'>('all');
	
	// Package lists
	let allPackages = $state<string[]>([]);
	let selectedPackages = new SvelteSet<string>();

	// Local cache mapping: { [deviceSerial]: { [filterTab]: packageNames[] } }
	let packageCache = $state<Record<string, Record<string, string[]>>>({});

	// Pagination variables
	let currentPage = $state(1);
	const pageSize = 50;

	// Host Downloads folder for Pull APK
	let downloadsDir = $state('');

	// Detailed single app view
	let selectedAppDetails = $state<AppDetails | null>(null);
	let loadingDetails = $state(false);

	// Install APK modal state
	let showInstallModal = $state(false);
	let installing = $state(false);

	// Derived state for filtered list (in-memory search)
	let filteredPackages = $derived(
		allPackages.filter((pkg) => pkg.toLowerCase().includes(searchQuery.toLowerCase()))
	);

	// Derived pagination bounds
	let totalPages = $derived(Math.ceil(filteredPackages.length / pageSize));
	let paginatedPackages = $derived(
		filteredPackages.slice((currentPage - 1) * pageSize, currentPage * pageSize)
	);

	// Reset page number back to 1 on search or filter tab change
	$effect(() => {
		if (searchQuery !== undefined || currentFilter !== undefined) {
			currentPage = 1;
		}
	});

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
		await fetchDownloadsDir();
	});

	async function fetchDownloadsDir() {
		try {
			if (isTauri && invoke) {
				const dirs = (await invoke('get_common_directories_cmd')) as [string, string][];
				const dl = dirs.find((d) => d[0] === 'Downloads');
				if (dl) {
					downloadsDir = dl[1];
				}
			}
		} catch (e) {
			console.error('Failed to get host downloads path', e);
		}
	}

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
					{ id: 'mock-device-123', name: 'Mock Galaxy S24 Ultra', status: 'Online', connection: 'USB' },
					{ id: 'mock-device-456', name: 'Mock Pixel 8 Pro', status: 'Online', connection: 'Wireless' }
				];
			}

			if (devices.length > 0) {
				selectedDevice = devices[0].id;
				await fetchPackages();
			} else {
				selectedDevice = '';
				allPackages = [];
			}
		} catch (e) {
			error = String(e);
			devices = [];
		} finally {
			loading = false;
		}
	}

	async function fetchPackages() {
		if (!selectedDevice) return;
		selectedPackages.clear();

		// Check cache for instant tab transitions
		if (packageCache[selectedDevice]?.[currentFilter]) {
			allPackages = packageCache[selectedDevice][currentFilter];
			loading = false;
		} else {
			loading = true;
		}
		error = '';

		// Map filter for Tauri endpoint
		let tauriFilter = 'all';
		if (currentFilter === 'system') tauriFilter = 'system';
		else if (currentFilter === 'user') tauriFilter = 'thirdparty';
		else if (currentFilter === 'enabled') tauriFilter = 'enabled';
		else if (currentFilter === 'disabled') tauriFilter = 'disabled';

		try {
			let packagesList: string[] = [];
			if (isTauri && invoke) {
				packagesList = (await invoke('list_packages', {
					serial: selectedDevice,
					filter: tauriFilter
				})) as string[];
			} else {
				// Mock Packages
				if (currentFilter === 'all') {
					packagesList = [
						'com.whatsapp',
						'com.instagram.android',
						'com.android.chrome',
						'com.google.android.youtube',
						'com.android.settings',
						'com.android.systemui',
						'org.mozilla.firefox',
						'com.spotify.music',
						'com.netflix.mediaclient',
						'com.termux',
						'com.android.providers.contacts',
						'com.android.vending',
						'com.google.android.gms'
					];
				} else if (currentFilter === 'system') {
					packagesList = [
						'com.android.chrome',
						'com.android.settings',
						'com.android.systemui',
						'com.android.providers.contacts',
						'com.android.vending',
						'com.google.android.gms'
					];
				} else if (currentFilter === 'user') {
					packagesList = [
						'com.whatsapp',
						'com.instagram.android',
						'org.mozilla.firefox',
						'com.spotify.music',
						'com.netflix.mediaclient',
						'com.termux'
					];
				} else if (currentFilter === 'enabled') {
					packagesList = [
						'com.whatsapp',
						'com.instagram.android',
						'com.android.chrome',
						'com.google.android.youtube',
						'com.android.settings',
						'org.mozilla.firefox',
						'com.spotify.music',
						'com.netflix.mediaclient',
						'com.termux'
					];
				} else {
					packagesList = ['com.android.systemui.theme.mock', 'com.bloatware.carrier.app'];
				}
			}

			allPackages = packagesList || [];
			
			// Save in cache
			if (!packageCache[selectedDevice]) {
				packageCache[selectedDevice] = {};
			}
			packageCache[selectedDevice][currentFilter] = allPackages;
		} catch (e) {
			error = `Failed to list packages: ${e}`;
			if (!packageCache[selectedDevice]?.[currentFilter]) {
				allPackages = [];
			}
		} finally {
			loading = false;
		}
	}

	async function selectDeviceChanged(id: string) {
		selectedDevice = id;
		selectedAppDetails = null;
		await fetchPackages();
	}

	async function selectFilterChanged(filter: 'all' | 'user' | 'system' | 'enabled' | 'disabled') {
		currentFilter = filter;
		selectedAppDetails = null;
		await fetchPackages();
	}

	// Toggle check for batch select
	function toggleSelectPackage(pkg: string) {
		if (selectedPackages.has(pkg)) {
			selectedPackages.delete(pkg);
		} else {
			selectedPackages.add(pkg);
		}
	}

	function toggleSelectAll() {
		if (selectedPackages.size === filteredPackages.length) {
			selectedPackages.clear();
		} else {
			filteredPackages.forEach((p) => selectedPackages.add(p));
		}
	}

	// Single Package details
	async function viewPackageDetails(pkg: string) {
		loadingDetails = true;
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				selectedAppDetails = (await invoke('get_package_info', {
					serial: selectedDevice,
					packageName: pkg
				})) as AppDetails;
			} else {
				// Mock app details
				const isSys = pkg.startsWith('com.android.') || pkg.startsWith('com.google.');
				selectedAppDetails = {
					package_name: pkg,
					version_name: '14.22.45',
					version_code: 4892100,
					label: pkg.split('.').pop()?.toUpperCase() || pkg,
					install_location: 'InternalOnly',
					flags: ['HAS_CODE', isSys ? 'SYSTEM' : 'USER'],
					first_install_time: '2025-06-12 11:24:02',
					last_update_time: '2026-07-01 16:50:31',
					apk_path: `/data/app/~~mockhash/${pkg}-1/base.apk`,
					data_dir: `/data/user/0/${pkg}`,
					is_system_app: isSys,
					is_enabled: !pkg.includes('bloatware')
				};
			}
		} catch (e) {
			error = `Failed to get package info: ${e}`;
			selectedAppDetails = null;
		} finally {
			loadingDetails = false;
		}
	}

	// Actions on individual packages
	async function forceStopApp(pkg: string) {
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				await invoke('force_stop_package', { serial: selectedDevice, packageName: pkg });
			}
			infoMessage = `Force stopped ${pkg} successfully.`;
		} catch (e) {
			error = `Failed to force stop: ${e}`;
		}
	}

	async function clearAppData(pkg: string) {
		if (!confirm(`Are you sure you want to clear all data for ${pkg}? This cannot be undone.`)) return;
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				await invoke('clear_package_data', { serial: selectedDevice, packageName: pkg });
			}
			infoMessage = `Cleared data for ${pkg} successfully.`;
		} catch (e) {
			error = `Failed to clear data: ${e}`;
		}
	}

	async function toggleAppStatus(pkg: string, enable: boolean) {
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				if (enable) {
					await invoke('enable_package', { serial: selectedDevice, packageName: pkg });
				} else {
					await invoke('disable_package', { serial: selectedDevice, packageName: pkg });
				}
			}
			if (selectedAppDetails && selectedAppDetails.package_name === pkg) {
				selectedAppDetails.is_enabled = enable;
			}
			infoMessage = `${enable ? 'Enabled' : 'Disabled'} ${pkg} successfully.`;
			await fetchPackages();
		} catch (e) {
			error = `Failed to change status: ${e}`;
		}
	}

	async function uninstallApp(pkg: string) {
		if (!confirm(`Uninstall ${pkg}?`)) return;
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				const res = (await invoke('uninstall_package', {
					serial: selectedDevice,
					packageName: pkg
				})) as { success: boolean; message: string };
				if (!res.success) {
					throw new Error(res.message);
				}
			}
			infoMessage = `Uninstalled ${pkg} successfully.`;
			selectedAppDetails = null;
			await fetchPackages();
		} catch (e) {
			error = `Failed to uninstall: ${e}`;
		}
	}

	// New Feature: Launch App
	async function launchApp(pkg: string) {
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				await invoke('start_package_app', { serial: selectedDevice, packageName: pkg });
				infoMessage = `Launched ${pkg} successfully.`;
			} else {
				infoMessage = `Launched ${pkg} successfully (mock).`;
			}
		} catch (e) {
			error = `Failed to launch app: ${e}`;
		}
	}

	// New Feature: Pull APK
	async function pullApk(pkg: string) {
		error = '';
		infoMessage = '';
		try {
			const destination = downloadsDir || '/tmp';
			if (isTauri && invoke) {
				const savePath = (await invoke('pull_package_apk', {
					serial: selectedDevice,
					packageName: pkg,
					destination
				})) as string;
				infoMessage = `APK downloaded successfully to: ${savePath}`;
			} else {
				infoMessage = `APK downloaded successfully to: ${destination}/${pkg}.apk (mock)`;
			}
		} catch (e) {
			error = `Failed to pull APK: ${e}`;
		}
	}

	// Batch Actions
	async function handleBatchUninstall() {
		const size = selectedPackages.size;
		if (size === 0) return;
		if (!confirm(`Are you sure you want to uninstall ${size} packages?`)) return;
		
		loading = true;
		error = '';
		infoMessage = '';
		try {
			let failCount = 0;
			for (const pkg of selectedPackages) {
				try {
					if (isTauri && invoke) {
						const res = (await invoke('uninstall_package', {
							serial: selectedDevice,
							packageName: pkg
						})) as { success: boolean; message: string };
						if (!res.success) failCount++;
					}
				} catch {
					failCount++;
				}
			}
			infoMessage = `Batch uninstall completed. Checked ${size} apps, ${size - failCount} uninstalled, ${failCount} failed.`;
			selectedPackages.clear();
			await fetchPackages();
		} catch (e) {
			error = `Batch operation failed: ${e}`;
		} finally {
			loading = false;
		}
	}

	async function handleBatchStatus(enable: boolean) {
		const size = selectedPackages.size;
		if (size === 0) return;
		loading = true;
		error = '';
		infoMessage = '';
		try {
			const pkgs = Array.from(selectedPackages);
			if (isTauri && invoke) {
				if (enable) {
					await invoke('enable_multiple_packages_cmd', { serial: selectedDevice, packageNames: pkgs });
				} else {
					await invoke('disable_multiple_packages_cmd', { serial: selectedDevice, packageNames: pkgs });
				}
			}
			infoMessage = `Batch ${enable ? 'enable' : 'disable'} completed for ${size} apps.`;
			selectedPackages.clear();
			await fetchPackages();
		} catch (e) {
			error = `Batch status update failed: ${e}`;
		} finally {
			loading = false;
		}
	}

	// APK Installation
	async function installApk(apkPath: string) {
		installing = true;
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				const res = (await invoke('install_package', {
					serial: selectedDevice,
					apkPath
				})) as { success: boolean; message: string; package_name: string | null };
				if (!res.success) {
					throw new Error(res.message);
				}
				infoMessage = `Installed APK successfully: ${res.package_name || ''}`;
			} else {
				await new Promise((resolve) => setTimeout(resolve, 1500));
				infoMessage = `Installed APK successfully in mock mode.`;
			}
			showInstallModal = false;
			await fetchPackages();
		} catch (e) {
			error = `Installation failed: ${e}`;
		} finally {
			installing = false;
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
					App Manager
					{#if !isTauri}
						<span
							class="text-xs bg-error text-on-error px-3 py-1 rounded-full font-medium tracking-normal border border-error/30"
							>MOCK MODE</span
						>
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
					<span class="material-symbols-outlined text-[18px]">refresh</span>
				</button>
			</div>
		</header>

		<!-- Alert Messages -->
		{#if error}
			<div
				class="bg-error/15 text-error border border-error/30 p-4 rounded-2xl mb-4 font-medium flex items-center gap-3 text-sm shrink-0"
			>
				<span class="material-symbols-outlined text-[20px]">error</span>
				<div class="flex-1 break-words">{error}</div>
				<button onclick={() => (error = '')} class="hover:opacity-85 text-xs font-semibold uppercase">Dismiss</button>
			</div>
		{/if}

		{#if infoMessage}
			<div
				class="bg-primary/10 text-primary p-4 rounded-2xl mb-4 font-medium flex items-center gap-3 text-sm shrink-0"
			>
				<span class="material-symbols-outlined text-[20px]">check_circle</span>
				<div class="flex-1 break-words">{infoMessage}</div>
				<button onclick={() => (infoMessage = '')} class="hover:opacity-85 text-xs font-semibold uppercase">Dismiss</button>
			</div>
		{/if}

		<!-- Toolbar / Search and Tabs -->
		<section class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-6 shrink-0">
			<!-- Segmented Controls / Filter Tabs -->
			<div class="flex rounded-full bg-surface-container p-1 w-fit">
				{#each ['all', 'user', 'system', 'enabled', 'disabled'] as filter (filter)}
					<button
						onclick={() => selectFilterChanged(filter as 'all' | 'user' | 'system' | 'enabled' | 'disabled')}
						class="px-4 py-1.5 rounded-full text-xs font-semibold uppercase tracking-wider transition-all duration-200 {currentFilter === filter ? 'bg-secondary-container text-on-secondary-container' : 'text-on-surface-variant hover:text-on-surface'}"
					>
						{filter}
					</button>
				{/each}
			</div>

			<!-- Search and Install Button -->
			<div class="flex items-center gap-3 w-full md:w-auto">
				<div class="relative flex-1 md:flex-none">
					<span
						class="material-symbols-outlined absolute left-4 top-1/2 -translate-y-1/2 text-on-surface-variant text-[18px]"
						>search</span
					>
					<input
						type="text"
						bind:value={searchQuery}
						placeholder="Search package name..."
						class="bg-surface-container-high rounded-full pl-10 pr-4 py-2 text-sm text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all w-full md:w-64 border border-outline-variant/20"
					/>
				</div>
				<button
					onclick={() => (showInstallModal = true)}
					class="flex items-center gap-2 rounded-full bg-primary px-5 py-2 text-sm font-semibold text-on-primary hover:brightness-105 transition-all shrink-0"
				>
					<span class="material-symbols-outlined text-[18px]">install_mobile</span>
					Install APK
				</button>
			</div>
		</section>

		<!-- Main Workspace (Split View) -->
		<div class="flex flex-1 gap-6 overflow-hidden min-h-0">
			<!-- Packages List Column -->
			<div class="flex flex-1 flex-col overflow-hidden rounded-[24px] bg-surface-container p-5">
				<!-- List Title and Header Select All -->
				<div class="flex items-center justify-between pb-3 mb-2 shrink-0">
					<div class="flex items-center gap-3">
						<input
							type="checkbox"
							checked={filteredPackages.length > 0 && selectedPackages.size === filteredPackages.length}
							onclick={toggleSelectAll}
							class="w-4 h-4 rounded text-primary focus:ring-primary/50 border-outline-variant"
							disabled={filteredPackages.length === 0}
						/>
						<span class="text-sm font-bold text-on-surface">
							Packages ({filteredPackages.length})
						</span>
					</div>
					{#if selectedPackages.size > 0}
						<span class="text-xs font-semibold bg-primary-container text-on-primary-container px-3 py-1 rounded-full">
							{selectedPackages.size} Selected
						</span>
					{/if}
				</div>

				<!-- Scrollable List -->
				<div class="flex-1 overflow-y-auto pr-1 flex flex-col gap-1">
					{#if loading && allPackages.length === 0}
						<!-- Skeletal Loading -->
						<div class="flex flex-col gap-2.5">
							{#each Array(6) as dummy, i (i)}
								<div class="h-[52px] bg-surface-container-high rounded-2xl animate-pulse" id="skeleton-row-{i}-{dummy || ''}"></div>
							{/each}
						</div>
					{:else if filteredPackages.length === 0}
						<div class="flex flex-col items-center justify-center p-12 text-center h-full">
							<span class="material-symbols-outlined text-[48px] text-on-surface-variant opacity-60 mb-3">category</span>
							<p class="text-sm text-on-surface-variant font-medium">No packages found matching search or filter</p>
						</div>
					{:else}
						<div class="flex flex-col gap-1">
							{#each paginatedPackages as pkg (pkg)}
								<AppListItem
									{pkg}
									selected={selectedPackages.has(pkg)}
									isActive={selectedAppDetails?.package_name === pkg}
									ontoggle={() => toggleSelectPackage(pkg)}
									onview={() => viewPackageDetails(pkg)}
								/>
							{/each}
						</div>
					{/if}
				</div>

				<!-- Pagination Controls at bottom of column list -->
				<PaginationControls
					{currentPage}
					{totalPages}
					onprev={() => { if (currentPage > 1) currentPage--; }}
					onnext={() => { if (currentPage < totalPages) currentPage++; }}
				/>
			</div>

			<!-- App Details Panel (Side panel) -->
			<AppDetailsPanel
				{selectedAppDetails}
				{loadingDetails}
				onforcestop={forceStopApp}
				oncleardata={clearAppData}
				ontogglestatus={toggleAppStatus}
				onuninstall={uninstallApp}
				onpullapk={pullApk}
				onlaunch={launchApp}
			/>
		</div>

		<!-- Batch Action Bar (Float up from bottom) -->
		<BatchActionBar
			selectedCount={selectedPackages.size}
			onenable={() => handleBatchStatus(true)}
			ondisable={() => handleBatchStatus(false)}
			onuninstall={handleBatchUninstall}
			onclear={() => selectedPackages.clear()}
		/>

		<!-- Install APK Dialog -->
		<InstallModal
			show={showInstallModal}
			installing={installing}
			oncancel={() => (showInstallModal = false)}
			oninstall={installApk}
		/>
	</div>
</main>
