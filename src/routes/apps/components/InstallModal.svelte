<script lang="ts">
	interface Props {
		show: boolean;
		installing: boolean;
		oncancel: () => void;
		oninstall: (path: string) => Promise<void>;
	}

	let { show, installing, oncancel, oninstall }: Props = $props();

	let apkPath = $state('');
	let isDragging = $state(false);
	let fileInput: HTMLInputElement;

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (!apkPath.trim() || installing) return;
		await oninstall(apkPath.trim());
		apkPath = '';
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		isDragging = false;
		if (installing) return;

		const files = e.dataTransfer?.files;
		if (files && files.length > 0) {
			const file = files[0];
			const fullPath = (file as unknown as { path?: string }).path || file.name;
			apkPath = fullPath;
		}
	}

	function handleFileSelect(e: Event) {
		const target = e.target as HTMLInputElement;
		if (target.files && target.files.length > 0) {
			const file = target.files[0];
			const fullPath = (file as unknown as { path?: string }).path || file.name;
			apkPath = fullPath;
		}
	}

	function browseFile() {
		if (fileInput) fileInput.click();
	}
</script>

{#if show}
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-md">
		<form
			onsubmit={handleSubmit}
			class="w-full max-w-md rounded-[32px] bg-surface-container-high p-7 shadow-2xl animate-scale-up flex flex-col gap-5"
		>
			<header class="flex items-center gap-3">
				<div class="flex h-11 w-11 items-center justify-center rounded-2xl bg-primary-container text-on-primary-container shadow-inner">
					<span class="material-symbols-outlined text-[24px]">install_mobile</span>
				</div>
				<div>
					<h3 class="text-lg font-bold text-on-surface">Install APK Package</h3>
					<p class="text-xs text-on-surface-variant/80 font-medium">Install Android application on connected target</p>
				</div>
			</header>

			<!-- Drag & Drop Zone -->
			<div
				role="region"
				aria-label="APK File Dropzone"
				ondragover={(e) => { e.preventDefault(); isDragging = true; }}
				ondragleave={() => (isDragging = false)}
				ondrop={handleDrop}
				class="flex flex-col items-center justify-center p-8 rounded-2xl transition-all cursor-pointer text-center gap-3.5 {isDragging ? 'bg-primary/20 scale-[1.02]' : apkPath ? 'bg-primary/10' : 'bg-surface-container hover:bg-surface-container-highest'}"
				onclick={browseFile}
			>
				<div class="flex h-14 w-14 items-center justify-center rounded-full bg-surface-container-high text-primary shadow-xs">
					<span class="material-symbols-outlined text-[30px]">{isDragging ? 'downloading' : apkPath ? 'check_circle' : 'cloud_upload'}</span>
				</div>
				
				{#if apkPath}
					<div class="flex flex-col items-center gap-1">
						<span class="text-xs font-bold text-on-surface">Selected APK File:</span>
						<span class="text-[11px] font-mono text-primary font-bold break-all bg-primary/15 px-3 py-1 rounded-xl max-w-full truncate">{apkPath}</span>
						<span class="text-[10px] text-on-surface-variant/70 mt-1">Click to change file</span>
					</div>
				{:else}
					<div class="flex flex-col items-center gap-0.5">
						<div class="text-xs font-bold text-on-surface">
							{isDragging ? 'Drop .APK file to install' : 'Drag & drop .APK file here'}
						</div>
						<span class="text-[10px] text-on-surface-variant font-semibold">or click to browse from computer</span>
					</div>
				{/if}
				
				<input
					type="file"
					accept=".apk"
					bind:this={fileInput}
					onchange={handleFileSelect}
					class="hidden"
				/>
			</div>

			<!-- Modal Footer -->
			<footer class="flex justify-end gap-3 pt-2">
				<button
					type="button"
					onclick={() => {
						apkPath = '';
						oncancel();
					}}
					class="px-5 py-2.5 rounded-full text-xs font-bold text-on-surface-variant hover:bg-surface-container-highest transition-all uppercase"
					disabled={installing}
				>
					Cancel
				</button>
				<button
					type="submit"
					class="flex items-center gap-2 rounded-full bg-primary px-6 py-2.5 text-xs font-bold text-on-primary hover:brightness-110 transition-all shadow-sm disabled:opacity-50"
					disabled={installing || !apkPath.trim()}
				>
					{#if installing}
						<span class="animate-spin h-3.5 w-3.5 border-2 border-on-primary border-t-transparent rounded-full"></span>
						Installing...
					{:else}
						<span class="material-symbols-outlined text-[16px]">check</span>
						Install APK
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
