<script lang="ts">
	interface Props {
		selectedCount: number;
		onenable: () => Promise<void>;
		ondisable: () => Promise<void>;
		onuninstall: () => Promise<void>;
		onclear: () => void;
	}

	let { selectedCount, onenable, ondisable, onuninstall, onclear }: Props = $props();

	let processing = $state(false);

	async function runAction(fn: () => Promise<void>) {
		if (processing) return;
		processing = true;
		try {
			await fn();
		} finally {
			processing = false;
		}
	}
</script>

{#if selectedCount > 0}
	<div
		class="absolute bottom-6 left-10 right-10 bg-surface-container-highest border border-primary/20 shadow-2xl rounded-3xl p-4 flex flex-col sm:flex-row items-center justify-between gap-4 z-20 transition-all duration-300 animate-slide-up"
	>
		<div class="flex items-center gap-3 pl-2">
			<span class="material-symbols-outlined text-[20px] text-primary">dynamic_feed</span>
			<span class="text-sm font-bold text-on-surface">
				Batch Action: {selectedCount} packages selected
			</span>
		</div>
		<div class="flex items-center gap-2.5 w-full sm:w-auto">
			<button
				onclick={() => runAction(onenable)}
				class="flex-1 sm:flex-none flex items-center justify-center gap-1.5 rounded-full bg-primary-container text-on-primary-container px-4 py-2 text-xs font-bold hover:brightness-105 transition-all disabled:opacity-50"
				disabled={processing}
			>
				<span class="material-symbols-outlined text-[16px]">play_circle</span>
				Enable
			</button>
			<button
				onclick={() => runAction(ondisable)}
				class="flex-1 sm:flex-none flex items-center justify-center gap-1.5 rounded-full bg-amber-500/10 text-amber-500 border border-amber-500/25 px-4 py-2 text-xs font-bold hover:brightness-105 transition-all disabled:opacity-50"
				disabled={processing}
			>
				<span class="material-symbols-outlined text-[16px]">block</span>
				Disable
			</button>
			<button
				onclick={() => runAction(onuninstall)}
				class="flex-1 sm:flex-none flex items-center justify-center gap-1.5 rounded-full bg-error text-on-error px-4 py-2 text-xs font-bold hover:brightness-105 transition-all disabled:opacity-50"
				disabled={processing}
			>
				<span class="material-symbols-outlined text-[16px]">delete</span>
				Uninstall
			</button>
			<div class="h-6 w-px bg-outline-variant/40 hidden sm:block"></div>
			<button
				onclick={onclear}
				class="flex-1 sm:flex-none flex items-center justify-center gap-1 text-xs font-semibold py-2 px-3 text-on-surface-variant hover:text-on-surface uppercase disabled:opacity-50"
				disabled={processing}
			>
				Clear
			</button>
		</div>
	</div>
{/if}

<style>
	@keyframes slide-up {
		from {
			transform: translateY(30px);
			opacity: 0;
		}
		to {
			transform: translateY(0);
			opacity: 1;
		}
	}
	.animate-slide-up {
		animation: slide-up 0.22s cubic-bezier(0.2, 0.8, 0.2, 1) forwards;
	}
</style>
