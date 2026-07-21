<script lang="ts">
	import ShapeBadge from '$lib/components/ShapeBadge.svelte';
	import type { MaterialShapeType } from '$lib/shapes/materialShapes';

	interface QuickLocation {
		label: string;
		path: string;
		icon: string;
		shape: MaterialShapeType | 'rounded';
	}

	interface Props {
		currentPath: string;
		filesCount: number;
		totalDirSize: number;
		onnavigate: (path: string) => void;
		formatBytes: (bytes: number) => string;
	}

	let { currentPath, filesCount, totalDirSize, onnavigate, formatBytes }: Props = $props();

	const quickLocations: QuickLocation[] = [
		{ label: 'Internal Storage', path: '/sdcard', icon: 'smartphone', shape: 'cookie7' },
		{ label: 'Downloads', path: '/sdcard/Download', icon: 'download', shape: 'clover' },
		{ label: 'DCIM (Photos)', path: '/sdcard/DCIM', icon: 'photo_camera', shape: 'sunny' },
		{ label: 'Pictures', path: '/sdcard/Pictures', icon: 'image', shape: 'gem' },
		{ label: 'Music', path: '/sdcard/Music', icon: 'library_music', shape: 'ghostish' },
		{ label: 'Movies', path: '/sdcard/Movies', icon: 'movie', shape: 'arch' },
		{ label: 'Documents', path: '/sdcard/Documents', icon: 'description', shape: 'bun' },
		{ label: 'System Root', path: '/system', icon: 'settings', shape: 'triangle' },
		{ label: 'Data Partition', path: '/data', icon: 'database', shape: 'pixelCircle' }
	];
</script>

<div class="w-60 shrink-0 flex flex-col justify-between gap-4 overflow-y-auto hidden md:flex">
	<div class="rounded-[28px] bg-surface-container p-4 flex flex-col gap-2 shadow-xs">
		<span class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80 px-2 mb-1">Quick Access</span>
		
		<div class="flex flex-col gap-1">
			{#each quickLocations as loc}
				{@const isActive = currentPath === loc.path || currentPath.startsWith(loc.path + '/')}
				<button
					onclick={() => onnavigate(loc.path)}
					class="flex items-center gap-3 px-3.5 py-2.5 rounded-2xl text-xs font-semibold transition-all text-left border-0 outline-none cursor-pointer {isActive ? 'bg-primary text-on-primary font-bold shadow-xs' : 'text-on-surface-variant hover:bg-surface-container-high hover:text-on-surface'}"
				>
					<span class="material-symbols-outlined text-[18px]">{loc.icon}</span>
					<span class="truncate">{loc.label}</span>
				</button>
			{/each}
		</div>
	</div>

	<!-- Directory Summary Card -->
	<div class="rounded-[28px] bg-surface-container p-4 flex flex-col gap-2 shadow-xs text-xs">
		<span class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">Folder Overview</span>
		<div class="flex items-center justify-between text-on-surface font-semibold pt-1">
			<span class="text-on-surface-variant">Items</span>
			<span>{filesCount}</span>
		</div>
		<div class="flex items-center justify-between text-on-surface font-semibold">
			<span class="text-on-surface-variant">Total Size</span>
			<span class="font-mono text-[11px]">{formatBytes(totalDirSize)}</span>
		</div>
	</div>
</div>
