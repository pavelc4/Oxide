<script lang="ts">
	interface FileEntry {
		name: string;
		is_dir: boolean;
		size: number;
		permissions: string;
		modified: number;
	}

	interface Props {
		file: FileEntry;
		isSelected: boolean;
		isDragTarget: boolean;
		getFileIcon: (name: string, is_dir: boolean) => string;
		formatBytes: (bytes: number) => string;
		onclick: (e: MouseEvent) => void;
		ondblclick: () => void;
		oncontextmenu: (e: MouseEvent) => void;
		ondragover: (e: DragEvent) => void;
		ondragleave: (e: DragEvent) => void;
		ondrop: (e: DragEvent) => void;
	}

	let {
		file,
		isSelected,
		isDragTarget,
		getFileIcon,
		formatBytes,
		onclick,
		ondblclick,
		oncontextmenu,
		ondragover,
		ondragleave,
		ondrop
	}: Props = $props();
</script>

<button
	{onclick}
	{ondblclick}
	{oncontextmenu}
	ondragover={(e) => file.is_dir && ondragover(e)}
	ondragleave={(e) => file.is_dir && ondragleave(e)}
	ondrop={(e) => file.is_dir && ondrop(e)}
	class="flex flex-col items-center gap-2 p-4 rounded-2xl transition-all group text-center border-0 outline-none cursor-pointer relative {isSelected ? 'bg-primary-container/40 ring-2 ring-primary/60' : 'bg-surface-container-high/40 hover:bg-surface-container-highest'} {isDragTarget ? 'ring-2 ring-primary bg-primary/20' : ''}"
>
	<span class="material-symbols-outlined text-[36px] {file.is_dir ? 'text-primary' : 'text-primary/80'}">
		{getFileIcon(file.name, file.is_dir)}
	</span>
	<span class="text-xs font-semibold text-on-surface truncate w-full group-hover:text-primary transition-colors">{file.name}</span>
	<span class="text-[10px] font-mono text-on-surface-variant/70">{file.is_dir ? 'Folder' : formatBytes(file.size)}</span>
</button>
