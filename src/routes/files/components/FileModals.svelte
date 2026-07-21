<script lang="ts">
	import ShapeBadge from '$lib/components/ShapeBadge.svelte';

	interface FileEntry {
		name: string;
		is_dir: boolean;
		size: number;
		permissions: string;
		modified: number;
	}

	interface ConfirmModalState {
		show: boolean;
		title: string;
		message: string;
		confirmText: string;
		icon: string;
		isDanger: boolean;
		action: () => void;
	}

	interface Props {
		currentPath: string;
		selectedFile: FileEntry | null;

		showNewFolderModal: boolean;
		newFolderName: string;
		oncreatenewfolder: () => void;
		oncancelnewfolder: () => void;

		showRenameModal: boolean;
		renameNewName: string;
		onrename: () => void;
		oncancelrename: () => void;

		showPushModal: boolean;
		localPushPath: string;
		onpush: () => void;
		oncancelpush: () => void;

		showPullModal: boolean;
		localPullPath: string;
		onpull: () => void;
		oncancelpull: () => void;

		confirmModalState: ConfirmModalState;
		oncancelconfirm: () => void;
	}

	let {
		currentPath,
		selectedFile,
		showNewFolderModal,
		newFolderName = $bindable(),
		oncreatenewfolder,
		oncancelnewfolder,
		showRenameModal,
		renameNewName = $bindable(),
		onrename,
		oncancelrename,
		showPushModal,
		localPushPath = $bindable(),
		onpush,
		oncancelpush,
		showPullModal,
		localPullPath = $bindable(),
		onpull,
		oncancelpull,
		confirmModalState,
		oncancelconfirm
	}: Props = $props();

	let fileInputRef: HTMLInputElement | undefined;

	function triggerBrowseFile() {
		fileInputRef?.click();
	}

	function handleFileSelected(e: Event) {
		const target = e.target as HTMLInputElement;
		if (target.files && target.files.length > 0) {
			const selected = target.files[0];
			const fullPath = (selected as any).path || selected.name;
			localPushPath = fullPath;
		}
	}
</script>

<!-- Hidden File Input for Native Browse File Picker -->
<input
	bind:this={fileInputRef}
	type="file"
	onchange={handleFileSelected}
	class="hidden"
/>

