<script lang="ts">
	import ShapeBadge from '$lib/components/ShapeBadge.svelte';

	interface ProcessItem {
		pid: number;
		user: string;
		name: string;
		cpu: number;
		memMb: number;
		isSystem: boolean;
		status: 'running' | 'sleeping';
		threads?: number;
		stateStr?: string;
	}

	interface Props {
		proc: ProcessItem | null;
		onkill: () => void;
	}

	let { proc, onkill }: Props = $props();

	function formatMemory(kb: number): string {
		if (!kb || kb <= 0) return '0 KB';
		if (kb >= 1024 * 1024) {
			return `${(kb / (1024 * 1024)).toFixed(1)} GB`;
		}
		if (kb >= 1024) {
			return `${(kb / 1024).toFixed(1)} MB`;
		}
		return `${kb} KB`;
	}
</script>

{#if proc}
	<div class="w-80 shrink-0 flex flex-col rounded-[32px] bg-surface-container p-6 overflow-hidden shadow-sm animate-fade-in gap-5">
		<!-- Card Header -->
		<header class="flex items-center gap-3 pb-3 border-b border-surface-container-high/60">
			<ShapeBadge
				icon={proc.isSystem ? 'settings' : 'widgets'}
				shape="sun"
				size={44}
				iconSize={22}
				bgClass={proc.isSystem ? 'bg-purple-500/20' : 'bg-primary/20'}
				textClass={proc.isSystem ? 'text-purple-400' : 'text-primary'}
			/>
			<div class="overflow-hidden">
				<h3 class="text-sm font-bold text-on-surface truncate">{proc.name}</h3>
				<span class="text-[10px] text-on-surface-variant font-mono font-medium block">PID: {proc.pid}</span>
			</div>
		</header>

		<!-- Properties List -->
		<div class="flex flex-1 flex-col gap-3 text-xs overflow-y-auto">
			<div class="flex flex-col gap-1 bg-surface-container-high/50 p-3 rounded-2xl">
				<span class="text-[10px] text-on-surface-variant/80 font-bold uppercase tracking-wider">Process Owner</span>
				<span class="font-mono text-on-surface font-semibold">{proc.user}</span>
			</div>

			<div class="grid grid-cols-2 gap-2">
				<div class="flex flex-col gap-1 bg-surface-container-high/50 p-3 rounded-2xl">
					<span class="text-[10px] text-on-surface-variant/80 font-bold uppercase tracking-wider">RAM Memory</span>
					<span class="font-mono text-on-surface font-semibold">{formatMemory(proc.memMb)}</span>
				</div>
				<div class="flex flex-col gap-1 bg-surface-container-high/50 p-3 rounded-2xl">
					<span class="text-[10px] text-on-surface-variant/80 font-bold uppercase tracking-wider">CPU Threads</span>
					<span class="font-mono text-on-surface font-semibold">{proc.threads || '1'}</span>
				</div>
			</div>

			<div class="flex flex-col gap-1 bg-surface-container-high/50 p-3 rounded-2xl">
				<span class="text-[10px] text-on-surface-variant/80 font-bold uppercase tracking-wider">Process Category</span>
				<span class="font-semibold text-on-surface">{proc.isSystem ? 'System Framework Daemon' : 'User Installed App'}</span>
			</div>

			{#if proc.stateStr}
				<div class="flex flex-col gap-1 bg-surface-container-high/50 p-3 rounded-2xl">
					<span class="text-[10px] text-on-surface-variant/80 font-bold uppercase tracking-wider">Linux Task State</span>
					<span class="font-mono text-xs text-on-surface-variant">{proc.stateStr}</span>
				</div>
			{/if}
		</div>

		<!-- Action Footer -->
		<div class="pt-2">
			<button
				onclick={onkill}
				class="w-full flex items-center justify-center gap-2 px-4 py-2.5 rounded-2xl bg-error/15 text-error hover:bg-error/25 text-xs font-bold transition-all cursor-pointer border-0 outline-none active:scale-95"
			>
				<span class="material-symbols-outlined text-[18px]">close</span>
				Kill Process ({proc.pid})
			</button>
		</div>
	</div>
{:else}
	<div class="w-80 shrink-0 hidden lg:flex flex-col items-center justify-center rounded-[32px] bg-surface-container p-6 text-center shadow-sm">
		<span class="material-symbols-outlined text-[48px] text-on-surface-variant/40 mb-3">memory</span>
		<h4 class="text-xs font-bold text-on-surface">No Process Selected</h4>
		<p class="text-[11px] text-on-surface-variant/70 mt-1">Select any running process to view memory allocation, PID, & kill options</p>
	</div>
{/if}
