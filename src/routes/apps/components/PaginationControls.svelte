<script lang="ts">
	interface Props {
		currentPage: number;
		totalPages: number;
		onprev: () => void;
		onnext: () => void;
	}

	let { currentPage, totalPages, onprev, onnext }: Props = $props();
</script>

{#if totalPages > 1}
	<footer class="flex items-center justify-between pt-3 mt-3 shrink-0">
		<span class="text-xs text-on-surface-variant font-medium">
			Page {currentPage} of {totalPages}
		</span>
		<div class="flex items-center gap-2">
			<button
				onclick={onprev}
				class="flex h-8 px-3 items-center gap-1 rounded-full bg-surface-container-high text-xs font-semibold text-on-surface hover:brightness-105 active:scale-95 transition-all shadow-xs disabled:opacity-40 disabled:scale-100"
				disabled={currentPage <= 1}
			>
				<span class="material-symbols-outlined text-[16px]">chevron_left</span>
				Prev
			</button>
			<button
				onclick={onnext}
				class="flex h-8 px-3 items-center gap-1 rounded-full bg-surface-container-high text-xs font-semibold text-on-surface hover:brightness-105 active:scale-95 transition-all shadow-xs disabled:opacity-40 disabled:scale-100"
				disabled={currentPage >= totalPages}
			>
				Next
				<span class="material-symbols-outlined text-[16px]">chevron_right</span>
			</button>
		</div>
	</footer>
{/if}
