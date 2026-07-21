<script lang="ts">
	interface FileEntry {
		name: string;
		is_dir: boolean;
		size: number;
		permissions: string;
		modified: number;
	}

	interface Props {
		show: boolean;
		x: number;
		y: number;
		file: FileEntry | null;
		formatBytes: (bytes: number) => string;
		onopenfolder: (file: FileEntry) => void;
		onpull: (file: FileEntry) => void;
		oncopypath: (file: FileEntry) => void;
		onrename: (file: FileEntry) => void;
		ondelete: (file: FileEntry) => void;
	}

	let {
		show,
		x,
		y,
		file,
		formatBytes,
		onopenfolder,
		onpull,
		oncopypath,
		onrename,
		ondelete
	}: Props = $props();
</script>

{#if show && file}
	<div
		style="left: {x}px; top: {y}px;"
		class="fixed z-50 bg-surface-container p-2 rounded-2xl shadow-2xl border border-surface-container-high/60 flex flex-col gap-1 w-48 animate-fade-in text-xs"
	>
		<div class="px-3 py-1.5 border-b border-surface-container-high/50 mb-1">
			<span class="font-bold text-on-surface block truncate">{file.name}</span>
			<span class="text-[10px] text-on-surface-variant font-mono block">{file.is_dir ? 'Directory' : formatBytes(file.size)}</span>
		</div>

		{#if file.is_dir}
			<button
				onclick={() => onopenfolder(file)}
				class="flex items-center gap-2 px-3 py-2 rounded-xl text-on-surface hover:bg-surface-container-high transition-all text-left border-0 outline-none cursor-pointer font-semibold"
			>
				<span class="material-symbols-outlined text-[16px] text-primary">folder_open</span>
				Open Folder
			</button>
		{/if}

		<button
			onclick={() => onpull(file)}
			class="flex items-center gap-2 px-3 py-2 rounded-xl text-on-surface hover:bg-surface-container-high transition-all text-left border-0 outline-none cursor-pointer font-semibold"
		>
			<span class="material-symbols-outlined text-[16px] text-primary">download</span>
			Download to PC
		</button>

		<button
			onclick={() => oncopypath(file)}
			class="flex items-center gap-2 px-3 py-2 rounded-xl text-on-surface hover:bg-surface-container-high transition-all text-left border-0 outline-none cursor-pointer font-semibold"
		>
			<span class="material-symbols-outlined text-[16px]">content_copy</span>
			Copy Remote Path
		</button>

		<button
			onclick={() => onrename(file)}
			class="flex items-center gap-2 px-3 py-2 rounded-xl text-on-surface hover:bg-surface-container-high transition-all text-left border-0 outline-none cursor-pointer font-semibold"
		>
			<span class="material-symbols-outlined text-[16px]">drive_file_rename_outline</span>
			Rename
		</button>

		<button
			onclick={() => ondelete(file)}
			class="flex items-center gap-2 px-3 py-2 rounded-xl text-error hover:bg-error/15 transition-all text-left border-0 outline-none cursor-pointer font-semibold"
		>
			<span class="material-symbols-outlined text-[16px]">delete_forever</span>
			Delete
		</button>
	</div>
{/if}
