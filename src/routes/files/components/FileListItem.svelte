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
		formatTimestamp: (time: number) => string;
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
		formatTimestamp,
		onclick,
		ondblclick,
		oncontextmenu,
		ondragover,
		ondragleave,
		ondrop
	}: Props = $props();
</script>

<tr
	{onclick}
	{ondblclick}
	{oncontextmenu}
	ondragover={(e) => file.is_dir && ondragover(e)}
	ondragleave={(e) => file.is_dir && ondragleave(e)}
	ondrop={(e) => file.is_dir && ondrop(e)}
	class="hover:bg-surface-container-high/70 transition-all cursor-pointer group rounded-2xl {isSelected ? 'bg-primary/15 font-bold' : ''} {isDragTarget ? 'ring-2 ring-primary bg-primary/20' : ''}"
>
	<td class="py-2.5 px-4 rounded-l-2xl">
		<div class="flex items-center gap-3">
			<span class="material-symbols-outlined text-[22px] shrink-0 {file.is_dir ? 'text-primary font-fill' : 'text-primary/80'}">
				{getFileIcon(file.name, file.is_dir)}
			</span>
			<span class="font-semibold text-on-surface group-hover:text-primary transition-colors truncate">{file.name}</span>
		</div>
	</td>
	<td class="py-2.5 px-4 text-on-surface-variant text-right font-mono text-[11px]">
		{file.is_dir ? '—' : formatBytes(file.size)}
	</td>
	<td class="py-2.5 px-4 text-on-surface-variant/70 text-center font-mono text-[10px] hidden lg:table-cell">
		{file.permissions}
	</td>
	<td class="py-2.5 px-4 text-on-surface-variant text-right font-mono text-[11px] rounded-r-2xl hidden sm:table-cell">
		{formatTimestamp(file.modified)}
	</td>
</tr>
