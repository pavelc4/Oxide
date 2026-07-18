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

	interface FileEntry {
		name: string;
		is_dir: boolean;
		size: number;
		permissions: string;
		modified: number;
	}

	// Devices State
	let devices = $state<Device[]>([]);
	let selectedDevice = $state<string>('');
	let loading = $state(true);
	let error = $state('');
	let infoMessage = $state('');

	// File Explorer State
	let currentPath = $state('/sdcard');
	let files = $state<FileEntry[]>([]);
	let loadingFiles = $state(false);
	let searchQuery = $state('');
	let viewMode = $state<'list' | 'grid'>('list');
	let selectedFile = $state<FileEntry | null>(null);

	// Sorting
	let sortBy = $state<'name' | 'size' | 'modified'>('name');
	let sortAsc = $state(true);

	// Action Modals State
	let showNewFolderModal = $state(false);
	let newFolderName = $state('');
	let showRenameModal = $state(false);
	let renameNewName = $state('');
	let showPushModal = $state(false);
	let localPushPath = $state('');
	let showPullModal = $state(false);
	let localPullPath = $state('');

	// Bookmarked Quick Locations
	const quickLocations = [
		{ label: 'Internal Storage', path: '/sdcard', icon: 'smartphone' },
		{ label: 'Downloads', path: '/sdcard/Download', icon: 'download' },
		{ label: 'DCIM (Photos)', path: '/sdcard/DCIM', icon: 'photo_camera' },
		{ label: 'Pictures', path: '/sdcard/Pictures', icon: 'image' },
		{ label: 'Music', path: '/sdcard/Music', icon: 'library_music' },
		{ label: 'Movies', path: '/sdcard/Movies', icon: 'movie' },
		{ label: 'Documents', path: '/sdcard/Documents', icon: 'description' },
		{ label: 'System Root', path: '/system', icon: 'settings' },
		{ label: 'Data Partition', path: '/data', icon: 'database' }
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
					{ id: 'fastboot-pixel-8', name: 'Google Pixel 8 Pro', status: 'Online', connection: 'USB' },
					{ id: 'adb-galaxy-s24', name: 'Samsung Galaxy S24 Ultra', status: 'Online', connection: 'USB' }
				];
			}

			if (devices.length > 0) {
				selectedDevice = devices[0].id;
				await navigateTo(currentPath);
			} else {
				selectedDevice = '';
				files = [];
			}
		} catch (e) {
			error = String(e);
			devices = [];
		} finally {
			loading = false;
		}
	}

	async function navigateTo(path: string) {
		if (!selectedDevice) return;
		loadingFiles = true;
		error = '';
		selectedFile = null;

		// Clean path slashes
		const targetPath = path.replace(/\/+/g, '/').replace(/\/$/, '') || '/';
		currentPath = targetPath;

		try {
			if (isTauri && invoke) {
				files = await safeInvoke<FileEntry[]>('list_files', {
					serial: selectedDevice,
					path: targetPath
				});
			} else {
				await new Promise((resolve) => setTimeout(resolve, 350));
				files = getMockFiles(targetPath);
			}
		} catch (e) {
			error = `Failed to list files in ${targetPath}: ${e}`;
			files = [];
		} finally {
			loadingFiles = false;
		}
	}

	function getMockFiles(path: string): FileEntry[] {
		if (path === '/sdcard/Download') {
			return [
				{ name: 'Magisk-v27.0.apk', is_dir: false, size: 11453210, permissions: '100644', modified: 1718000000 },
				{ name: 'custom_kernel.zip', is_dir: false, size: 28453210, permissions: '100644', modified: 1718100000 },
				{ name: 'ubuntu_touch_payload.iso', is_dir: false, size: 845321000, permissions: '100644', modified: 1718200000 }
			];
		}
		return [
			{ name: 'Android', is_dir: true, size: 4096, permissions: '40755', modified: 1710000000 },
			{ name: 'DCIM', is_dir: true, size: 4096, permissions: '40755', modified: 1711000000 },
			{ name: 'Download', is_dir: true, size: 4096, permissions: '40755', modified: 1712000000 },
			{ name: 'Pictures', is_dir: true, size: 4096, permissions: '40755', modified: 1713000000 },
			{ name: 'Music', is_dir: true, size: 4096, permissions: '40755', modified: 1714000000 },
			{ name: 'Documents', is_dir: true, size: 4096, permissions: '40755', modified: 1715000000 },
			{ name: 'build.prop', is_dir: false, size: 4820, permissions: '100644', modified: 1716000000 },
			{ name: 'boot_animation.zip', is_dir: false, size: 14520900, permissions: '100644', modified: 1717000000 }
		];
	}

	function handleItemClick(file: FileEntry) {
		if (file.is_dir) {
			const newPath = currentPath === '/' ? `/${file.name}` : `${currentPath}/${file.name}`;
			navigateTo(newPath);
		} else {
			selectedFile = file;
		}
	}

	function navigateUp() {
		if (currentPath === '/' || currentPath === '') return;
		const parts = currentPath.split('/').filter(Boolean);
		parts.pop();
		const parentPath = '/' + parts.join('/');
		navigateTo(parentPath);
	}

	async function handleCreateFolder() {
		if (!newFolderName.trim() || !selectedDevice) return;
		error = '';
		infoMessage = '';
		const target = currentPath === '/' ? `/${newFolderName.trim()}` : `${currentPath}/${newFolderName.trim()}`;
		try {
			if (isTauri && invoke) {
				await safeInvoke('create_dir', { serial: selectedDevice, path: target });
			}
			infoMessage = `Folder created: ${newFolderName}`;
			showNewFolderModal = false;
			newFolderName = '';
			await navigateTo(currentPath);
		} catch (e) {
			error = `Failed to create folder: ${e}`;
		}
	}

	async function handleRename() {
		if (!selectedFile || !renameNewName.trim() || !selectedDevice) return;
		error = '';
		infoMessage = '';
		const oldPath = currentPath === '/' ? `/${selectedFile.name}` : `${currentPath}/${selectedFile.name}`;
		const newPath = currentPath === '/' ? `/${renameNewName.trim()}` : `${currentPath}/${renameNewName.trim()}`;
		try {
			if (isTauri && invoke) {
				await safeInvoke('rename_file', { serial: selectedDevice, src: oldPath, dst: newPath });
			}
			infoMessage = `Renamed to ${renameNewName}`;
			showRenameModal = false;
			renameNewName = '';
			selectedFile = null;
			await navigateTo(currentPath);
		} catch (e) {
			error = `Failed to rename: ${e}`;
		}
	}

	async function handleDelete(file: FileEntry) {
		if (!selectedDevice) return;
		if (!confirm(`Are you sure you want to delete ${file.is_dir ? 'folder' : 'file'} "${file.name}"?`)) return;
		error = '';
		infoMessage = '';
		const pathToDelete = currentPath === '/' ? `/${file.name}` : `${currentPath}/${file.name}`;
		try {
			if (isTauri && invoke) {
				await safeInvoke('delete_file', { serial: selectedDevice, path: pathToDelete });
			}
			infoMessage = `Deleted: ${file.name}`;
			selectedFile = null;
			await navigateTo(currentPath);
		} catch (e) {
			error = `Failed to delete: ${e}`;
		}
	}

	async function handlePushFile() {
		if (!localPushPath.trim() || !selectedDevice) return;
		error = '';
		infoMessage = '';
		const filename = localPushPath.split('/').pop() || 'file';
		const remotePath = currentPath === '/' ? `/${filename}` : `${currentPath}/${filename}`;
		try {
			if (isTauri && invoke) {
				await safeInvoke('push_file', { serial: selectedDevice, local: localPushPath.trim(), remote: remotePath });
			}
			infoMessage = `Uploaded ${filename} to ${currentPath}`;
			showPushModal = false;
			localPushPath = '';
			await navigateTo(currentPath);
		} catch (e) {
			error = `Push failed: ${e}`;
		}
	}

	async function handlePullFile() {
		if (!selectedFile || !localPullPath.trim() || !selectedDevice) return;
		error = '';
		infoMessage = '';
		const remotePath = currentPath === '/' ? `/${selectedFile.name}` : `${currentPath}/${selectedFile.name}`;
		try {
			if (isTauri && invoke) {
				await safeInvoke('pull_file', { serial: selectedDevice, remote: remotePath, local: localPullPath.trim() });
			}
			infoMessage = `Downloaded ${selectedFile.name} to ${localPullPath}`;
			showPullModal = false;
			localPullPath = '';
		} catch (e) {
			error = `Pull failed: ${e}`;
		}
	}

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
	}

	function formatTimestamp(time: number): string {
		if (!time) return '—';
		const date = new Date(time * 1000);
		return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
	}

	function getFileIcon(name: string, is_dir: boolean): string {
		if (is_dir) return 'folder';
		const ext = name.split('.').pop()?.toLowerCase();
		switch (ext) {
			case 'apk': return 'android';
			case 'zip': case 'tar': case 'gz': case '7z': return 'archive';
			case 'img': case 'iso': case 'bin': return 'disc_full';
			case 'jpg': case 'jpeg': case 'png': case 'webp': case 'gif': return 'image';
			case 'mp4': case 'mkv': case 'avi': case 'webm': return 'movie';
			case 'mp3': case 'flac': case 'wav': case 'ogg': return 'audio_file';
			case 'txt': case 'log': case 'prop': case 'json': case 'xml': return 'description';
			case 'sh': case 'py': case 'js': case 'ts': return 'code';
			default: return 'insert_drive_file';
		}
	}

	// Filtered & sorted files list
	let filteredFiles = $derived.by(() => {
		let list = files.filter((f) => f.name.toLowerCase().includes(searchQuery.toLowerCase()));
		list.sort((a, b) => {
			if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1; // Folders first
			let mult = sortAsc ? 1 : -1;
			if (sortBy === 'name') return a.name.localeCompare(b.name) * mult;
			if (sortBy === 'size') return (a.size - b.size) * mult;
			if (sortBy === 'modified') return (a.modified - b.modified) * mult;
			return 0;
		});
		return list;
	});

	// Split current path into breadcrumbs
	let breadcrumbs = $derived.by(() => {
		const parts = currentPath.split('/').filter(Boolean);
		let accum = '';
		return parts.map((part) => {
			accum += '/' + part;
			return { name: part, path: accum };
		});
	});
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2 h-screen overflow-hidden">
	<div class="flex flex-1 flex-col overflow-hidden rounded-[32px] bg-surface-container-low p-6 lg:p-8 relative gap-6 w-full">

		<!-- Top Header & Device Selector -->
		<header class="flex flex-col md:flex-row md:items-center justify-between gap-4 shrink-0 pb-2 border-b border-outline-variant/15 w-full">
			<div class="flex items-center gap-4">
				<button
					onclick={() => goto('/')}
					class="flex h-10 w-10 items-center justify-center rounded-full bg-surface-container hover:bg-surface-container-high text-on-surface-variant transition-all hover:scale-105 active:scale-95 shrink-0"
					title="Back to dashboard"
				>
					<span class="material-symbols-outlined text-[20px]">arrow_back</span>
				</button>
				<div>
					<div class="flex items-center gap-3">
						<h2 class="text-2xl font-bold tracking-tight text-on-surface">Device File Explorer</h2>
						{#if !isTauri}
							<span class="text-[10px] bg-warning/15 text-warning border border-warning/30 px-2.5 py-0.5 rounded-full font-bold uppercase tracking-wider">MOCK PREVIEW</span>
						{/if}
					</div>
					<p class="text-xs text-on-surface-variant/80 font-medium mt-0.5">Explore, transfer, and manage files on connected Android device</p>
				</div>
			</div>

			<!-- Right Control Group -->
			<div class="flex items-center gap-3 shrink-0">
				<div class="flex items-center gap-2 bg-surface-container px-3.5 py-1.5 rounded-full border border-outline-variant/20 shadow-sm">
					<span class="relative flex h-2.5 w-2.5">
						<span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
						<span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500"></span>
					</span>
					<select
						value={selectedDevice}
						onchange={(e) => { selectedDevice = e.currentTarget.value; navigateTo(currentPath); }}
						class="bg-transparent border-none text-xs font-semibold text-on-surface focus:outline-none cursor-pointer pr-2"
					>
						{#if devices.length === 0}
							<option value="">No Device Detected</option>
						{:else}
							{#each devices as dev (dev.id)}
								<option value={dev.id}>{dev.name}</option>
							{/each}
						{/if}
					</select>
				</div>

				<button
					onclick={loadDevices}
					class="flex h-9 w-9 items-center justify-center rounded-full bg-surface-container hover:bg-surface-container-high text-on-surface-variant transition-all hover:scale-105 active:scale-95 border border-outline-variant/20"
					title="Refresh Devices"
				>
					<span class="material-symbols-outlined text-[18px] {loading ? 'animate-spin' : ''}">refresh</span>
				</button>
			</div>
		</header>

		<!-- Alert Messages -->
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

		<!-- Main File Explorer Workspace Flex Layout -->
		<div class="flex-1 flex gap-6 overflow-hidden min-h-0 w-full">

			<!-- Left Quick Links Sidebar (Fixed Width w-60) -->
			<div class="w-60 shrink-0 flex flex-col gap-4 overflow-y-auto hidden md:flex">
				<div class="rounded-[24px] bg-surface-container p-4 border border-outline-variant/10 flex flex-col gap-2">
					<span class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80 px-2">Storage Shortcuts</span>
					
					<div class="flex flex-col gap-1">
						{#each quickLocations as loc}
							<button
								onclick={() => navigateTo(loc.path)}
								class="flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-semibold transition-all text-left {currentPath === loc.path ? 'bg-primary text-on-primary font-bold shadow-sm' : 'text-on-surface-variant hover:bg-surface-container-high hover:text-on-surface'}"
							>
								<span class="material-symbols-outlined text-[18px]">{loc.icon}</span>
								<span class="truncate">{loc.label}</span>
							</button>
						{/each}
					</div>
				</div>
			</div>

			<!-- Middle Main File Browser Area (Takes 100% Remaining Width: flex-1 min-w-0) -->
			<div class="flex-1 min-w-0 flex flex-col gap-4 overflow-hidden min-h-0">
				
				<!-- Action Toolbar & Breadcrumb Navigation Bar -->
				<div class="flex flex-col gap-3 shrink-0 w-full">
					
					<!-- Navigation Bar -->
					<div class="flex items-center gap-2 bg-surface-container p-2 rounded-2xl border border-outline-variant/10 w-full">
						<button
							onclick={navigateUp}
							disabled={currentPath === '/'}
							class="flex h-8 w-8 items-center justify-center rounded-xl bg-surface-container-high text-on-surface hover:bg-surface-container-highest disabled:opacity-30 transition-all shrink-0"
							title="Go Up One Directory"
						>
							<span class="material-symbols-outlined text-[18px]">arrow_upward</span>
						</button>

						<!-- Breadcrumb Trail -->
						<div class="flex-1 flex items-center gap-1 overflow-x-auto text-xs font-mono px-2 min-w-0">
							<button
								onclick={() => navigateTo('/')}
								class="hover:text-primary font-bold transition-colors shrink-0"
							>
								/
							</button>
							{#each breadcrumbs as crumb, idx}
								<span class="text-on-surface-variant/40 shrink-0">/</span>
								<button
									onclick={() => navigateTo(crumb.path)}
									class="hover:text-primary transition-colors whitespace-nowrap shrink-0 {idx === breadcrumbs.length - 1 ? 'text-primary font-bold' : 'text-on-surface-variant'}"
								>
									{crumb.name}
								</button>
							{/each}
						</div>

						<button
							onclick={() => navigateTo(currentPath)}
							class="flex h-8 w-8 items-center justify-center rounded-xl bg-surface-container-high text-on-surface hover:bg-surface-container-highest transition-all shrink-0"
							title="Refresh Current Folder"
						>
							<span class="material-symbols-outlined text-[18px] {loadingFiles ? 'animate-spin' : ''}">refresh</span>
						</button>
					</div>

					<!-- Toolbar Actions & Search Bar -->
					<div class="flex flex-wrap items-center justify-between gap-3 w-full">
						<div class="flex items-center gap-2">
							<button
								onclick={() => (showNewFolderModal = true)}
								class="flex items-center gap-1.5 px-3.5 py-2 rounded-xl bg-primary text-on-primary hover:brightness-110 text-xs font-bold transition-all shadow-sm shrink-0"
							>
								<span class="material-symbols-outlined text-[16px]">create_new_folder</span>
								New Folder
							</button>

							<button
								onclick={() => (showPushModal = true)}
								class="flex items-center gap-1.5 px-3.5 py-2 rounded-xl bg-surface-container-high text-on-surface hover:bg-surface-container-highest text-xs font-bold transition-all border border-outline-variant/20 shrink-0"
							>
								<span class="material-symbols-outlined text-[16px]">upload</span>
								Upload File
							</button>
						</div>

						<!-- Filter & View Controls -->
						<div class="flex items-center gap-2">
							<div class="relative">
								<span class="material-symbols-outlined absolute left-2.5 top-1/2 -translate-y-1/2 text-on-surface-variant text-[16px]">search</span>
								<input
									type="text"
									bind:value={searchQuery}
									placeholder="Search files..."
									class="bg-surface-container-high border border-outline-variant/30 rounded-xl pl-8 pr-3 py-1.5 text-xs text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 w-44"
								/>
							</div>

							<!-- Grid/List toggle -->
							<div class="flex bg-surface-container p-1 rounded-xl border border-outline-variant/10 shrink-0">
								<button
									onclick={() => (viewMode = 'list')}
									class="p-1 rounded-lg text-xs transition-all {viewMode === 'list' ? 'bg-primary text-on-primary' : 'text-on-surface-variant hover:text-on-surface'}"
									title="List View"
								>
									<span class="material-symbols-outlined text-[16px]">view_list</span>
								</button>
								<button
									onclick={() => (viewMode = 'grid')}
									class="p-1 rounded-lg text-xs transition-all {viewMode === 'grid' ? 'bg-primary text-on-primary' : 'text-on-surface-variant hover:text-on-surface'}"
									title="Grid View"
								>
									<span class="material-symbols-outlined text-[16px]">grid_view</span>
								</button>
							</div>
						</div>
					</div>

				</div>

				<!-- Files Display Container (Expands Full Width) -->
				<div class="flex-1 bg-surface-container rounded-[24px] p-5 border border-outline-variant/10 overflow-y-auto min-h-0 w-full">
					{#if loadingFiles}
						<div class="flex flex-col items-center justify-center h-48 gap-3 text-on-surface-variant">
							<span class="animate-spin h-6 w-6 border-2 border-primary border-t-transparent rounded-full"></span>
							<span class="text-xs font-semibold">Loading directory items...</span>
						</div>
					{:else if filteredFiles.length === 0}
						<div class="flex flex-col items-center justify-center h-48 gap-2 text-on-surface-variant/70 text-center">
							<span class="material-symbols-outlined text-[36px]">folder_open</span>
							<span class="text-xs font-bold">This directory is empty</span>
						</div>
					{:else if viewMode === 'list'}
						<!-- Full Width Table List View -->
						<table class="w-full text-left border-collapse text-xs">
							<thead>
								<tr class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider border-b border-outline-variant/15 pb-2">
									<th class="pb-2.5 font-bold cursor-pointer" onclick={() => { sortBy = 'name'; sortAsc = !sortAsc; }}>Name</th>
									<th class="pb-2.5 font-bold cursor-pointer text-right w-28" onclick={() => { sortBy = 'size'; sortAsc = !sortAsc; }}>Size</th>
									<th class="pb-2.5 font-bold cursor-pointer text-center w-28 hidden lg:table-cell">Permissions</th>
									<th class="pb-2.5 font-bold cursor-pointer text-right w-40 hidden sm:table-cell" onclick={() => { sortBy = 'modified'; sortAsc = !sortAsc; }}>Modified</th>
								</tr>
							</thead>
							<tbody class="divide-y divide-outline-variant/10">
								{#each filteredFiles as file (file.name)}
									<tr
										onclick={() => handleItemClick(file)}
										class="hover:bg-surface-container-high/60 transition-all cursor-pointer group {selectedFile?.name === file.name ? 'bg-primary/15 font-bold' : ''}"
									>
										<td class="py-2.5 pr-3">
											<div class="flex items-center gap-3">
												<span class="material-symbols-outlined text-[22px] shrink-0 {file.is_dir ? 'text-amber-400 font-fill' : 'text-primary/80'}">
													{getFileIcon(file.name, file.is_dir)}
												</span>
												<span class="font-semibold text-on-surface group-hover:text-primary transition-colors truncate">{file.name}</span>
											</div>
										</td>
										<td class="py-2.5 text-on-surface-variant text-right font-mono text-[11px]">
											{file.is_dir ? '—' : formatBytes(file.size)}
										</td>
										<td class="py-2.5 text-on-surface-variant/70 text-center font-mono text-[10px] hidden lg:table-cell">
											{file.permissions}
										</td>
										<td class="py-2.5 text-on-surface-variant text-right font-mono text-[11px] hidden sm:table-cell">
											{formatTimestamp(file.modified)}
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					{:else}
						<!-- Grid View -->
						<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3">
							{#each filteredFiles as file (file.name)}
								<button
									onclick={() => handleItemClick(file)}
									class="flex flex-col items-center gap-2 p-3.5 rounded-2xl bg-surface-container-high/50 hover:bg-surface-container-highest transition-all border border-outline-variant/10 group text-center {selectedFile?.name === file.name ? 'ring-2 ring-primary bg-primary/10' : ''}"
								>
									<span class="material-symbols-outlined text-[36px] {file.is_dir ? 'text-amber-400' : 'text-primary/80'}">
										{getFileIcon(file.name, file.is_dir)}
									</span>
									<span class="text-xs font-semibold text-on-surface truncate w-full group-hover:text-primary transition-colors">{file.name}</span>
									<span class="text-[10px] font-mono text-on-surface-variant/70">{file.is_dir ? 'Folder' : formatBytes(file.size)}</span>
								</button>
							{/each}
						</div>
					{/if}
				</div>
			</div>

			<!-- Right File Details Panel (Fixed Width w-72 when selected) -->
			{#if selectedFile}
				<div class="w-72 shrink-0 rounded-[24px] bg-surface-container p-5 border border-outline-variant/10 flex flex-col justify-between gap-4 animate-fade-in">
					<div class="flex flex-col gap-4">
						<div class="flex items-center justify-between">
							<h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">File Properties</h3>
							<button onclick={() => (selectedFile = null)} class="text-on-surface-variant hover:text-on-surface">
								<span class="material-symbols-outlined text-[18px]">close</span>
							</button>
						</div>

						<div class="flex flex-col items-center p-4 rounded-2xl bg-surface-container-high/50 gap-2">
							<span class="material-symbols-outlined text-[48px] {selectedFile.is_dir ? 'text-amber-400' : 'text-primary'}">
								{getFileIcon(selectedFile.name, selectedFile.is_dir)}
							</span>
							<span class="text-xs font-bold text-on-surface text-center break-all">{selectedFile.name}</span>
						</div>

						<div class="flex flex-col gap-2 text-xs">
							<div class="flex justify-between py-1 border-b border-outline-variant/10">
								<span class="text-on-surface-variant">Type</span>
								<span class="font-bold text-on-surface">{selectedFile.is_dir ? 'Directory' : 'File'}</span>
							</div>
							<div class="flex justify-between py-1 border-b border-outline-variant/10">
								<span class="text-on-surface-variant">Size</span>
								<span class="font-mono text-on-surface">{formatBytes(selectedFile.size)}</span>
							</div>
							<div class="flex justify-between py-1 border-b border-outline-variant/10">
								<span class="text-on-surface-variant">Permissions</span>
								<span class="font-mono text-on-surface">{selectedFile.permissions}</span>
							</div>
							<div class="flex justify-between py-1 border-b border-outline-variant/10">
								<span class="text-on-surface-variant">Modified</span>
								<span class="font-mono text-on-surface text-[10px]">{formatTimestamp(selectedFile.modified)}</span>
							</div>
						</div>
					</div>

					<div class="flex flex-col gap-2">
						{#if !selectedFile.is_dir}
							<button
								onclick={() => (showPullModal = true)}
								class="flex items-center justify-center gap-2 py-2 px-3 rounded-xl bg-primary text-on-primary hover:brightness-110 text-xs font-bold transition-all shadow-sm"
							>
								<span class="material-symbols-outlined text-[16px]">download</span>
								Download to PC
							</button>
						{/if}

						<button
							onclick={() => { renameNewName = selectedFile?.name || ''; showRenameModal = true; }}
							class="flex items-center justify-center gap-2 py-2 px-3 rounded-xl bg-surface-container-high hover:bg-surface-container-highest text-on-surface text-xs font-bold transition-all border border-outline-variant/20"
						>
							<span class="material-symbols-outlined text-[16px]">edit</span>
							Rename
						</button>

						<button
							onclick={() => selectedFile && handleDelete(selectedFile)}
							class="flex items-center justify-center gap-2 py-2 px-3 rounded-xl bg-error/15 text-error hover:bg-error/25 border border-error/30 text-xs font-bold transition-all"
						>
							<span class="material-symbols-outlined text-[16px]">delete</span>
							Delete
						</button>
					</div>
				</div>
			{/if}

		</div>
	</div>
</main>

<!-- Action Modals -->

<!-- New Folder Modal -->
{#if showNewFolderModal}
	<div class="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
		<div class="bg-surface-container p-6 rounded-[28px] max-w-sm w-full flex flex-col gap-4 border border-outline-variant/20 shadow-2xl animate-fade-in">
			<h3 class="text-base font-bold text-on-surface">Create New Directory</h3>
			<input
				type="text"
				bind:value={newFolderName}
				placeholder="Folder name"
				class="bg-surface-container-high border border-outline-variant/30 rounded-xl px-3 py-2 text-xs font-semibold text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40"
			/>
			<div class="flex gap-2 justify-end">
				<button onclick={() => (showNewFolderModal = false)} class="px-4 py-2 rounded-xl text-xs font-bold text-on-surface-variant hover:bg-surface-container-high">Cancel</button>
				<button onclick={handleCreateFolder} class="px-4 py-2 rounded-xl text-xs font-bold bg-primary text-on-primary hover:brightness-110">Create Folder</button>
			</div>
		</div>
	</div>
{/if}

<!-- Rename Modal -->
{#if showRenameModal}
	<div class="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
		<div class="bg-surface-container p-6 rounded-[28px] max-w-sm w-full flex flex-col gap-4 border border-outline-variant/20 shadow-2xl animate-fade-in">
			<h3 class="text-base font-bold text-on-surface">Rename Item</h3>
			<input
				type="text"
				bind:value={renameNewName}
				placeholder="New name"
				class="bg-surface-container-high border border-outline-variant/30 rounded-xl px-3 py-2 text-xs font-semibold text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40"
			/>
			<div class="flex gap-2 justify-end">
				<button onclick={() => (showRenameModal = false)} class="px-4 py-2 rounded-xl text-xs font-bold text-on-surface-variant hover:bg-surface-container-high">Cancel</button>
				<button onclick={handleRename} class="px-4 py-2 rounded-xl text-xs font-bold bg-primary text-on-primary hover:brightness-110">Rename</button>
			</div>
		</div>
	</div>
{/if}

<!-- Upload File Modal -->
{#if showPushModal}
	<div class="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
		<div class="bg-surface-container p-6 rounded-[28px] max-w-md w-full flex flex-col gap-4 border border-outline-variant/20 shadow-2xl animate-fade-in">
			<h3 class="text-base font-bold text-on-surface">Upload File to Device</h3>
			<p class="text-xs text-on-surface-variant">Destination path: <code class="font-mono text-primary">{currentPath}</code></p>
			<input
				type="text"
				bind:value={localPushPath}
				placeholder="/path/to/local/file.txt"
				class="bg-surface-container-high border border-outline-variant/30 rounded-xl px-3 py-2 text-xs font-mono text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40"
			/>
			<div class="flex gap-2 justify-end">
				<button onclick={() => (showPushModal = false)} class="px-4 py-2 rounded-xl text-xs font-bold text-on-surface-variant hover:bg-surface-container-high">Cancel</button>
				<button onclick={handlePushFile} class="px-4 py-2 rounded-xl text-xs font-bold bg-primary text-on-primary hover:brightness-110">Upload File</button>
			</div>
		</div>
	</div>
{/if}

<!-- Download File Modal -->
{#if showPullModal}
	<div class="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
		<div class="bg-surface-container p-6 rounded-[28px] max-w-md w-full flex flex-col gap-4 border border-outline-variant/20 shadow-2xl animate-fade-in">
			<h3 class="text-base font-bold text-on-surface">Download File to PC</h3>
			<p class="text-xs text-on-surface-variant">Source item: <code class="font-mono text-primary">{selectedFile?.name}</code></p>
			<input
				type="text"
				bind:value={localPullPath}
				placeholder="/path/to/destination/file"
				class="bg-surface-container-high border border-outline-variant/30 rounded-xl px-3 py-2 text-xs font-mono text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40"
			/>
			<div class="flex gap-2 justify-end">
				<button onclick={() => (showPullModal = false)} class="px-4 py-2 rounded-xl text-xs font-bold text-on-surface-variant hover:bg-surface-container-high">Cancel</button>
				<button onclick={handlePullFile} class="px-4 py-2 rounded-xl text-xs font-bold bg-primary text-on-primary hover:brightness-110">Download File</button>
			</div>
		</div>
	</div>
{/if}

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
