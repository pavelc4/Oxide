<script lang="ts">
	interface Props {
		pkg: string;
		selected: boolean;
		isActive: boolean;
		ontoggle: () => void;
		onview: () => void;
	}

	let { pkg, selected, isActive, ontoggle, onview }: Props = $props();

	function getCategoryIcon(packageName: string): string {
		const name = packageName.toLowerCase();
		if (name.includes('chrome') || name.includes('firefox') || name.includes('browser') || name.includes('opera') || name.includes('kiwi'))
			return 'language';
		if (name.includes('music') || name.includes('spotify') || name.includes('soundcloud') || name.includes('deezer'))
			return 'music_note';
		if (name.includes('whatsapp') || name.includes('instagram') || name.includes('facebook') || name.includes('messenger') || name.includes('telegram') || name.includes('twitter') || name.includes('reddit') || name.includes('discord') || name.includes('tiktok'))
			return 'chat';
		if (name.includes('youtube') || name.includes('netflix') || name.includes('vlc') || name.includes('mxplayer') || name.includes('player') || name.includes('video') || name.includes('primevideo'))
			return 'play_circle';
		if (name.includes('camera') || name.includes('gallery') || name.includes('photos') || name.includes('snapseed'))
			return 'photo_camera';
		if (name.includes('settings') || name.includes('setup') || name.includes('provision'))
			return 'settings';
		if (name.includes('terminal') || name.includes('termux') || name.includes('shell'))
			return 'terminal';
		if (name.includes('file') || name.includes('documents') || name.includes('manager') || name.includes('explorer'))
			return 'folder_open';
		if (name.includes('game') || name.includes('steam') || name.includes('playstation'))
			return 'sports_esports';

		if (packageName.startsWith('com.android.') || packageName.startsWith('com.google.android.') || packageName === 'android') {
			return 'settings_suggest';
		}
		return 'extension';
	}

	const isSystem = $derived(
		pkg.startsWith('com.android.') || pkg.startsWith('com.google.') || pkg === 'android'
	);

	function handleClick(e: MouseEvent) {
		if (e.ctrlKey || e.metaKey || e.shiftKey) {
			ontoggle();
		} else {
			onview();
		}
	}
</script>

<div
	role="button"
	tabindex="0"
	onclick={handleClick}
	onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') onview(); }}
	class="group flex items-center justify-between rounded-xl px-4 py-2.5 transition-all cursor-pointer select-none {selected ? 'bg-primary-container/30 text-on-primary-container font-bold shadow-xs' : isActive ? 'bg-surface-container-highest text-on-surface font-semibold' : 'bg-surface-container-low hover:bg-surface-container-highest text-on-surface'}"
>
	<div class="flex items-center gap-3 flex-1 min-w-0">
		<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-surface-container-high text-primary shrink-0 shadow-xs">
			<span class="material-symbols-outlined text-[18px]">{getCategoryIcon(pkg)}</span>
		</div>

		<div class="flex flex-col min-w-0">
			<span class="text-xs font-semibold truncate group-hover:text-primary transition-colors">
				{pkg}
			</span>
			<div class="flex items-center gap-1.5 mt-0.5">
				{#if isSystem}
					<span class="text-[8px] bg-tertiary-container/60 text-on-tertiary-container px-1.5 py-0.5 rounded-full font-bold uppercase tracking-wider">System</span>
				{:else}
					<span class="text-[8px] bg-secondary-container/60 text-on-secondary-container px-1.5 py-0.5 rounded-full font-bold uppercase tracking-wider">User</span>
				{/if}
			</div>
		</div>
	</div>

	<!-- Multi-selected check indicator or active arrow -->
	<div class="flex items-center gap-2 shrink-0">
		{#if selected}
			<span class="material-symbols-outlined text-primary text-[18px]">check_circle</span>
		{:else}
			<span class="material-symbols-outlined text-on-surface-variant text-[16px] opacity-0 group-hover:opacity-100 transition-opacity">
				chevron_right
			</span>
		{/if}
	</div>
</div>
