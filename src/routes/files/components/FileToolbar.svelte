<script lang="ts">
	export type CategoryFilter = 'all' | 'folders' | 'media' | 'documents' | 'archives' | 'apks';

	interface FileEntry {
		name: string;
		is_dir: boolean;
		size: number;
		permissions: string;
		modified: number;
	}

	interface Props {
		selectedFile: FileEntry | null;
		selectedCount: number;
		searchQuery: string;
		viewMode: 'list' | 'grid';
		selectedCategory: CategoryFilter;
		onnewfolder: () => void;
		onupload: () => void;
		onpull: () => void;
		oncopypath: () => void;
		onrename: () => void;
		ondelete: () => void;
		onbatchdelete: () => void;
		onclearselection: () => void;
		onsearchchange: (query: string) => void;
		onviewmodechange: (mode: 'list' | 'grid') => void;
		oncategorychange: (cat: CategoryFilter) => void;
	}

	let {
		selectedFile,
		selectedCount,
		searchQuery,
		viewMode,
		selectedCategory,
		onnewfolder,
		onupload,
		onpull,
		oncopypath,
		onrename,
		ondelete,
		onbatchdelete,
		onclearselection,
		onsearchchange,
		onviewmodechange,
		oncategorychange
	}: Props = $props();

	const categories: Array<{ id: CategoryFilter; label: string; icon: string }> = [
		{ id: 'all', label: 'All Files', icon: 'category' },
		{ id: 'folders', label: 'Folders', icon: 'folder' },
		{ id: 'media', label: 'Photos & Videos', icon: 'image' },
		{ id: 'documents', label: 'Documents', icon: 'description' },
		{ id: 'archives', label: 'Archives', icon: 'archive' },
		{ id: 'apks', label: 'APKs', icon: 'android' }
	];
</script>

<div class="flex flex-col gap-3 shrink-0 w-full">
	<!-- Toolbar Actions & Search Bar -->
	<div class="flex flex-wrap items-center justify-between gap-3 w-full">
		<div class="flex items-center gap-2 flex-wrap">
			<button
				onclick={onnewfolder}
				class="flex items-center gap-1.5 px-3.5 py-1.5 rounded-full bg-primary text-on-primary hover:brightness-110 text-xs font-bold transition-all shadow-xs shrink-0 cursor-pointer border-0 outline-none active:scale-95"
			>
				<span class="material-symbols-outlined text-[16px]">create_new_folder</span>
				New Folder
			</button>

			<button
				onclick={onupload}
				class="flex items-center gap-1.5 px-3.5 py-1.5 rounded-full bg-surface-container-high hover:bg-surface-container-highest text-on-surface text-xs font-bold transition-all shrink-0 cursor-pointer border-0 outline-none active:scale-95"
			>
				<span class="material-symbols-outlined text-[16px]">upload_file</span>
				Upload File
			</button>

			{#if selectedCount > 0}
				<div class="h-4 w-px bg-surface-container-highest mx-1"></div>
				<span class="text-xs font-bold text-primary px-1">{selectedCount} selected</span>
				
				<button
					onclick={onbatchdelete}
					class="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-error/15 text-error hover:bg-error/25 text-xs font-bold transition-all shrink-0 cursor-pointer border-0 outline-none active:scale-95"
				>
					<span class="material-symbols-outlined text-[16px]">delete_sweep</span>
					Delete Selected
				</button>
				<button
					onclick={onclearselection}
					class="text-[11px] font-bold text-on-surface-variant hover:text-on-surface px-2.5 py-1 rounded-full bg-surface-container-high transition-all border-0 outline-none cursor-pointer"
				>
					Clear
				</button>
			{:else if selectedFile}
				<div class="h-4 w-px bg-surface-container-highest mx-1"></div>

				<button
					onclick={onpull}
					class="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-surface-container-high hover:bg-surface-container-highest text-primary text-xs font-bold transition-all shrink-0 cursor-pointer border-0 outline-none active:scale-95"
					title="Download to PC"
				>
					<span class="material-symbols-outlined text-[16px]">download</span>
					Pull {selectedFile.is_dir ? 'Folder' : 'File'}
				</button>

				<button
					onclick={oncopypath}
					class="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-surface-container-high hover:bg-surface-container-highest text-on-surface text-xs font-bold transition-all shrink-0 cursor-pointer border-0 outline-none active:scale-95"
				>
					<span class="material-symbols-outlined text-[16px]">content_copy</span>
					Copy Path
				</button>

				<button
					onclick={onrename}
					class="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-surface-container-high hover:bg-surface-container-highest text-on-surface text-xs font-bold transition-all shrink-0 cursor-pointer border-0 outline-none active:scale-95"
				>
					<span class="material-symbols-outlined text-[16px]">drive_file_rename_outline</span>
					Rename
				</button>

				<button
					onclick={ondelete}
					class="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-error/15 text-error hover:bg-error/25 text-xs font-bold transition-all shrink-0 cursor-pointer border-0 outline-none active:scale-95"
				>
					<span class="material-symbols-outlined text-[16px]">delete_forever</span>
					Delete
				</button>
			{/if}
		</div>

		<!-- Filter & View Controls -->
		<div class="flex items-center gap-2">
			<div class="relative">
				<span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-on-surface-variant text-[16px]">search</span>
				<input
					type="text"
					value={searchQuery}
					oninput={(e) => onsearchchange((e.target as HTMLInputElement).value)}
					placeholder="Search files..."
					class="bg-surface-container-high rounded-full pl-9 pr-3 py-1.5 text-xs text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 w-40 border-0 outline-none"
				/>
			</div>

			<!-- Grid/List toggle -->
			<div class="flex bg-surface-container p-1 rounded-full shrink-0">
				<button
					onclick={() => onviewmodechange('list')}
					class="p-1 rounded-full text-xs transition-all border-0 outline-none cursor-pointer {viewMode === 'list' ? 'bg-primary text-on-primary shadow-xs' : 'text-on-surface-variant hover:text-on-surface'}"
					title="List View"
				>
					<span class="material-symbols-outlined text-[16px] block">view_list</span>
				</button>
				<button
					onclick={() => onviewmodechange('grid')}
					class="p-1 rounded-full text-xs transition-all border-0 outline-none cursor-pointer {viewMode === 'grid' ? 'bg-primary text-on-primary shadow-xs' : 'text-on-surface-variant hover:text-on-surface'}"
					title="Grid View"
				>
					<span class="material-symbols-outlined text-[16px] block">grid_view</span>
				</button>
			</div>
		</div>
	</div>

	<!-- Category Chips Bar -->
	<div class="flex items-center gap-1.5 overflow-x-auto pb-1 text-[11px] font-bold">
		{#each categories as cat}
			<button
				onclick={() => oncategorychange(cat.id)}
				class="flex items-center gap-1.5 px-3 py-1 rounded-full transition-all border-0 outline-none cursor-pointer shrink-0 {selectedCategory === cat.id ? 'bg-secondary-container text-on-secondary-container shadow-xs' : 'bg-surface-container text-on-surface-variant hover:bg-surface-container-high hover:text-on-surface'}"
			>
				<span class="material-symbols-outlined text-[14px]">{cat.icon}</span>
				<span>{cat.label}</span>
			</button>
		{/each}
	</div>
</div>
