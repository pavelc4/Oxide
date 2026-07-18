<script lang="ts">
	interface Props {
		show: boolean;
		installing: boolean;
		oncancel: () => void;
		oninstall: (path: string) => Promise<void>;
	}

	let { show, installing, oncancel, oninstall }: Props = $props();

	let apkPath = $state('');

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (!apkPath.trim() || installing) return;
		await oninstall(apkPath.trim());
		apkPath = '';
	}
</script>

{#if show}
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/40 backdrop-blur-sm">
		<form
			onsubmit={handleSubmit}
			class="w-full max-w-md rounded-[28px] bg-surface-container-high p-6 shadow-2xl border border-outline-variant/30 animate-scale-up"
		>
			<header class="mb-4 flex items-center gap-3">
				<div
					class="flex h-10 w-10 items-center justify-center rounded-full bg-primary-container text-on-primary-container"
				>
					<span class="material-symbols-outlined">install_mobile</span>
				</div>
				<h3 class="text-lg font-bold text-on-surface">Install APK Package</h3>
			</header>

			<p class="text-xs text-on-surface-variant font-medium mb-4 leading-relaxed">
				Specify the absolute path of the APK file on your host computer to install it on the selected device.
			</p>

			<div class="mb-5">
				<label
					for="apk-path-input"
					class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant block mb-1.5 pl-1"
				>
					Host APK Path
				</label>
				<input
					id="apk-path-input"
					type="text"
					bind:value={apkPath}
					placeholder="e.g. /home/downloads/app.apk or C:\apps\app.apk"
					class="w-full bg-surface-container rounded-2xl px-4 py-2.5 text-sm text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all border border-outline-variant/30"
					disabled={installing}
					required
				/>
			</div>

			<footer class="flex justify-end gap-3">
				<button
					type="button"
					onclick={() => {
						apkPath = '';
						oncancel();
					}}
					class="px-5 py-2 rounded-full text-xs font-bold text-on-surface-variant hover:bg-surface-variant/40 transition-all uppercase"
					disabled={installing}
				>
					Cancel
				</button>
				<button
					type="submit"
					class="flex items-center gap-1.5 rounded-full bg-primary px-6 py-2 text-xs font-bold text-on-primary hover:brightness-105 transition-all shadow-sm disabled:opacity-50"
					disabled={installing || !apkPath.trim()}
				>
					{#if installing}
						<span
							class="animate-spin h-3.5 w-3.5 border-2 border-on-primary border-t-transparent rounded-full"
						></span>
						Installing...
					{:else}
						<span class="material-symbols-outlined text-[16px]">check</span>
						Install
					{/if}
				</button>
			</footer>
		</form>
	</div>
{/if}

<style>
	@keyframes scale-up {
		from {
			transform: scale(0.96);
			opacity: 0;
		}
		to {
			transform: scale(1);
			opacity: 1;
		}
	}
	.animate-scale-up {
		animation: scale-up 0.18s cubic-bezier(0.3, 0, 0.2, 1) forwards;
	}
</style>
