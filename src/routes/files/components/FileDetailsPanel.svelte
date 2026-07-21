<script lang="ts">
	interface FileEntry {
		name: string;
		is_dir: boolean;
		size: number;
		permissions: string;
		modified: number;
	}

	interface Props {
		selectedFile: FileEntry | null;
		getFileIcon: (name: string, is_dir: boolean) => string;
		formatBytes: (bytes: number) => string;
		formatTimestamp: (time: number) => string;
		onclose: () => void;
		onpull: () => void;
		oncopypath: () => void;
		onrename: () => void;
		ondelete: () => void;
	}

	let {
		selectedFile,
		getFileIcon,
		formatBytes,
		formatTimestamp,
		onclose,
		onpull,
		oncopypath,
		onrename,
		ondelete
	}: Props = $props();
</script>

{#if selectedFile}
	<div class="w-72 shrink-0 rounded-[28px] bg-surface-container p-5 flex flex-col justify-between gap-4 animate-fade-in shadow-sm">
		<div class="flex flex-col gap-4">
			<div class="flex items-center justify-between">
				<h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">File Properties</h3>
				<button onclick={onclose} class="text-on-surface-variant hover:text-on-surface border-0 outline-none cursor-pointer">
					<span class="material-symbols-outlined text-[18px]">close</span>
				</button>
			</div>

			<div class="flex flex-col items-center p-4 rounded-2xl bg-surface-container-high/50 gap-2">
				<span class="material-symbols-outlined text-[48px] text-primary">
					{getFileIcon(selectedFile.name, selectedFile.is_dir)}
				</span>
				<span class="text-xs font-bold text-on-surface text-center break-all">{selectedFile.name}</span>
			</div>

			<div class="flex flex-col gap-2 text-xs">
				<div class="flex justify-between py-1">
					<span class="text-on-surface-variant">Type</span>
					<span class="font-bold text-on-surface">{selectedFile.is_dir ? 'Directory' : 'File'}</span>
				</div>
				<div class="flex justify-between py-1">
					<span class="text-on-surface-variant">Size</span>
					<span class="font-mono text-on-surface">{formatBytes(selectedFile.size)}</span>
				</div>
				<div class="flex justify-between py-1">
					<span class="text-on-surface-variant">Permissions</span>
					<span class="font-mono text-on-surface">{selectedFile.permissions}</span>
				</div>
				<div class="flex justify-between py-1">
					<span class="text-on-surface-variant">Modified</span>
					<span class="font-mono text-on-surface text-[10px]">{formatTimestamp(selectedFile.modified)}</span>
				</div>
			</div>
		</div>

		<div class="flex flex-col gap-2">
			<button
				onclick={onpull}
				class="flex items-center justify-center gap-2 py-2.5 px-3 rounded-2xl bg-primary text-on-primary hover:brightness-110 text-xs font-bold transition-all shadow-xs border-0 outline-none cursor-pointer active:scale-95"
			>
				<span class="material-symbols-outlined text-[16px]">download</span>
				Download {selectedFile.is_dir ? 'Folder' : 'File'} to PC
			</button>

			<button
				onclick={oncopypath}
				class="flex items-center justify-center gap-2 py-2.5 px-3 rounded-2xl bg-surface-container-high hover:bg-surface-container-highest text-on-surface text-xs font-bold transition-all border-0 outline-none cursor-pointer active:scale-95"
			>
				<span class="material-symbols-outlined text-[16px]">content_copy</span>
				Copy Path
			</button>

			<button
				onclick={onrename}
				class="flex items-center justify-center gap-2 py-2.5 px-3 rounded-2xl bg-surface-container-high hover:bg-surface-container-highest text-on-surface text-xs font-bold transition-all border-0 outline-none cursor-pointer active:scale-95"
			>
				<span class="material-symbols-outlined text-[16px]">drive_file_rename_outline</span>
				Rename
			</button>

			<button
				onclick={ondelete}
				class="flex items-center justify-center gap-2 py-2.5 px-3 rounded-2xl bg-error/15 text-error hover:bg-error/25 text-xs font-bold transition-all border-0 outline-none cursor-pointer active:scale-95"
			>
				<span class="material-symbols-outlined text-[16px]">delete_forever</span>
				Delete
			</button>
		</div>
	</div>
{/if}
