<script lang="ts">
	interface Props {
		pkg: string;
		selected: boolean;
		isActive: boolean;
		ontoggle: () => void;
		onview: () => void;
	}

	let { pkg, selected, isActive, ontoggle, onview }: Props = $props();

	// Smart pattern matching for lightweight icons
	function getCategoryIcon(packageName: string): string {
		const name = packageName.toLowerCase();
		if (
			name.includes('chrome') ||
			name.includes('firefox') ||
			name.includes('browser') ||
			name.includes('opera') ||
			name.includes('kiwi')
		)
			return 'language';
		if (
			name.includes('music') ||
			name.includes('spotify') ||
			name.includes('soundcloud') ||
			name.includes('deezer')
		)
			return 'music_note';
		if (
			name.includes('whatsapp') ||
			name.includes('instagram') ||
			name.includes('facebook') ||
			name.includes('messenger') ||
			name.includes('telegram') ||
			name.includes('twitter') ||
			name.includes('reddit') ||
			name.includes('discord') ||
			name.includes('tiktok')
		)
			return 'chat';
		if (
			name.includes('youtube') ||
			name.includes('netflix') ||
			name.includes('vlc') ||
			name.includes('mxplayer') ||
			name.includes('player') ||
			name.includes('video') ||
			name.includes('primevideo')
		)
			return 'play_circle';
		if (
			name.includes('camera') ||
			name.includes('gallery') ||
			name.includes('photos') ||
			name.includes('snapseed')
		)
			return 'photo_camera';
		if (name.includes('settings') || name.includes('setup') || name.includes('provision'))
			return 'settings';
		if (name.includes('terminal') || name.includes('termux') || name.includes('shell'))
			return 'terminal';
		if (
			name.includes('file') ||
			name.includes('documents') ||
			name.includes('manager') ||
			name.includes('explorer')
		)
			return 'folder_open';
		if (name.includes('game') || name.includes('steam') || name.includes('playstation'))
			return 'sports_esports';

		// Defaults based on package type
		if (
			packageName.startsWith('com.android.') ||
			packageName.startsWith('com.google.android.') ||
			packageName === 'android'
		) {
			return 'settings_suggest';
		}
		return 'extension';
	}

	const isSystem = $derived(
		pkg.startsWith('com.android.') || pkg.startsWith('com.google.') || pkg === 'android'
	);
</script>

<div
	onclick={onview}
	class="group flex items-center justify-between rounded-xl bg-surface-container-low px-4 py-2.5 transition-all hover:bg-surface-container-highest cursor-pointer border-b border-outline-variant/5 last:border-b-0 {isActive ? 'bg-surface-container-highest ring-1 ring-primary/20' : ''}"
>
	<!-- Checkbox & Label details -->
	<div class="flex items-center gap-3 flex-1 min-w-0" onclick={(e) => e.stopPropagation()}>
		<input
			type="checkbox"
			checked={selected}
			onclick={ontoggle}
			class="w-4 h-4 rounded text-primary focus:ring-primary/50 border-outline-variant"
		/>
		<div class="flex flex-col min-w-0">
			<span
				class="text-[13px] font-semibold text-on-surface truncate group-hover:text-primary transition-colors"
			>
				{pkg}
			</span>
			<div class="flex items-center gap-1.5 mt-0.5">
				<span class="material-symbols-outlined text-[12px] text-on-surface-variant/80">
					{getCategoryIcon(pkg)}
				</span>
				{#if isSystem}
					<span
						class="text-[8px] bg-tertiary-container/60 text-on-tertiary-container px-1 rounded font-bold tracking-wide uppercase"
						>System</span
					>
				{:else}
					<span
						class="text-[8px] bg-secondary-container/60 text-on-secondary-container px-1 rounded font-bold tracking-wide uppercase"
						>User</span
					>
				{/if}
			</div>
		</div>
	</div>

	<!-- Action arrow indicator -->
	<span
		class="material-symbols-outlined text-on-surface-variant text-[16px] opacity-0 group-hover:opacity-100 transition-opacity ml-2 shrink-0"
	>
		chevron_right
	</span>
</div>
