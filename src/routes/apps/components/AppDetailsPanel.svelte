<script lang="ts">
	interface AppDetails {
		package_name: string;
		version_name: string | null;
		version_code: number | null;
		label: string | null;
		install_location: string;
		flags: string[];
		first_install_time: string | null;
		last_update_time: string | null;
		apk_path: string | null;
		data_dir: string | null;
		is_system_app: boolean;
		is_enabled: boolean;
	}

	interface Props {
		selectedAppDetails: AppDetails | null;
		loadingDetails: boolean;
		onforcestop: (pkg: string) => Promise<void>;
		oncleardata: (pkg: string) => Promise<void>;
		ontogglestatus: (pkg: string, enable: boolean) => Promise<void>;
		onuninstall: (pkg: string) => Promise<void>;
		onpullapk: (pkg: string) => Promise<void>;
		onlaunch: (pkg: string) => Promise<void>;
	}

	let {
		selectedAppDetails,
		loadingDetails,
		onforcestop,
		oncleardata,
		ontogglestatus,
		onuninstall,
		onpullapk,
		onlaunch
	}: Props = $props();

	let runningAction = $state(false);

	async function runAction(fn: () => Promise<void>) {
		if (runningAction) return;
		runningAction = true;
		try {
			await fn();
		} finally {
			runningAction = false;
		}
	}
</script>

<div
	class="w-80 shrink-0 flex flex-col rounded-[24px] bg-surface-container border border-outline-variant/15 p-5 overflow-hidden"
