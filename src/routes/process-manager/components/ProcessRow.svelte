<script lang="ts">
	interface ProcessItem {
		pid: number;
		user: string;
		name: string;
		cpu: number;
		memMb: number; // Raw KB value from ps
		isSystem: boolean;
		status: 'running' | 'sleeping';
		threads?: number;
		stateStr?: string;
	}

	interface Props {
		proc: ProcessItem;
		isSelected: boolean;
		onselect: () => void;
		oncopy: (proc: ProcessItem) => void;
		onkill: (proc: ProcessItem) => void;
	}

	let { proc, isSelected, onselect, oncopy, onkill }: Props = $props();

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

<div
	role="button"
	tabindex="0"
	onclick={onselect}
	onkeydown={(e) => { if (e.key === 'Enter') onselect(); }}
	class="flex flex-col px-3 py-2.5 rounded-xl transition-all cursor-pointer gap-2 {isSelected ? 'bg-surface-container-highest shadow-xs' : 'hover:bg-surface-container-high/60'}"
>
	<!-- Main Row Content (Matches Logcat Grid) -->
	<div class="grid grid-cols-12 gap-3 items-center w-full font-mono text-xs">
		<!-- PID -->
		<div class="col-span-2 sm:col-span-1 text-[11px] text-on-surface-variant/80 font-bold truncate">
			{proc.pid}
		</div>

		<!-- User -->
		<div class="col-span-3 sm:col-span-2 text-[11px] text-on-surface-variant/70 truncate">
			{proc.user}
		</div>

		<!-- Category / Type Badge -->
		<div class="col-span-3 sm:col-span-2">
			{#if proc.isSystem}
				<span class="text-[9px] font-bold bg-purple-500/15 text-purple-400 px-2.5 py-0.5 rounded-full uppercase tracking-wider">SYSTEM</span>
			{:else}
				<span class="text-[9px] font-bold bg-emerald-500/15 text-emerald-400 px-2.5 py-0.5 rounded-full uppercase tracking-wider">USER APP</span>
			{/if}
		</div>

		<!-- Memory -->
		<div class="col-span-4 sm:col-span-2 text-[11px] font-semibold text-on-surface-variant truncate">
			{formatMemory(proc.memMb)}
		</div>

		<!-- Process Name -->
		<div class="col-span-12 sm:col-span-5 text-xs font-bold text-on-surface truncate font-sans">
			{proc.name}
		</div>
	</div>

	<!-- Inline Expanded Process Details Inspector (Logcat Style) -->
	{#if isSelected}
		<div
			role="region"
			aria-label="Process Details"
			class="w-full mt-1.5 flex flex-col gap-3 bg-surface-container-high p-4 rounded-2xl font-mono text-xs animate-fade-in shadow-xs"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
		>
			<div class="flex justify-between items-center text-on-surface-variant font-sans">
				<span class="font-bold text-primary text-xs flex items-center gap-1.5">
					<span class="material-symbols-outlined text-[16px]">info</span>
					Process Properties Payload (PID {proc.pid})
				</span>
				<div class="flex items-center gap-2">
					<button
						onclick={() => oncopy(proc)}
						class="flex items-center gap-1.5 text-[11px] font-bold bg-surface-container hover:bg-surface-container-highest text-on-surface px-3 py-1.5 rounded-xl transition-all border-0 outline-none cursor-pointer"
					>
						<span class="material-symbols-outlined text-[14px]">content_copy</span>
						Copy Details
					</button>
					<button
						onclick={() => onkill(proc)}
						class="flex items-center gap-1.5 text-[11px] font-bold bg-error/15 text-error hover:bg-error/25 px-3 py-1.5 rounded-xl transition-all border-0 outline-none cursor-pointer active:scale-95"
					>
						<span class="material-symbols-outlined text-[14px]">close</span>
						Kill Process ({proc.pid})
					</button>
				</div>
			</div>

			<!-- Grid Details -->
			<div class="grid grid-cols-2 sm:grid-cols-4 gap-2.5 font-mono text-xs text-on-surface">
				<div class="bg-surface-container p-2.5 rounded-xl flex flex-col gap-0.5">
					<span class="text-[10px] text-on-surface-variant/70 font-bold uppercase">Process ID</span>
					<span class="font-bold text-primary">{proc.pid}</span>
				</div>
				<div class="bg-surface-container p-2.5 rounded-xl flex flex-col gap-0.5">
					<span class="text-[10px] text-on-surface-variant/70 font-bold uppercase">Owner User</span>
					<span class="font-bold text-on-surface truncate">{proc.user}</span>
				</div>
				<div class="bg-surface-container p-2.5 rounded-xl flex flex-col gap-0.5">
					<span class="text-[10px] text-on-surface-variant/70 font-bold uppercase">RAM Memory</span>
					<span class="font-bold text-on-surface">{formatMemory(proc.memMb)}</span>
				</div>
				<div class="bg-surface-container p-2.5 rounded-xl flex flex-col gap-0.5">
					<span class="text-[10px] text-on-surface-variant/70 font-bold uppercase">Task State</span>
					<span class="font-bold text-on-surface truncate">{proc.stateStr || 'S (active)'}</span>
				</div>
			</div>
		</div>
	{/if}
</div>
