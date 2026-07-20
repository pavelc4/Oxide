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
	import ShapeBadge from '$lib/components/ShapeBadge.svelte';

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
	let totalPages = $derived(Math.max(1, Math.ceil(filteredPackages.length / pageSize)));
	let paginatedPackages = $derived(
		filteredPackages.slice((currentPage - 1) * pageSize, currentPage * pageSize)
	);

	// Reset page number back to 1 on search or filter tab change
	$effect(() => {
		if (searchQuery !== undefined || currentFilter !== undefined) {
			currentPage = 1;
		}
	});

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
		await fetchDownloadsDir();
	});

	async function fetchDownloadsDir() {
		try {
			if (isTauri && invoke) {
				const dirs = await safeInvoke<[string, string][]>('get_common_directories_cmd');
				const dl = dirs.find((d) => d[0] === 'Downloads');
				if (dl) {
					downloadsDir = dl[1];
				}
			}
		} catch (e) {
			console.warn('Downloads directory fetch info:', e);
		}
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
						connection: 'USB'
					}));
				} else {
					devices = [];
				}
			} else {
				devices = [
					{ id: 'mock-device-123', name: 'Google Pixel 8 Pro', status: 'Online', connection: 'USB 3.2' },
					{ id: 'mock-device-456', name: 'Samsung Galaxy S24 Ultra', status: 'Online', connection: 'Wireless' }
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

		let tauriFilter = 'all';
		if (currentFilter === 'system') tauriFilter = 'system';
		else if (currentFilter === 'user') tauriFilter = 'thirdparty';
		else if (currentFilter === 'enabled') tauriFilter = 'enabled';
		else if (currentFilter === 'disabled') tauriFilter = 'disabled';

		try {
			let packagesList: string[] = [];
			if (isTauri && invoke) {
				packagesList = await safeInvoke<string[]>('list_packages', {
					serial: selectedDevice,
					filter: tauriFilter
				});
			} else {
				// Mock Packages for browser preview
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

	async function selectFilterChanged(filter: 'all' | 'user' | 'system' | 'enabled' | 'disabled') {
		currentFilter = filter;
		selectedAppDetails = null;
		await fetchPackages();
	}

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

	async function viewPackageDetails(pkg: string) {
		loadingDetails = true;
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				selectedAppDetails = await safeInvoke<AppDetails>('get_package_info', {
					serial: selectedDevice,
					package: pkg
				});
			} else {
				const isSys = pkg.startsWith('com.android.') || pkg.startsWith('com.google.');
				const namePart = pkg.split('.').pop() || pkg;
				selectedAppDetails = {
					package_name: pkg,
					version_name: '14.22.45',
					version_code: 4892100,
					label: namePart.charAt(0).toUpperCase() + namePart.slice(1),
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

	async function forceStopApp(pkg: string) {
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				await safeInvoke('force_stop_package', { serial: selectedDevice, package: pkg });
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
				await safeInvoke('clear_package', { serial: selectedDevice, package: pkg });
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
					await safeInvoke('enable_package', { serial: selectedDevice, package: pkg });
				} else {
					await safeInvoke('disable_package', { serial: selectedDevice, package: pkg });
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
				await safeInvoke('uninstall_package', {
					serial: selectedDevice,
					package: pkg
				});
			}
			infoMessage = `Uninstalled ${pkg} successfully.`;
			selectedAppDetails = null;
			await fetchPackages();
		} catch (e) {
			error = `Failed to uninstall: ${e}`;
		}
	}

	async function launchApp(pkg: string) {
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				await safeInvoke('start_package_app', { serial: selectedDevice, package: pkg });
				infoMessage = `Launched ${pkg} successfully.`;
			} else {
				infoMessage = `Launched ${pkg} successfully (mock).`;
			}
		} catch (e) {
			error = `Failed to launch app: ${e}`;
		}
	}

	async function pullApk(pkg: string) {
		error = '';
		infoMessage = '';
		try {
			const destination = downloadsDir || '/tmp';
			if (isTauri && invoke) {
				const savePath = await safeInvoke<string>('pull_package_apk', {
					serial: selectedDevice,
					package: pkg,
					destination
				});
				infoMessage = `APK downloaded successfully to: ${savePath}`;
			} else {
				infoMessage = `APK downloaded successfully to: ${destination}/${pkg}.apk (mock)`;
			}
		} catch (e) {
			error = `Failed to pull APK: ${e}`;
		}
	}

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
						const res = await safeInvoke<{ success: boolean; message: string }>('uninstall_package', {
							serial: selectedDevice,
							package: pkg
						});
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
					await safeInvoke('enable_multiple_packages_cmd', { serial: selectedDevice, packageNames: pkgs });
				} else {
					await safeInvoke('disable_multiple_packages_cmd', { serial: selectedDevice, packageNames: pkgs });
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

	async function installApk(apkPath: string) {
		installing = true;
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				await safeInvoke('install_apk', {
					serial: selectedDevice,
					path: apkPath
				});
				infoMessage = `Installed APK successfully.`;
			} else {
				await new Promise((resolve) => setTimeout(resolve, 1200));
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
	<div class="flex flex-1 flex-col overflow-hidden rounded-[32px] bg-surface-container-low p-6 lg:p-8 relative gap-5 shadow-sm">
		
		<!-- Clean Page Header -->
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
						<ShapeBadge icon="apps" shape="cookie7" size={40} iconSize={20} />
						<h2 class="text-2xl font-bold tracking-tight text-on-surface">App Manager Studio</h2>
						{#if !isTauri}
							<span class="text-[10px] bg-warning/15 text-warning px-2.5 py-0.5 rounded-full font-bold uppercase tracking-wider">MOCK MODE</span>
						{/if}
					</div>
					<p class="text-xs text-on-surface-variant/80 font-medium mt-0.5">Manage, install, disable bloatware, & export Android packages</p>
				</div>
			</div>

			<div class="flex items-center gap-3">
				<button
					onclick={fetchPackages}
					class="flex h-9 w-9 items-center justify-center rounded-full bg-surface-container hover:bg-surface-container-high text-on-surface-variant transition-all hover:scale-105 active:scale-95 shadow-xs"
					title="Refresh Package List"
				>
					<span class="material-symbols-outlined text-[18px] {loading ? 'animate-spin' : ''}">refresh</span>
				</button>
				<button
					onclick={() => (showInstallModal = true)}
					class="flex items-center gap-2 rounded-full bg-primary px-4 py-2 text-xs font-bold text-on-primary hover:brightness-110 transition-all shadow-sm active:scale-95"
				>
					<ShapeBadge shape="pixelCircle" icon="install_mobile" size={24} iconSize={13} bgClass="bg-on-primary/20" textClass="text-on-primary" />
					Install APK
				</button>
			</div>
		</header>

		<!-- Alert Messages -->
		{#if error}
			<div class="bg-error/15 text-error p-3.5 rounded-2xl font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[20px]">error</span>
				<div class="flex-1 break-words font-semibold">{error}</div>
				<button onclick={() => (error = '')} class="hover:opacity-85 text-[10px] font-bold uppercase tracking-wider bg-error/20 px-2.5 py-1 rounded-lg">Dismiss</button>
			</div>
		{/if}

		{#if infoMessage}
			<div class="bg-primary/10 text-primary p-3.5 rounded-2xl font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[20px]">check_circle</span>
				<div class="flex-1 break-words font-semibold">{infoMessage}</div>
				<button onclick={() => (infoMessage = '')} class="hover:opacity-85 text-[10px] font-bold uppercase tracking-wider bg-primary/20 px-2.5 py-1 rounded-lg">Dismiss</button>
			</div>
		{/if}

		<!-- Toolbar / Search and Tabs -->
		<section class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 shrink-0">
			<!-- Segmented Controls / Filter Tabs -->
			<div class="flex rounded-full bg-surface-container p-1 w-fit shadow-xs">
				{#each ['all', 'user', 'system', 'enabled', 'disabled'] as filter (filter)}
					<button
						onclick={() => selectFilterChanged(filter as 'all' | 'user' | 'system' | 'enabled' | 'disabled')}
						class="px-4 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider transition-all duration-200 {currentFilter === filter ? 'bg-secondary-container text-on-secondary-container shadow-xs' : 'text-on-surface-variant hover:text-on-surface'}"
					>
						{filter}
					</button>
				{/each}
			</div>

			<!-- Search Bar -->
			<div class="relative w-full sm:w-72">
				<span class="material-symbols-outlined absolute left-3.5 top-1/2 -translate-y-1/2 text-on-surface-variant text-[18px]">search</span>
				<input
					type="text"
					bind:value={searchQuery}
					placeholder="Search package name..."
					class="bg-surface-container rounded-full pl-10 pr-4 py-2 text-xs font-medium text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 transition-all w-full shadow-xs"
				/>
			</div>
		</section>

		<!-- Main Workspace (Split View) -->
		<div class="flex flex-1 gap-5 overflow-hidden min-h-0">
			<!-- Packages List Column -->
			<div class="flex flex-1 flex-col overflow-hidden rounded-[32px] bg-surface-container p-5 shadow-sm">
				<!-- Package Header Bar -->
				<div class="flex items-center justify-between pb-3 mb-2 shrink-0 text-xs font-bold text-on-surface">
					<div class="flex items-center gap-2">
						<span class="text-xs font-bold text-on-surface">
							Packages ({filteredPackages.length})
						</span>
						<span class="text-[10px] text-on-surface-variant/70 font-medium hidden sm:inline">(Ctrl + Click to multi-select)</span>
					</div>
					<div class="flex items-center gap-2">
						{#if selectedPackages.size > 0}
							<button
								onclick={() => selectedPackages.clear()}
								class="text-[10px] font-bold text-on-surface-variant hover:text-on-surface px-2 py-0.5 rounded-lg bg-surface-container-high transition-all"
							>
								Clear Selection ({selectedPackages.size})
							</button>
						{:else}
							<button
								onclick={toggleSelectAll}
								class="text-[10px] font-bold text-primary hover:underline px-2 py-0.5 rounded-lg"
							>
								Select All
							</button>
						{/if}
					</div>
				</div>

				<!-- Scrollable List -->
				<div class="flex-1 overflow-y-auto pr-1 flex flex-col gap-1.5">
					{#if loading && allPackages.length === 0}
						<!-- Skeletal Loading -->
						<div class="flex flex-col gap-2.5">
							{#each Array(6) as dummy, i (i)}
								<div class="h-[52px] bg-surface-container-high rounded-2xl animate-pulse" id="skeleton-row-{i}-{dummy || ''}"></div>
							{/each}
						</div>
					{:else if filteredPackages.length === 0}
						<div class="flex flex-col items-center justify-center p-12 text-center h-full">
							<span class="material-symbols-outlined text-[48px] text-on-surface-variant opacity-50 mb-3">category</span>
							<p class="text-xs text-on-surface-variant font-medium">No packages found matching search or filter</p>
						</div>
					{:else}
						<div class="flex flex-col gap-1.5">
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
