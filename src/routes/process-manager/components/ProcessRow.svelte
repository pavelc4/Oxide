<script lang="ts">
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
		proc: ProcessItem;
		isActive: boolean;
		onselect: () => void;
		onkill: (e: MouseEvent) => void;
	}

	let { proc, isActive, onselect, onkill }: Props = $props();
</script>

<div
	role="button"
	tabindex="0"
	onclick={onselect}
	onkeydown={(e) => { if (e.key === 'Enter') onselect(); }}
	class="grid grid-cols-12 gap-3 items-center px-4 py-3 rounded-2xl transition-all cursor-pointer border border-transparent text-xs font-medium {isActive ? 'bg-primary-container/30 text-on-surface border-primary/20 shadow-xs' : 'hover:bg-surface-container-high/60 text-on-surface'}"
>
	<!-- PID & Badge -->
	<div class="col-span-2 sm:col-span-2 flex items-center gap-2">
		<span class="font-mono text-xs font-bold text-primary">{proc.pid}</span>
	</div>

	<!-- Name & Package -->
	<div class="col-span-5 sm:col-span-4 flex items-center gap-2 truncate">
		<span class="font-bold text-on-surface truncate">{proc.name}</span>
	</div>

	<!-- User -->
	<div class="col-span-2 hidden sm:block text-on-surface-variant font-mono text-[11px] truncate">
		{proc.user}
	</div>

	<!-- Memory (RAM) -->
	<div class="col-span-2 sm:col-span-2 font-mono text-on-surface-variant text-right pr-2">
		{proc.memMb} MB
	</div>

	<!-- Actions / Kill Button -->
	<div class="col-span-3 sm:col-span-2 flex items-center justify-end">
		<button
			onclick={onkill}
			class="flex items-center gap-1.5 px-3 py-1 rounded-xl text-[11px] font-bold bg-error/10 hover:bg-error/20 text-error transition-all active:scale-95 border-0 outline-none cursor-pointer"
			title="Terminate Process (kill -9 {proc.pid})"
		>
			<span class="material-symbols-outlined text-[15px]">close</span>
			Kill
		</button>
	</div>
</div>
