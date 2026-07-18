<script lang="ts">
	/* eslint-disable svelte/no-navigation-without-resolve */
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | undefined;
	let isTauri = $state(false);

	interface AuditEntry {
		id: string;
		timestamp: string;
		level: 'Info' | 'Warning' | 'Error' | 'Debug' | 'Success';
		operation: string;
		message: string;
		details: string | null;
		duration_ms: number | null;
		success: boolean | null;
	}

	// State
	let entries = $state<AuditEntry[]>([]);
	let loading = $state(true);
	let error = $state('');
	let infoMessage = $state('');
	let searchQuery = $state('');
	let selectedLevelFilter = $state<string>('All');
	let expandedEntryId = $state<string | null>(null);

	// Derived filtered entries
	let filteredEntries = $derived(
		entries
			.filter((e) => {
				if (selectedLevelFilter === 'All') return true;
				return e.level.toLowerCase() === selectedLevelFilter.toLowerCase();
			})
			.filter((e) => {
				const q = searchQuery.toLowerCase();
				return e.operation.toLowerCase().includes(q) || e.message.toLowerCase().includes(q);
			})
			.reverse() // show latest first
	);

	onMount(async () => {
		try {
			const tauriApi = await import('@tauri-apps/api/core');
			invoke = tauriApi.invoke;
			isTauri = true;
		} catch {
			console.warn('Tauri not available, using mock mode');
			isTauri = false;
		}

		await fetchLogs();
	});

	async function fetchLogs() {
		loading = true;
		error = '';
		try {
			if (isTauri && invoke) {
				const logs = (await invoke('audit_log_get_entries')) as AuditEntry[];
				entries = logs || [];
			} else {
				// Populate mock logs
				entries = [
					{
						id: '1',
						timestamp: '2026-07-08 19:40:11',
						level: 'Success',
						operation: 'detect_binaries',
						message: 'Binaries checked across PATH and SDK environments',
						details: 'Detected adb at /usr/bin/adb, fastboot at /usr/bin/fastboot, scrcpy at /usr/bin/scrcpy',
						duration_ms: 125,
						success: true
					},
					{
						id: '2',
						timestamp: '2026-07-08 19:41:05',
						level: 'Success',
						operation: 'list_packages',
						message: 'Listed user third-party packages',
						details: 'Device serial 23049PCD8G: returned 6 apps',
						duration_ms: 220,
						success: true
					},
					{
						id: '3',
						timestamp: '2026-07-08 19:43:52',
						level: 'Error',
						operation: 'install_package',
						message: 'Installation failed: INSTALL_FAILED_ALREADY_EXISTS',
						details: 'Command: adb install /home/downloads/chrome.apk\nReason: Package com.android.chrome already exists on device. Use replacement flag.',
						duration_ms: 850,
						success: false
					},
					{
						id: '4',
						timestamp: '2026-07-08 19:45:00',
						level: 'Info',
						operation: 'scrcpy_start',
						message: 'Mirroring session initialized',
						details: 'Executing: scrcpy -s 23049PCD8G --max-size 1024 --max-fps 60 --no-audio',
						duration_ms: 45,
						success: null
					}
				];
			}
		} catch (e) {
			error = `Failed to retrieve logs: ${e}`;
		} finally {
			loading = false;
		}
	}

	async function clearLogs() {
		if (!confirm('Are you sure you want to permanently clear the audit log history?')) return;
		loading = true;
		error = '';
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				await invoke('audit_log_clear');
			}
			entries = [];
			infoMessage = 'Audit history logs cleared successfully.';
		} catch (e) {
			error = `Failed to clear logs: ${e}`;
		} finally {
			loading = false;
		}
	}

	function toggleExpandRow(id: string) {
		if (expandedEntryId === id) {
			expandedEntryId = null;
		} else {
			expandedEntryId = id;
		}
	}

	function getLevelBadgeStyle(level: string): string {
		switch (level) {
			case 'Success':
				return 'bg-emerald-500/10 text-emerald-500 border-emerald-500/20';
			case 'Error':
				return 'bg-error/15 text-error border-error/30';
			case 'Warning':
				return 'bg-amber-500/10 text-amber-500 border-amber-500/25';
			case 'Info':
				return 'bg-primary/10 text-primary border-primary/20';
			default:
				return 'bg-surface-container-highest text-on-surface-variant border-outline-variant/30';
		}
	}
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2 h-screen overflow-hidden">
	<div
		class="flex flex-1 flex-col overflow-hidden rounded-[32px] bg-surface-container-low p-6 lg:p-10 relative"
	>
		<!-- Page Header -->
		<header class="mb-6 flex justify-between items-center shrink-0">
			<div class="flex items-center gap-4">
				<button
					onclick={() => goto('/')}
					class="flex h-10 w-10 items-center justify-center rounded-full text-on-surface-variant transition-colors hover:bg-surface-variant hover:text-on-surface"
					title="Back to dashboard"
				>
					<span class="material-symbols-outlined text-[22px]">arrow_back</span>
				</button>
				<h2 class="text-2xl font-bold tracking-tight text-on-surface flex items-center gap-4">
					Audit History Log
					{#if !isTauri}
						<span class="text-xs bg-error text-on-error px-3 py-1 rounded-full font-medium tracking-normal border border-error/30">MOCK MODE</span>
					{/if}
				</h2>
			</div>

			<div class="flex items-center gap-3">
				<button
					onclick={clearLogs}
					class="flex items-center gap-2 rounded-xl bg-error/10 text-error hover:bg-error/15 border border-error/25 transition-all text-xs font-bold py-1.5 px-4"
					disabled={entries.length === 0}
					title="Clear persistent logs"
				>
					<span class="material-symbols-outlined text-[16px]">delete_sweep</span>
					Clear Logs
				</button>
				<button
					onclick={fetchLogs}
					class="flex h-8 w-8 items-center justify-center rounded-full bg-surface-container-highest text-on-surface-variant hover:text-on-surface transition-all hover:scale-105 active:scale-95"
					title="Refresh audit entries"
				>
					<span class="material-symbols-outlined text-[18px] {loading ? 'animate-spin' : ''}">refresh</span>
				</button>
			</div>
		</header>

		<!-- Alert Messages -->
		{#if error}
			<div
				class="bg-error/15 text-error border border-error/30 p-4 rounded-2xl mb-4 font-medium flex items-center gap-3 text-sm shrink-0 animate-fade-in"
			>
				<span class="material-symbols-outlined text-[20px]">error</span>
				<div class="flex-1 break-words">{error}</div>
				<button onclick={() => (error = '')} class="hover:opacity-85 text-xs font-semibold uppercase">Dismiss</button>
			</div>
		{/if}

		{#if infoMessage}
			<div
				class="bg-primary/10 text-primary p-4 rounded-2xl mb-4 font-medium flex items-center gap-3 text-sm shrink-0 animate-fade-in"
			>
				<span class="material-symbols-outlined text-[20px]">check_circle</span>
				<div class="flex-1 break-words">{infoMessage}</div>
				<button onclick={() => (infoMessage = '')} class="hover:opacity-85 text-xs font-semibold uppercase">Dismiss</button>
			</div>
		{/if}

		<!-- Search and Filtering Toolbar -->
		<section class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6 shrink-0">
			<!-- Level Filter Tabs -->
			<div class="flex rounded-full bg-surface-container p-1 w-fit">
				{#each ['All', 'Success', 'Info', 'Warning', 'Error'] as level (level)}
					<button
						onclick={() => (selectedLevelFilter = level)}
						class="px-4 py-1.5 rounded-full text-xs font-semibold uppercase tracking-wider transition-all duration-200 {selectedLevelFilter === level ? 'bg-secondary-container text-on-secondary-container' : 'text-on-surface-variant hover:text-on-surface'}"
					>
						{level}
					</button>
				{/each}
			</div>

			<!-- Search input -->
			<div class="relative w-full sm:w-64">
				<span
					class="material-symbols-outlined absolute left-4 top-1/2 -translate-y-1/2 text-on-surface-variant text-[18px]"
					>search</span
				>
				<input
					type="text"
					bind:value={searchQuery}
					placeholder="Search operation / message..."
					class="bg-surface-container-high rounded-full pl-10 pr-4 py-2 text-sm text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all w-full border border-outline-variant/20"
				/>
			</div>
		</section>

		<!-- Scrollable Log Table container -->
		<div class="flex-1 rounded-[24px] bg-surface-container overflow-hidden flex flex-col min-h-0">
			<!-- Table Header -->
			<div class="grid grid-cols-12 gap-3 px-6 py-3 text-[10px] font-bold uppercase tracking-wider text-on-surface-variant shrink-0 bg-surface-container-high select-none">
				<div class="col-span-2">Level</div>
				<div class="col-span-3">Timestamp</div>
				<div class="col-span-3">Operation</div>
				<div class="col-span-3">Summary Message</div>
				<div class="col-span-1 text-right">Time</div>
			</div>

			<!-- Table Body Rows -->
			<div class="flex-1 overflow-y-auto min-h-0 select-text">
				{#if loading && entries.length === 0}
					<div class="flex flex-col gap-2 p-4">
						{#each Array(6) as dummy, i (i)}
							<div class="h-10 bg-surface-container-high rounded-xl animate-pulse" id="dummy-audit-row-{i}-{dummy || ''}"></div>
						{/each}
					</div>
				{:else if filteredEntries.length === 0}
					<div class="flex flex-col items-center justify-center p-12 text-center h-full">
						<span class="material-symbols-outlined text-[48px] text-on-surface-variant opacity-60 mb-3">history</span>
						<p class="text-sm text-on-surface-variant font-medium">No audit logs found matching criteria</p>
					</div>
				{:else}
					<div class="flex flex-col">
						{#each filteredEntries as entry (entry.id)}
							<!-- Row container -->
							<div
								onclick={() => toggleExpandRow(entry.id)}
								class="grid grid-cols-12 gap-3 px-6 py-3 items-center hover:bg-surface-container-high cursor-pointer transition-colors text-xs font-semibold text-on-surface {expandedEntryId === entry.id ? 'bg-surface-container-high' : ''}"
							>
								<!-- Level badge -->
								<div class="col-span-2">
									<span class="px-2 py-0.5 rounded text-[10px] font-bold tracking-wide uppercase border {getLevelBadgeStyle(entry.level)}">
										{entry.level}
									</span>
								</div>

								<!-- Timestamp -->
								<div class="col-span-3 font-mono text-[11px] opacity-80">{entry.timestamp}</div>

								<!-- Operation -->
								<div class="col-span-3 font-mono text-[11px] text-primary break-all pr-2">{entry.operation}</div>

								<!-- Message -->
								<div class="col-span-3 truncate pr-2">{entry.message}</div>

								<!-- Duration -->
								<div class="col-span-1 text-right font-mono text-[11px] opacity-85">
									{entry.duration_ms !== null && entry.duration_ms !== undefined ? `${entry.duration_ms}ms` : '—'}
								</div>
							</div>

							<!-- Expanded Details Pane -->
							{#if expandedEntryId === entry.id && entry.details}
								<div class="bg-black/95 text-green-400 font-mono text-[10px] px-6 py-4 flex flex-col gap-2 animate-fade-in leading-normal select-all">
									<div class="flex justify-between items-center pb-1 border-b border-green-950 text-gray-500 select-none text-[8px] uppercase tracking-wider">
										<span>Command Exec Execution Details</span>
										<span>Log ID: {entry.id}</span>
									</div>
									<pre class="whitespace-pre-wrap max-h-48 overflow-y-auto">{entry.details}</pre>
								</div>
							{/if}
						{/each}
					</div>
				{/if}
			</div>
		</div>
	</div>
</main>

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
