<script lang="ts">
	interface ProcessItem {
		pid: number;
		user: string;
		name: string;
		cpu: number;
		memMb: number; // Raw byte/KB value from ps
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

	function formatMemory(rawNum: number): string {
		if (!rawNum || rawNum <= 0) return '0 KB';
		// If rawNum > 1,000,000, it's raw Bytes (e.g. 10,962,792 B = 10.45 MB)
		// If rawNum <= 1,000,000, it's in KB (e.g. 4380 KB = 4.2 MB)
		let bytes = rawNum;
		if (rawNum < 1000000) {
			bytes = rawNum * 1024;
		}

		if (bytes >= 1024 * 1024 * 1024) {
			return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
		}
		if (bytes >= 1024 * 1024) {
			return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
		}
		return `${Math.round(bytes / 1024)} KB`;
	}
</script>

<div
	role="button"
	tabindex="0"
	onclick={onselect}
	onkeydown={(e) => { if (e.key === 'Enter') onselect(); }}
	class="flex flex-col px-3.5 py-2.5 rounded-xl transition-all cursor-pointer gap-2.5 outline-none select-none {isSelected ? 'bg-surface-container-highest shadow-xs' : 'hover:bg-surface-container-high/60'}"
>
	<!-- Main Row Content (Clear Column Separation to Prevent Name/User Blending) -->
	<div class="grid grid-cols-12 gap-3 items-center w-full font-mono text-xs">
		<!-- PID (Col 1) -->
		<div class="col-span-2 sm:col-span-1 text-[11px] text-primary font-bold truncate">
			{proc.pid}
		</div>

		<!-- Process Name (Col 4) -->
		<div class="col-span-5 sm:col-span-4 text-xs font-bold text-on-surface truncate font-sans">
			{proc.name}
		</div>

		<!-- Type Badge (Col 2) -->
		<div class="col-span-3 sm:col-span-2">
			{#if proc.isSystem}
				<span class="text-[9px] font-bold bg-surface-container text-on-surface-variant px-2.5 py-0.5 rounded-full uppercase tracking-wider">SYSTEM</span>
			{:else}
				<span class="text-[9px] font-bold bg-primary/20 text-primary px-2.5 py-0.5 rounded-full uppercase tracking-wider">USER APP</span>
			{/if}
		</div>

		<!-- Memory (Col 2) -->
		<div class="col-span-2 sm:col-span-2 text-[11px] font-bold text-on-surface-variant truncate">
			{formatMemory(proc.memMb)}
		</div>

		<!-- Status (Col 1) -->
		<div class="col-span-2 sm:col-span-1 text-[10px] text-on-surface-variant/70 font-medium truncate">
			{proc.status === 'running' ? 'Active' : 'Sleep'}
		</div>

		<!-- User / Owner (Col 2 - Far Right to avoid confusion with Process Name) -->
		<div class="col-span-3 sm:col-span-2 text-[11px] text-on-surface-variant/60 truncate font-mono text-right pr-1">
			{proc.user}
		</div>
	</div>

	<!-- High-Contrast Expanded Details Card -->
	{#if isSelected}
		<div
			role="region"
			aria-label="Process Details"
			class="w-full mt-1.5 flex flex-col gap-3 bg-surface-container-high p-4 rounded-2xl font-mono text-xs animate-fade-in shadow-sm"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
		>
			<div class="flex justify-between items-center text-on-surface-variant font-sans">
				<span class="font-bold text-primary text-xs flex items-center gap-1.5">
					<span class="material-symbols-outlined text-[16px]">info</span>
					Process Properties Payload
				</span>
				<div class="flex items-center gap-2">
					<button
						onclick={() => oncopy(proc)}
						class="flex items-center gap-1.5 text-[11px] font-bold bg-surface-container-highest hover:bg-surface-container-low text-on-surface px-3.5 py-1.5 rounded-xl transition-all shadow-xs border-0 outline-none cursor-pointer"
					>
						<span class="material-symbols-outlined text-[15px]">content_copy</span>
						Copy Payload
					</button>
					<button
						onclick={() => onkill(proc)}
						class="flex items-center gap-1.5 text-[11px] font-bold bg-error/20 text-error hover:bg-error/30 px-3.5 py-1.5 rounded-xl transition-all shadow-xs border-0 outline-none cursor-pointer active:scale-95"
					>
						<span class="material-symbols-outlined text-[15px]">close</span>
						Kill Process ({proc.pid})
					</button>
				</div>
			</div>

			<!-- Clear High-Contrast Specifications Grid -->
			<div class="grid grid-cols-2 sm:grid-cols-4 gap-2.5 font-mono text-xs">
				<div class="bg-surface-container p-3 rounded-xl flex flex-col gap-1">
					<span class="text-[10px] text-on-surface-variant/80 font-bold uppercase tracking-wider">Process ID</span>
					<span class="font-bold text-primary text-sm">{proc.pid}</span>
				</div>
				<div class="bg-surface-container p-3 rounded-xl flex flex-col gap-1">
					<span class="text-[10px] text-on-surface-variant/80 font-bold uppercase tracking-wider">Owner User</span>
					<span class="font-bold text-on-surface truncate">{proc.user}</span>
				</div>
				<div class="bg-surface-container p-3 rounded-xl flex flex-col gap-1">
					<span class="text-[10px] text-on-surface-variant/80 font-bold uppercase tracking-wider">RAM Memory</span>
					<span class="font-bold text-on-surface">{formatMemory(proc.memMb)}</span>
				</div>
				<div class="bg-surface-container p-3 rounded-xl flex flex-col gap-1">
					<span class="text-[10px] text-on-surface-variant/80 font-bold uppercase tracking-wider">Task State</span>
					<span class="font-bold text-on-surface truncate">{proc.stateStr || 'S (active)'}</span>
				</div>
			</div>
		</div>
	{/if}
</div>