<!-- New Folder Modal -->
{#if showNewFolderModal}
	<div class="fixed inset-0 bg-black/60 backdrop-blur-md flex items-center justify-center z-50 p-4 animate-fade-in">
		<div class="bg-surface-container p-6 rounded-[32px] max-w-sm w-full flex flex-col gap-5 shadow-2xl border border-surface-container-high/60">
			<div class="flex items-center gap-3">
				<ShapeBadge icon="create_new_folder" shape="cookie7" size={40} iconSize={20} />
				<div>
					<h3 class="text-base font-bold text-on-surface">Create New Directory</h3>
					<span class="text-[11px] text-on-surface-variant font-medium">Path: <code class="font-mono text-primary">{currentPath}</code></span>
				</div>
			</div>
			<input
				type="text"
				bind:value={newFolderName}
				placeholder="Folder name (e.g. MyFolder)"
				class="bg-surface-container-high rounded-2xl px-4 py-2.5 text-xs font-semibold text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 border-0 outline-none"
			/>
			<div class="flex gap-2 justify-end pt-1">
				<button onclick={oncancelnewfolder} class="px-4 py-2.5 rounded-2xl text-xs font-bold text-on-surface-variant hover:bg-surface-container-high border-0 outline-none cursor-pointer">Cancel</button>
				<button onclick={oncreatenewfolder} class="px-4 py-2.5 rounded-2xl text-xs font-bold bg-primary text-on-primary hover:brightness-110 border-0 outline-none cursor-pointer shadow-xs">Create Folder</button>
			</div>
		</div>
	</div>
{/if}

<!-- Rename Modal -->
{#if showRenameModal}
	<div class="fixed inset-0 bg-black/60 backdrop-blur-md flex items-center justify-center z-50 p-4 animate-fade-in">
		<div class="bg-surface-container p-6 rounded-[32px] max-w-sm w-full flex flex-col gap-5 shadow-2xl border border-surface-container-high/60">
			<div class="flex items-center gap-3">
				<ShapeBadge icon="drive_file_rename_outline" shape="gem" size={40} iconSize={20} />
				<div>
					<h3 class="text-base font-bold text-on-surface">Rename Item</h3>
					<span class="text-[11px] text-on-surface-variant font-medium">Original: <code class="font-mono text-primary">{selectedFile?.name}</code></span>
				</div>
			</div>
			<input
				type="text"
				bind:value={renameNewName}
				placeholder="New name"
				class="bg-surface-container-high rounded-2xl px-4 py-2.5 text-xs font-semibold text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 border-0 outline-none"
			/>
			<div class="flex gap-2 justify-end pt-1">
				<button onclick={oncancelrename} class="px-4 py-2.5 rounded-2xl text-xs font-bold text-on-surface-variant hover:bg-surface-container-high border-0 outline-none cursor-pointer">Cancel</button>
				<button onclick={onrename} class="px-4 py-2.5 rounded-2xl text-xs font-bold bg-primary text-on-primary hover:brightness-110 border-0 outline-none cursor-pointer shadow-xs">Rename</button>
			</div>
		</div>
	</div>
{/if}

<!-- Upload File Modal -->
{#if showPushModal}
	<div class="fixed inset-0 bg-black/60 backdrop-blur-md flex items-center justify-center z-50 p-4 animate-fade-in">
		<div class="bg-surface-container p-6 rounded-[32px] max-w-md w-full flex flex-col gap-5 shadow-2xl border border-surface-container-high/60">
			<div class="flex items-center gap-3">
				<ShapeBadge icon="upload_file" shape="clover" size={40} iconSize={20} />
				<div>
					<h3 class="text-base font-bold text-on-surface">Upload File to Device</h3>
					<span class="text-[11px] text-on-surface-variant font-medium">Destination: <code class="font-mono text-primary">{currentPath}</code></span>
				</div>
			</div>

			<div class="flex flex-col gap-2">
				<label for="local-file-path-input" class="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant">Local File Path</label>
				<div class="flex items-center gap-2">
					<input
						id="local-file-path-input"
						type="text"
						bind:value={localPushPath}
						placeholder="/path/to/local/file.txt"
						class="flex-1 bg-surface-container-high rounded-2xl px-4 py-2.5 text-xs font-mono text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 border-0 outline-none"
					/>
					<button
						type="button"
						onclick={triggerBrowseFile}
						class="px-4 py-2.5 rounded-2xl bg-surface-container-high hover:bg-surface-container-highest text-on-surface text-xs font-bold transition-all border-0 outline-none cursor-pointer flex items-center gap-1.5 shrink-0"
					>
						<span class="material-symbols-outlined text-[16px]">folder_open</span>
						Browse
					</button>
				</div>
			</div>

			<div class="flex gap-2 justify-end pt-1">
				<button onclick={oncancelpush} class="px-4 py-2.5 rounded-2xl text-xs font-bold text-on-surface-variant hover:bg-surface-container-high border-0 outline-none cursor-pointer">Cancel</button>
				<button onclick={onpush} class="px-4 py-2.5 rounded-2xl text-xs font-bold bg-primary text-on-primary hover:brightness-110 border-0 outline-none cursor-pointer shadow-xs">Upload File</button>
			</div>
		</div>
	</div>
{/if}

<!-- Download Item Modal -->
{#if showPullModal}
	<div class="fixed inset-0 bg-black/60 backdrop-blur-md flex items-center justify-center z-50 p-4 animate-fade-in">
		<div class="bg-surface-container p-6 rounded-[32px] max-w-md w-full flex flex-col gap-5 shadow-2xl border border-surface-container-high/60">
			<div class="flex items-center gap-3">
				<ShapeBadge icon="download" shape="sunny" size={40} iconSize={20} />
				<div>
					<h3 class="text-base font-bold text-on-surface">Download {selectedFile?.is_dir ? 'Folder' : 'File'} to PC</h3>
					<span class="text-[11px] text-on-surface-variant font-medium">Source: <code class="font-mono text-primary">{selectedFile?.name}</code></span>
				</div>
			</div>
			<div class="flex flex-col gap-1.5">
				<label for="local-pull-path-input" class="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant">Local Destination Path</label>
				<input
					id="local-pull-path-input"
					type="text"
					bind:value={localPullPath}
					placeholder="/path/to/destination"
					class="bg-surface-container-high rounded-2xl px-4 py-2.5 text-xs font-mono text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 border-0 outline-none"
				/>
			</div>
			<div class="flex gap-2 justify-end pt-1">
				<button onclick={oncancelpull} class="px-4 py-2.5 rounded-2xl text-xs font-bold text-on-surface-variant hover:bg-surface-container-high border-0 outline-none cursor-pointer">Cancel</button>
				<button onclick={onpull} class="px-4 py-2.5 rounded-2xl text-xs font-bold bg-primary text-on-primary hover:brightness-110 border-0 outline-none cursor-pointer shadow-xs">Download {selectedFile?.is_dir ? 'Folder' : 'File'}</button>
			</div>
		</div>
	</div>
{/if}

<!-- Custom Material 3 Delete Confirmation Modal -->
{#if confirmModalState.show}
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-md animate-fade-in">
		<div class="bg-surface-container rounded-[32px] p-6 max-w-md w-full shadow-2xl flex flex-col gap-5 border border-surface-container-high/60">
			<div class="flex items-center gap-3">
				<ShapeBadge
					icon={confirmModalState.icon}
					shape="burst"
					size={44}
					iconSize={22}
					bgClass={confirmModalState.isDanger ? 'bg-error/20' : 'bg-primary/20'}
					textClass={confirmModalState.isDanger ? 'text-error' : 'text-primary'}
				/>
				<div>
					<h3 class="text-base font-bold text-on-surface">{confirmModalState.title}</h3>
					<span class="text-[11px] text-on-surface-variant/80 font-medium">Confirmation Required</span>
				</div>
			</div>

			<p class="text-xs text-on-surface-variant leading-relaxed bg-surface-container-high/40 p-4 rounded-2xl border border-surface-container-high/30">
				{confirmModalState.message}
			</p>

			<div class="flex items-center justify-end gap-3 pt-2">
				<button
					onclick={oncancelconfirm}
					class="px-5 py-2.5 rounded-2xl text-xs font-bold bg-surface-container-high hover:bg-surface-container-highest text-on-surface transition-all active:scale-95 cursor-pointer border-0 outline-none"
				>
					Cancel
				</button>
				<button
					onclick={() => {
						oncancelconfirm();
						confirmModalState.action();
					}}
					class="px-5 py-2.5 rounded-2xl text-xs font-bold text-white shadow-md transition-all active:scale-95 cursor-pointer border-0 outline-none {confirmModalState.isDanger ? 'bg-error hover:brightness-110' : 'bg-primary hover:brightness-110'}"
				>
					{confirmModalState.confirmText}
				</button>
			</div>
		</div>
	</div>
{/if}
