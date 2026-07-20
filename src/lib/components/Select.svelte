<script lang="ts">
	interface Option {
		value: string;
		label: string;
	}

	let {
		value = $bindable(),
		options = [],
		id = ''
	}: {
		value: string;
		options: Option[];
		id?: string;
	} = $props();

	let open = $state(false);
	let selectedLabel = $derived(options.find((o) => o.value === value)?.label || value);
</script>

<div class="relative w-full">
	<button
		type="button"
		{id}
		onclick={() => (open = !open)}
		class="w-full flex items-center justify-between px-4 py-2.5 rounded-2xl bg-surface-container-high hover:bg-surface-container-highest text-xs font-semibold text-on-surface transition-all focus:outline-none shadow-xs border-0 outline-none cursor-pointer"
	>
		<span class="truncate">{selectedLabel}</span>
		<span
			class="material-symbols-outlined text-[18px] text-on-surface-variant transition-transform duration-200 shrink-0"
			class:rotate-180={open}
		>
			expand_more
		</span>
	</button>

	{#if open}
		<!-- Backdrop to close dropdown -->
		<button
			type="button"
			tabindex="-1"
			aria-label="Close dropdown"
			onclick={() => (open = false)}
			class="fixed inset-0 z-40 bg-transparent cursor-default border-none outline-none"
		></button>

		<!-- Custom Material 3 Floating Menu -->
		<div
			class="absolute left-0 right-0 top-full mt-1.5 z-50 rounded-2xl bg-surface-container-highest p-1.5 shadow-2xl border border-surface-container-highest/40 flex flex-col gap-1 max-h-56 overflow-y-auto animate-fade-in"
		>
			{#each options as opt (opt.value)}
				<button
					type="button"
					onclick={() => {
						value = opt.value;
						open = false;
					}}
					class="w-full text-left px-3.5 py-2 rounded-xl text-xs font-semibold transition-all flex items-center justify-between cursor-pointer border-0 outline-none {value === opt.value
						? 'bg-primary/20 text-primary font-bold'
						: 'text-on-surface hover:bg-surface-container-high'}"
				>
					<span class="truncate">{opt.label}</span>
					{#if value === opt.value}
						<span class="material-symbols-outlined text-[16px] text-primary shrink-0">check</span>
					{/if}
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	@keyframes fadeIn {
		from {
			opacity: 0;
			transform: translateY(-4px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
	.animate-fade-in {
		animation: fadeIn 0.15s cubic-bezier(0.2, 0.8, 0.2, 1) forwards;
	}
</style>
