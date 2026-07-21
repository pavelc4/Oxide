<script lang="ts">
	interface Props {
		currentPath: string;
		loadingFiles: boolean;
		onnavigate: (path: string) => void;
		onnavigateup: () => void;
	}

	let { currentPath, loadingFiles, onnavigate, onnavigateup }: Props = $props();

	let isEditingPath = $state(false);
	let editPathInput = $state(currentPath);

	$effect(() => {
		editPathInput = currentPath;
	});

	let breadcrumbs = $derived.by(() => {
		const parts = currentPath.split('/').filter(Boolean);
		let accum = '';
		return parts.map((part) => {
			accum += '/' + part;
			return { name: part, path: accum };
		});
	});

	function handlePathSubmit() {
		isEditingPath = false;
		if (editPathInput.trim()) {
			onnavigate(editPathInput.trim());
		}
	}
</script>

<div class="flex items-center gap-2 bg-surface-container p-2.5 rounded-2xl shadow-xs w-full">
	<button
		onclick={onnavigateup}
		disabled={currentPath === '/'}
		class="flex h-8 w-8 items-center justify-center rounded-xl bg-surface-container-high text-on-surface hover:bg-surface-container-highest disabled:opacity-30 transition-all shrink-0 border-0 outline-none cursor-pointer"
		title="Go Up One Directory"
	>
		<span class="material-symbols-outlined text-[18px]">arrow_upward</span>
	</button>

	<!-- Editable Address Bar / Breadcrumb Trail -->
	{#if isEditingPath}
		<form onsubmit={(e) => { e.preventDefault(); handlePathSubmit(); }} class="flex-1 flex items-center gap-2">
			<input
				type="text"
				bind:value={editPathInput}
				class="flex-1 bg-surface-container-high rounded-xl px-3 py-1.5 text-xs font-mono text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 border-0 outline-none"
				placeholder="/sdcard/path"
			/>
			<button type="submit" class="px-3 py-1.5 rounded-xl bg-primary text-on-primary text-xs font-bold border-0 outline-none cursor-pointer">Go</button>
			<button type="button" onclick={() => (isEditingPath = false)} class="px-3 py-1.5 rounded-xl bg-surface-container-high text-on-surface text-xs font-bold border-0 outline-none cursor-pointer">Cancel</button>
		</form>
	{:else}
		<div
			onclick={() => (isEditingPath = true)}
			class="flex-1 flex items-center gap-1 overflow-x-auto text-xs font-mono px-2 min-w-0 cursor-text hover:bg-surface-container-high/40 py-1 rounded-xl transition-all"
			title="Click to type path directly"
		>
			<button
				onclick={(e) => { e.stopPropagation(); onnavigate('/'); }}
				class="hover:text-primary font-bold transition-colors shrink-0 border-0 outline-none cursor-pointer"
			>
				/
			</button>
			{#each breadcrumbs as crumb, idx}
				<span class="text-on-surface-variant/40 shrink-0">/</span>
				<button
					onclick={(e) => { e.stopPropagation(); onnavigate(crumb.path); }}
					class="hover:text-primary transition-colors whitespace-nowrap shrink-0 border-0 outline-none cursor-pointer {idx === breadcrumbs.length - 1 ? 'text-primary font-bold' : 'text-on-surface-variant'}"
				>
					{crumb.name}
				</button>
			{/each}
		</div>
	{/if}

	<button
		onclick={() => onnavigate(currentPath)}
		class="flex h-8 w-8 items-center justify-center rounded-xl bg-surface-container-high text-on-surface hover:bg-surface-container-highest transition-all shrink-0 border-0 outline-none cursor-pointer"
		title="Refresh Current Folder"
	>
		<span class="material-symbols-outlined text-[18px] {loadingFiles ? 'animate-spin' : ''}">refresh</span>
	</button>
</div>
