<script lang="ts">
	interface LogEntry {
		id: number;
		time: string;
		pid: number;
		tid: number;
		level: 'V' | 'D' | 'I' | 'W' | 'E' | 'F';
		tag: string;
		message: string;
	}

	interface Props {
		log: LogEntry;
		isSelected: boolean;
		onselect: () => void;
		oncopy: (log: LogEntry) => void;
	}

	let { log, isSelected, onselect, oncopy }: Props = $props();
</script>

<div
	role="button"
	tabindex="0"
	onclick={onselect}
	onkeydown={(e) => { if (e.key === 'Enter') onselect(); }}
	class="flex flex-col px-3 py-2 rounded-xl transition-all cursor-pointer gap-2 {isSelected ? 'bg-surface-container-highest shadow-xs' : 'hover:bg-surface-container-high/60'}"
>
	<!-- Row Content -->
	<div class="grid grid-cols-12 gap-3 items-center w-full font-mono text-xs">
		<!-- Time -->
		<div class="col-span-2 sm:col-span-2 text-[11px] text-on-surface-variant/80 truncate">
			{log.time}
		</div>

		<!-- PID -->
		<div class="col-span-1 hidden sm:block text-[11px] text-on-surface-variant/60">
			{log.pid}
		</div>

		<!-- Level Badge -->
		<div class="col-span-1">
			{#if log.level === 'I'}
				<span class="text-[9px] font-bold bg-emerald-500/15 text-emerald-400 px-2 py-0.5 rounded-full">INFO</span>
			{:else if log.level === 'D'}
				<span class="text-[9px] font-bold bg-sky-500/15 text-sky-400 px-2 py-0.5 rounded-full">DEBUG</span>
			{:else if log.level === 'W'}
				<span class="text-[9px] font-bold bg-amber-500/15 text-amber-400 px-2 py-0.5 rounded-full">WARN</span>
			{:else if log.level === 'E'}
				<span class="text-[9px] font-bold bg-rose-500/15 text-rose-400 px-2 py-0.5 rounded-full">ERROR</span>
			{:else if log.level === 'F'}
				<span class="text-[9px] font-bold bg-purple-500/15 text-purple-400 px-2 py-0.5 rounded-full">FATAL</span>
			{:else}
				<span class="text-[9px] font-bold bg-neutral-500/15 text-neutral-400 px-2 py-0.5 rounded-full">VERB</span>
			{/if}
		</div>

		<!-- Tag -->
		<div class="col-span-3 sm:col-span-2 text-xs font-bold text-on-surface truncate">
			{log.tag}
		</div>

		<!-- Message -->
		<div class="col-span-6 sm:col-span-6 text-xs text-on-surface-variant truncate font-sans">
			{log.message}
		</div>
	</div>

	<!-- Expanded Details Card (Full-Width) -->
	{#if isSelected}
		<div
			role="region"
			aria-label="Log Details"
			class="w-full mt-1 flex flex-col gap-2.5 bg-surface-container-high p-4 rounded-2xl font-mono text-xs animate-fade-in shadow-xs"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
		>
			<div class="flex justify-between items-center text-on-surface-variant">
				<span class="font-bold text-primary text-xs">Full Log Entry Payload</span>
				<button
					onclick={() => oncopy(log)}
					class="flex items-center gap-1.5 text-[11px] font-bold bg-primary/15 text-primary px-3.5 py-1.5 rounded-xl hover:brightness-110 transition-all shadow-xs border-0 outline-none cursor-pointer"
				>
					<span class="material-symbols-outlined text-[15px]">content_copy</span>
					Copy Payload
				</button>
			</div>
			<div class="bg-surface-container p-3.5 rounded-xl text-on-surface break-all leading-relaxed font-mono text-xs shadow-inner">
				{log.time} [{log.pid}:{log.tid}] {log.level}/{log.tag}: {log.message}
			</div>
		</div>
	{/if}
</div>
