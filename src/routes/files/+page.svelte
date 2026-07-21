<script lang="ts">
	/* eslint-disable svelte/no-navigation-without-resolve */
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { SvelteSet } from 'svelte/reactivity';
	import ShapeBadge from '$lib/components/ShapeBadge.svelte';

	// Sub-components
	import FileSidebar from './components/FileSidebar.svelte';
	import FileBreadcrumb from './components/FileBreadcrumb.svelte';
	import FileToolbar, { type CategoryFilter } from './components/FileToolbar.svelte';
	import FileListItem from './components/FileListItem.svelte';
	import FileGridItem from './components/FileGridItem.svelte';
	import FileDetailsPanel from './components/FileDetailsPanel.svelte';
	import FileContextMenu from './components/FileContextMenu.svelte';
	import FileModals from './components/FileModals.svelte';

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

	// Devices & Global State
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

	// Category Filter
	let selectedCategory = $state<CategoryFilter>('all');

	// Multi-select state (keyboard/click selection)
	let selectedFileNames = new SvelteSet<string>();

	// Sorting
	let sortBy = $state<'name' | 'size' | 'modified'>('name');
	let sortAsc = $state(true);

	// Drag & Drop Upload State
	let isDraggingOverWorkspace = $state(false);
	let dragTargetFolder = $state<string | null>(null);

	// Context Menu State
	let contextMenu = $state<{
		show: boolean;
		x: number;
		y: number;
		file: FileEntry | null;
	}>({
		show: false,
		x: 0,
		y: 0,
		file: null
	});

	// Action Modals State
	let showNewFolderModal = $state(false);
	let newFolderName = $state('');
	let showRenameModal = $state(false);
	let renameNewName = $state('');
	let showPushModal = $state(false);
	let localPushPath = $state('');
	let showPullModal = $state(false);
	let localPullPath = $state('');

	// Custom Confirmation Modal state
	let confirmModalState = $state<{
		show: boolean;
		title: string;
		message: string;
		confirmText: string;
		icon: string;
		isDanger: boolean;
		action: () => void;
	}>({
		show: false,
		title: '',
		message: '',
		confirmText: 'Confirm',
		icon: 'warning',
		isDanger: true,
		action: () => {}
	});

	let baseDownloadDir = $state('~/Downloads/Oxide');

	let unlistenDrop: (() => void) | undefined;

	onMount(async () => {
		try {
			if (typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window)) {
				const tauriApi = await import('@tauri-apps/api/core');
				invoke = tauriApi.invoke;
				isTauri = true;

				try {
					const webviewModule = await import('@tauri-apps/api/webviewWindow');
					const appWindow = webviewModule.getCurrentWebviewWindow();
					unlistenDrop = await appWindow.onDragDropEvent((event) => {
						if (event.payload.type === 'over' || event.payload.type === 'enter') {
							isDraggingOverWorkspace = true;
						} else if (event.payload.type === 'drop') {
							isDraggingOverWorkspace = false;
							const paths = event.payload.paths;
							if (paths && paths.length > 0) {
								processDroppedPaths(paths, dragTargetFolder);
							}
							dragTargetFolder = null;
						} else if (event.payload.type === 'leave' || event.payload.type === 'cancel') {
							isDraggingOverWorkspace = false;
							dragTargetFolder = null;
						}
					});
				} catch (err) {
					console.warn('Tauri onDragDropEvent listener skipped:', err);
				}
			} else {
				isTauri = false;
			}
		} catch {
			isTauri = false;
		}

		if (isTauri && invoke) {
			try {
				baseDownloadDir = await safeInvoke<string>('get_default_download_dir');
			} catch {
				baseDownloadDir = '~/Downloads/Oxide';
			}
		}

		const closeContextMenu = () => {
			if (contextMenu.show) contextMenu.show = false;
		};
		window.addEventListener('click', closeContextMenu);

		await loadDevices();

		return () => {
			window.removeEventListener('click', closeContextMenu);
			if (unlistenDrop) unlistenDrop();
		};
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
		selectedFileNames.clear();

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

	function handleItemClick(file: FileEntry, e?: MouseEvent) {
		if (e?.ctrlKey || e?.metaKey) {
			toggleSelectFile(file.name);
			return;
		}
		selectedFile = file;
	}

	function handleItemDblClick(file: FileEntry) {
		if (file.is_dir) {
			const newPath = currentPath === '/' ? `/${file.name}` : `${currentPath}/${file.name}`;
			navigateTo(newPath);
		}
	}

	function handleContextMenu(e: MouseEvent, file: FileEntry) {
		e.preventDefault();
		e.stopPropagation();
		selectedFile = file;
		contextMenu = {
			show: true,
			x: Math.min(e.clientX, window.innerWidth - 220),
			y: Math.min(e.clientY, window.innerHeight - 240),
			file
		};
	}

	function toggleSelectFile(name: string) {
		if (selectedFileNames.has(name)) {
			selectedFileNames.delete(name);
		} else {
			selectedFileNames.add(name);
		}
	}

	function navigateUp() {
		if (currentPath === '/' || currentPath === '') return;
		const parts = currentPath.split('/').filter(Boolean);
		parts.pop();
		const parentPath = '/' + parts.join('/');
		navigateTo(parentPath);
	}

	async function copyPathToClipboard(file: FileEntry) {
		const fullPath = currentPath === '/' ? `/${file.name}` : `${currentPath}/${file.name}`;
		try {
			await navigator.clipboard.writeText(fullPath);
			infoMessage = `Copied "${fullPath}" to clipboard!`;
		} catch {
			error = 'Failed to copy path to clipboard.';
		}
	}

	// Drag & Drop Upload Handlers
	function handleDragOver(e: DragEvent, folderName?: string) {
		e.preventDefault();
		e.stopPropagation();
		isDraggingOverWorkspace = true;
		dragTargetFolder = folderName || null;
	}

	function handleDragLeave(e: DragEvent) {
		e.preventDefault();
		e.stopPropagation();
		isDraggingOverWorkspace = false;
		dragTargetFolder = null;
	}

	async function processDroppedPaths(paths: string[], folderName?: string | null) {
		if (!selectedDevice || paths.length === 0) return;

		const destDir = folderName
			? (currentPath === '/' ? `/${folderName}` : `${currentPath}/${folderName}`)
			: currentPath;

		loadingFiles = true;
		error = '';
		infoMessage = '';

		let successCount = 0;
		let failCount = 0;

		for (const localPath of paths) {
			const name = localPath.split('/').pop()?.split('\\').pop() || 'item';
			const remoteDest = destDir === '/' ? `/${name}` : `${destDir}/${name}`;

			try {
				if (isTauri && invoke) {
					await safeInvoke('push_file', {
						serial: selectedDevice,
						local: localPath,
						remote: remoteDest
					});
					successCount++;
				} else {
					successCount++;
				}
			} catch (e) {
				failCount++;
				console.error('Push failed for path:', localPath, e);
			}
		}

		infoMessage = `Uploaded ${successCount} item(s) to ${destDir}${failCount > 0 ? ` (${failCount} failed)` : ''}`;
		await navigateTo(currentPath);
	}

	async function handleDrop(e: DragEvent, folderName?: string) {
		e.preventDefault();
		e.stopPropagation();
		isDraggingOverWorkspace = false;

		const dtFiles = e.dataTransfer?.files;
		if (!dtFiles || dtFiles.length === 0) return;

		const paths: string[] = [];
		for (let i = 0; i < dtFiles.length; i++) {
			const file = dtFiles[i];
			const p = (file as any).path;
			if (p) {
				paths.push(p);
			}
		}

		if (paths.length > 0) {
			await processDroppedPaths(paths, folderName || dragTargetFolder);
		} else {
			// Mock mode or browser fallback
			infoMessage = `Dropped ${dtFiles.length} file(s) into ${folderName || currentPath} (Mock Mode)`;
		}
		dragTargetFolder = null;
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

	function openRenameModal(file?: FileEntry) {
		const target = file || selectedFile;
		if (!target) return;
		selectedFile = target;
		renameNewName = target.name;
		showRenameModal = true;
	}

	function getCategorySubfolder(filename: string, is_dir: boolean): string {
		if (is_dir) return '';
		const ext = filename.split('.').pop()?.toLowerCase() || '';
		if (['png', 'jpg', 'jpeg', 'webp', 'gif', 'bmp', 'svg', 'heic', 'raw'].includes(ext)) return 'Photos';
		if (['mp4', 'mkv', 'avi', 'mov', 'webm', 'flv', '3gp', 'm4v'].includes(ext)) return 'Videos';
		if (['mp3', 'flac', 'wav', 'aac', 'm4a', 'ogg', 'opus', 'wma'].includes(ext)) return 'Music';
		if (['pdf', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'txt', 'zip', 'rar', '7z', 'gz', 'tar', 'apk', 'json', 'xml', 'md'].includes(ext)) return 'Documents';
		return 'Others';
	}

	function openPullModal(file?: FileEntry) {
		const target = file || selectedFile;
		if (!target) return;
		selectedFile = target;

		const base = baseDownloadDir || '~/Downloads/Oxide';
		if (target.is_dir) {
			localPullPath = `${base}/${target.name}`;
		} else {
			const subfolder = getCategorySubfolder(target.name, false);
			localPullPath = subfolder ? `${base}/${subfolder}/${target.name}` : `${base}/${target.name}`;
		}
		showPullModal = true;
	}

	function requestDelete(file?: FileEntry) {
		const target = file || selectedFile;
		if (!target) return;
		selectedFile = target;
		confirmModalState = {
			show: true,
			title: `Delete ${target.is_dir ? 'Folder' : 'File'}`,
			message: `Are you sure you want to delete "${target.name}"? ${target.is_dir ? 'All files inside this folder will be permanently wiped.' : 'This action cannot be undone.'}`,
			confirmText: `Delete ${target.is_dir ? 'Folder' : 'File'}`,
			icon: 'delete_forever',
			isDanger: true,
			action: () => executeDelete(target)
		};
	}

	function requestBatchDelete() {
		const count = selectedFileNames.size;
		if (count === 0) return;

		confirmModalState = {
			show: true,
			title: `Delete ${count} Items`,
			message: `Are you sure you want to delete ${count} selected items permanently from ${currentPath}?`,
			confirmText: `Delete ${count} Items`,
			icon: 'delete_sweep',
			isDanger: true,
			action: () => executeBatchDelete()
		};
	}

	async function executeDelete(file: FileEntry) {
		if (!selectedDevice) return;
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

	async function executeBatchDelete() {
		if (!selectedDevice) return;
		loadingFiles = true;
		error = '';
		infoMessage = '';
		let failCount = 0;

		for (const name of selectedFileNames) {
			const pathToDelete = currentPath === '/' ? `/${name}` : `${currentPath}/${name}`;
			try {
				if (isTauri && invoke) {
					await safeInvoke('delete_file', { serial: selectedDevice, path: pathToDelete });
				}
			} catch {
				failCount++;
			}
		}

		infoMessage = `Batch delete completed${failCount > 0 ? ` with ${failCount} errors` : ' successfully'}.`;
		selectedFileNames.clear();
		await navigateTo(currentPath);
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
			infoMessage = `Downloaded ${selectedFile.name} (${selectedFile.is_dir ? 'folder' : 'file'}) to ${localPullPath.trim()}`;
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
			case 'zip': case 'tar': case 'gz': case '7z': case 'rar': return 'archive';
			case 'img': case 'iso': case 'bin': return 'disc_full';
			case 'jpg': case 'jpeg': case 'png': case 'webp': case 'gif': case 'svg': return 'image';
			case 'mp4': case 'mkv': case 'avi': case 'webm': return 'movie';
			case 'mp3': case 'flac': case 'wav': case 'ogg': return 'audio_file';
			case 'txt': case 'log': case 'prop': case 'json': case 'xml': case 'pdf': return 'description';
			case 'sh': case 'py': case 'js': case 'ts': return 'code';
			default: return 'insert_drive_file';
		}
	}

	// Filtered & sorted files list with category filter
	let filteredFiles = $derived.by(() => {
		let list = files.filter((f) => {
			const matchesSearch = f.name.toLowerCase().includes(searchQuery.toLowerCase());
			if (!matchesSearch) return false;

			if (selectedCategory === 'folders') return f.is_dir;
			if (selectedCategory === 'all') return true;

			const ext = f.name.split('.').pop()?.toLowerCase() || '';
			if (selectedCategory === 'media') return ['jpg', 'jpeg', 'png', 'webp', 'gif', 'mp4', 'mkv', 'avi', 'webm'].includes(ext);
			if (selectedCategory === 'documents') return ['pdf', 'doc', 'docx', 'txt', 'log', 'prop', 'json', 'xml'].includes(ext);
			if (selectedCategory === 'archives') return ['zip', 'tar', 'gz', '7z', 'rar', 'iso', 'img'].includes(ext);
			if (selectedCategory === 'apks') return ext === 'apk';

			return true;
		});

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

	// Total directory size sum
	let totalDirSize = $derived.by(() => {
		return files.reduce((acc, curr) => acc + (curr.is_dir ? 0 : curr.size), 0);
	});
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2 h-screen overflow-hidden select-none">
	<div class="flex flex-1 flex-col overflow-hidden rounded-[32px] bg-surface-container-low p-6 lg:p-8 relative gap-5 w-full shadow-sm">

		<!-- Top Header & Device Selector -->
		<header class="flex flex-col md:flex-row md:items-center justify-between gap-4 shrink-0 pb-1 w-full">
			<div class="flex items-center gap-4">
				<button
					onclick={() => goto('/')}
					class="flex h-10 w-10 items-center justify-center rounded-full bg-surface-container hover:bg-surface-container-high text-on-surface-variant transition-all hover:scale-105 active:scale-95 shrink-0 border-0 outline-none cursor-pointer"
					title="Back to dashboard"
				>
					<span class="material-symbols-outlined text-[20px]">arrow_back</span>
				</button>
				<div>
					<div class="flex items-center gap-3">
						<ShapeBadge icon="folder_open" shape="clover" size={40} iconSize={20} />
						<h2 class="text-2xl font-bold tracking-tight text-on-surface">Device File Manager</h2>
						{#if !isTauri}
							<span class="text-[10px] bg-warning/15 text-warning px-2.5 py-0.5 rounded-full font-bold uppercase tracking-wider">MOCK PREVIEW</span>
						{/if}
					</div>
					<p class="text-xs text-on-surface-variant/80 font-medium mt-0.5">Drag & drop files to upload, manage remote storage, & batch transfer</p>
				</div>
			</div>
		</header>

		<!-- Alert Messages -->
		{#if error}
			<div class="bg-error/15 text-error p-3.5 rounded-2xl font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[20px]">error</span>
				<div class="flex-1 break-words font-semibold">{error}</div>
				<button onclick={() => (error = '')} class="hover:opacity-80 text-[10px] font-bold uppercase tracking-wider bg-error/20 px-2.5 py-1 rounded-lg border-0 outline-none cursor-pointer">Dismiss</button>
			</div>
		{/if}

		{#if infoMessage}
			<div class="bg-primary/10 text-primary p-3.5 rounded-2xl font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[20px]">check_circle</span>
				<div class="flex-1 break-words font-semibold">{infoMessage}</div>
				<button onclick={() => (infoMessage = '')} class="hover:opacity-80 text-[10px] font-bold uppercase tracking-wider bg-primary/20 px-2.5 py-1 rounded-lg border-0 outline-none cursor-pointer">Dismiss</button>
			</div>
		{/if}

		<!-- Main File Explorer Workspace Flex Layout -->
		<div class="flex-1 flex gap-5 overflow-hidden min-h-0 w-full">

			<!-- Left Quick Links Sidebar -->
			<FileSidebar
				{currentPath}
				filesCount={files.length}
				{totalDirSize}
				onnavigate={navigateTo}
				{formatBytes}
			/>

			<!-- Middle Main File Browser Area -->
			<div class="flex-1 min-w-0 flex flex-col gap-4 overflow-hidden min-h-0">
				
				<!-- Action Toolbar & Breadcrumb Navigation Bar -->
				<div class="flex flex-col gap-3 shrink-0 w-full">
					<FileBreadcrumb
						{currentPath}
						{loadingFiles}
						onnavigate={navigateTo}
						onnavigateup={navigateUp}
					/>

					<FileToolbar
						{selectedFile}
						selectedCount={selectedFileNames.size}
						{searchQuery}
						{viewMode}
						{selectedCategory}
						onnewfolder={() => (showNewFolderModal = true)}
						onupload={() => (showPushModal = true)}
						onpull={() => selectedFile && openPullModal(selectedFile)}
						oncopypath={() => selectedFile && copyPathToClipboard(selectedFile)}
						onrename={() => selectedFile && openRenameModal(selectedFile)}
						ondelete={() => selectedFile && requestDelete(selectedFile)}
						onbatchdelete={requestBatchDelete}
						onclearselection={() => selectedFileNames.clear()}
						onsearchchange={(q) => (searchQuery = q)}
						onviewmodechange={(m) => (viewMode = m)}
						oncategorychange={(c) => (selectedCategory = c)}
					/>
				</div>

				<!-- Files Display Container (Drag & Drop Zone + Clean Borderless Rows) -->
				<div
					ondragover={(e) => handleDragOver(e)}
					ondragleave={(e) => handleDragLeave(e)}
					ondrop={(e) => handleDrop(e)}
					class="flex-1 bg-surface-container rounded-[28px] p-5 overflow-y-auto min-h-0 w-full shadow-sm relative transition-all duration-200 {isDraggingOverWorkspace && !dragTargetFolder ? 'ring-4 ring-primary/60 bg-primary/10' : ''}"
				>
					<!-- Drag & Drop Overlay Indicator -->
					{#if isDraggingOverWorkspace}
						<div class="absolute inset-0 z-30 bg-black/60 backdrop-blur-md rounded-[28px] flex flex-col items-center justify-center p-6 text-white border-2 border-dashed border-primary animate-fade-in pointer-events-none">
							<ShapeBadge icon="cloud_upload" shape="sunny" size={64} iconSize={32} bgClass="bg-primary/30" textClass="text-primary" />
							<h3 class="text-lg font-bold mt-4">Drop Files to Upload</h3>
							<p class="text-xs text-white/80 mt-1">Uploading to: <code class="font-mono text-primary font-bold">{dragTargetFolder ? `${currentPath}/${dragTargetFolder}` : currentPath}</code></p>
						</div>
					{/if}

					{#if loadingFiles}
						<div class="flex flex-col items-center justify-center h-48 gap-3 text-on-surface-variant">
							<span class="animate-spin h-6 w-6 border-2 border-primary border-t-transparent rounded-full"></span>
							<span class="text-xs font-semibold">Loading directory items...</span>
						</div>
					{:else if filteredFiles.length === 0}
						<div class="flex flex-col items-center justify-center h-48 gap-2 text-on-surface-variant/70 text-center">
							<span class="material-symbols-outlined text-[36px]">folder_open</span>
							<span class="text-xs font-bold">This directory is empty</span>
							<p class="text-[11px] text-on-surface-variant/60">Drag and drop files here to upload instantly</p>
						</div>
					{:else if viewMode === 'list'}
						<!-- Borderless Clean List View (No Ugly Checkboxes) -->
						<table class="w-full text-left border-collapse text-xs">
							<thead>
								<tr class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider pb-3">
									<th class="pb-3 px-4 font-bold cursor-pointer" onclick={() => { sortBy = 'name'; sortAsc = !sortAsc; }}>Name</th>
									<th class="pb-3 px-4 font-bold cursor-pointer text-right w-28" onclick={() => { sortBy = 'size'; sortAsc = !sortAsc; }}>Size</th>
									<th class="pb-3 px-4 font-bold cursor-pointer text-center w-28 hidden lg:table-cell">Permissions</th>
									<th class="pb-3 px-4 font-bold cursor-pointer text-right w-40 hidden sm:table-cell" onclick={() => { sortBy = 'modified'; sortAsc = !sortAsc; }}>Modified</th>
								</tr>
							</thead>
							<tbody class="space-y-1">
								{#each filteredFiles as file (file.name)}
									<FileListItem
										{file}
										isSelected={selectedFile?.name === file.name || selectedFileNames.has(file.name)}
										isDragTarget={dragTargetFolder === file.name}
										{getFileIcon}
										{formatBytes}
										{formatTimestamp}
										onclick={(e) => handleItemClick(file, e)}
										ondblclick={() => handleItemDblClick(file)}
										oncontextmenu={(e) => handleContextMenu(e, file)}
										ondragover={(e) => handleDragOver(e, file.name)}
										ondragleave={(e) => handleDragLeave(e)}
										ondrop={(e) => handleDrop(e, file.name)}
									/>
								{/each}
							</tbody>
						</table>
					{:else}
						<!-- Grid View -->
						<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3">
							{#each filteredFiles as file (file.name)}
								<FileGridItem
									{file}
									isSelected={selectedFile?.name === file.name || selectedFileNames.has(file.name)}
									isDragTarget={dragTargetFolder === file.name}
									{getFileIcon}
									{formatBytes}
									onclick={(e) => handleItemClick(file, e)}
									ondblclick={() => handleItemDblClick(file)}
									oncontextmenu={(e) => handleContextMenu(e, file)}
									ondragover={(e) => handleDragOver(e, file.name)}
									ondragleave={(e) => handleDragLeave(e)}
									ondrop={(e) => handleDrop(e, file.name)}
								/>
							{/each}
						</div>
					{/if}
				</div>
			</div>

			<!-- Right File Details Panel -->
			<FileDetailsPanel
				{selectedFile}
				{getFileIcon}
				{formatBytes}
				{formatTimestamp}
				onclose={() => (selectedFile = null)}
				onpull={() => selectedFile && openPullModal(selectedFile)}
				oncopypath={() => selectedFile && copyPathToClipboard(selectedFile)}
				onrename={() => selectedFile && openRenameModal(selectedFile)}
				ondelete={() => selectedFile && requestDelete(selectedFile)}
			/>

		</div>
	</div>
</main>

<!-- Floating Context Menu -->
<FileContextMenu
	show={contextMenu.show}
	x={contextMenu.x}
	y={contextMenu.y}
	file={contextMenu.file}
	{formatBytes}
	onopenfolder={handleItemDblClick}
	onpull={openPullModal}
	oncopypath={copyPathToClipboard}
	onrename={openRenameModal}
	ondelete={requestDelete}
/>

<!-- Action Modals (New Folder, Upload with Browse File Picker, Rename, Pull, Delete Confirm) -->
<FileModals
	{currentPath}
	{selectedFile}
	{showNewFolderModal}
	bind:newFolderName
	oncreatenewfolder={handleCreateFolder}
	oncancelnewfolder={() => (showNewFolderModal = false)}
	{showRenameModal}
	bind:renameNewName
	onrename={handleRename}
	oncancelrename={() => (showRenameModal = false)}
	{showPushModal}
	bind:localPushPath
	onpush={handlePushFile}
	oncancelpush={() => (showPushModal = false)}
	{showPullModal}
	bind:localPullPath
	onpull={handlePullFile}
	oncancelpull={() => (showPullModal = false)}
	{confirmModalState}
	oncancelconfirm={() => (confirmModalState.show = false)}
/>

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