>
	{#if loadingDetails}
		<div class="flex-1 flex flex-col items-center justify-center gap-3">
			<span class="animate-spin h-8 w-8 border-4 border-primary border-t-transparent rounded-full"></span>
			<p class="text-xs text-on-surface-variant font-medium">Fetching app metadata...</p>
		</div>
	{:else if selectedAppDetails}
		<div class="flex-1 flex flex-col overflow-hidden animate-fade-in">
			<!-- App Header -->
			<div class="border-b border-outline-variant/30 pb-4 mb-4 shrink-0">
				<div class="flex items-start justify-between">
					<div
						class="flex h-12 w-12 items-center justify-center rounded-[18px] bg-primary-container text-on-primary-container shadow-inner"
					>
						<span class="material-symbols-outlined text-[24px]">
							{selectedAppDetails.is_system_app ? 'settings_suggest' : 'extension'}
						</span>
					</div>
					<!-- Status tag -->
					<span
						class="text-[10px] font-bold px-3 py-1 rounded-full {selectedAppDetails.is_enabled
							? 'bg-primary/10 text-primary'
							: 'bg-error/15 text-error'}"
					>
						{selectedAppDetails.is_enabled ? 'ENABLED' : 'DISABLED'}
					</span>
				</div>
				<h3 class="text-lg font-bold text-on-surface mt-3 break-all leading-tight">
					{selectedAppDetails.label || selectedAppDetails.package_name.split('.').pop()}
				</h3>
				<p class="text-xs text-on-surface-variant font-semibold break-all mt-1">
					{selectedAppDetails.package_name}
				</p>
			</div>

			<!-- Technical details -->
			<div class="flex-1 overflow-y-auto pr-1 flex flex-col gap-4 text-xs">
				<div>
					<span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant block mb-1"
						>Version</span
					>
					<p class="font-semibold text-on-surface">
						{selectedAppDetails.version_name || 'N/A'} ({selectedAppDetails.version_code || 'N/A'})
					</p>
				</div>

				<div>
					<span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant block mb-1"
						>Install Location</span
					>
					<p class="font-semibold text-on-surface">{selectedAppDetails.install_location}</p>
				</div>

				{#if selectedAppDetails.first_install_time}
					<div>
						<span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant block mb-1"
							>Installed At</span
						>
						<p class="font-semibold text-on-surface">{selectedAppDetails.first_install_time}</p>
					</div>
				{/if}

				{#if selectedAppDetails.last_update_time}
					<div>
						<span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant block mb-1"
							>Updated At</span
						>
						<p class="font-semibold text-on-surface">{selectedAppDetails.last_update_time}</p>
					</div>
				{/if}

				{#if selectedAppDetails.apk_path}
					<div>
						<span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant block mb-1"
							>APK Storage Path</span
						>
						<p
							class="font-mono text-[10px] bg-surface-container-high border border-outline-variant/15 p-2 rounded-lg break-all select-all text-on-surface"
						>
							{selectedAppDetails.apk_path}
						</p>
					</div>
				{/if}

				{#if selectedAppDetails.data_dir}
					<div>
						<span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant block mb-1"
							>Data Directory</span
						>
						<p
							class="font-mono text-[10px] bg-surface-container-high border border-outline-variant/15 p-2 rounded-lg break-all select-all text-on-surface"
						>
							{selectedAppDetails.data_dir}
						</p>
					</div>
				{/if}
			</div>

			<!-- Actions panel -->
			<div class="border-t border-outline-variant/30 pt-4 mt-4 shrink-0 flex flex-col gap-2">
				<div class="flex gap-2">
					<button
						onclick={() => runAction(() => onlaunch(selectedAppDetails!.package_name))}
						class="flex-1 flex items-center justify-center gap-1.5 rounded-full bg-primary text-on-primary hover:brightness-105 transition-all text-xs font-bold py-2 shadow-sm disabled:opacity-50"
						disabled={runningAction || !selectedAppDetails.is_enabled}
						title="Launch the application on device screen"
					>
						<span class="material-symbols-outlined text-[16px]">play_arrow</span>
						Launch
					</button>
					<button
						onclick={() => runAction(() => onpullapk(selectedAppDetails!.package_name))}
						class="flex-1 flex items-center justify-center gap-1.5 rounded-full bg-surface-container-highest hover:brightness-105 transition-all text-xs font-bold py-2 text-on-surface border border-outline-variant/40 disabled:opacity-50"
						disabled={runningAction}
						title="Pull base APK to local Downloads folder"
					>
						<span class="material-symbols-outlined text-[16px]">download</span>
						Pull APK
					</button>
				</div>

				<div class="flex gap-2">
					<button
						onclick={() => runAction(() => onforcestop(selectedAppDetails!.package_name))}
						class="flex-1 flex items-center justify-center gap-1 rounded-full bg-surface-container-highest hover:brightness-105 transition-all text-[11px] font-semibold py-1.5 text-on-surface border border-outline-variant/40 disabled:opacity-50"
						disabled={runningAction}
					>
						Stop
					</button>
					<button
						onclick={() => runAction(() => oncleardata(selectedAppDetails!.package_name))}
						class="flex-1 flex items-center justify-center gap-1 rounded-full bg-surface-container-highest hover:brightness-105 transition-all text-[11px] font-semibold py-1.5 text-on-surface border border-outline-variant/40 disabled:opacity-50"
						disabled={runningAction}
					>
						Clear Data
					</button>
				</div>

				<div class="flex gap-2 mt-1">
					<button
						onclick={() =>
							runAction(() =>
								ontogglestatus(selectedAppDetails!.package_name, !selectedAppDetails!.is_enabled)
							)}
						class="flex-1 flex items-center justify-center gap-1.5 rounded-full hover:brightness-105 transition-all text-[11px] font-bold py-2 disabled:opacity-50 {selectedAppDetails.is_enabled
							? 'bg-amber-500/10 text-amber-500 border border-amber-500/20'
							: 'bg-primary-container text-on-primary-container border border-transparent'}"
						disabled={runningAction}
					>
						<span class="material-symbols-outlined text-[14px]">
							{selectedAppDetails.is_enabled ? 'block' : 'check_circle'}
						</span>
						{selectedAppDetails.is_enabled ? 'Disable' : 'Enable'}
					</button>

					<button
						onclick={() => runAction(() => onuninstall(selectedAppDetails!.package_name))}
						class="flex-1 flex items-center justify-center gap-1.5 rounded-full bg-error text-on-error hover:brightness-105 transition-all text-[11px] font-bold py-2 disabled:opacity-50"
						disabled={runningAction || selectedAppDetails.is_system_app}
					>
						<span class="material-symbols-outlined text-[14px]">delete</span>
						Uninstall
					</button>
				</div>
			</div>
		</div>
	{:else}
		<div
			class="flex-1 flex flex-col items-center justify-center p-6 text-center text-on-surface-variant select-none"
		>
			<div class="flex h-14 w-14 items-center justify-center rounded-full bg-surface-container-high mb-4">
				<span class="material-symbols-outlined text-[28px] opacity-60">info</span>
			</div>
			<p class="text-sm font-semibold">No package selected</p>
			<p class="text-xs text-on-surface-variant opacity-80 mt-1.5 max-w-[200px]">
				Click any package on the list to view its properties and execute operations.
			</p>
		</div>
	{/if}
</div>

<style>
	@keyframes fadeIn {
		from {
			opacity: 0;
			transform: scale(0.98);
		}
		to {
			opacity: 1;
			transform: scale(1);
		}
	}
	.animate-fade-in {
		animation: fadeIn 0.18s cubic-bezier(0.2, 0.8, 0.2, 1) forwards;
	}
</style>
