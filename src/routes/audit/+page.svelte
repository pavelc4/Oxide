<script lang="ts">
	/* eslint-disable svelte/no-navigation-without-resolve */
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | undefined;
	let isTauri = $state(false);

	interface AuditEntry {
		id: string;
		timestamp: string;
		operation: string;
		level: 'Info' | 'Warning' | 'Error' | 'Success';
		message: string;
		details: string | null;
		duration_ms: number | null;
	}

	let entries = $state<AuditEntry[]>([]);
	let loading = $state(true);
	let error = $state('');
	let infoMessage = $state('');

	// Filters state
	let filterLevel = $state<string>('All');
	let searchQuery = $state<string>('');
	let expandedEntryId = $state<string | null>(null);

	const mockAuditEntries: AuditEntry[] = [
		{
			id: 'audit-001',
			timestamp: '2026-07-18 21:04:12',
			operation: 'fastboot_flash_partition',
			level: 'Success',
			message: 'Flashed boot.img partition to slot A',
			details: 'target: boot_a\nfile: /tmp/boot.img (67,108,864 bytes)\ntime: 1.42s\nstatus: OKAY',
			duration_ms: 1420
		},
		{
			id: 'audit-002',
			timestamp: '2026-07-18 21:02:45',
			operation: 'install_app_cmd',
			level: 'Info',
			message: 'Installed com.example.app via ADB',
			details: 'pkg: com.example.app\napk: /downloads/base.apk\nresult: Success',
			duration_ms: 2840
		},
		{
			id: 'audit-003',
			timestamp: '2026-07-18 20:58:10',
			operation: 'adb_sideload_cmd',
			level: 'Error',
			message: 'Sideload failed: device unauthorized',
			details: 'error: device disconnected or failed handshake during sideload transmission',
			duration_ms: 850
		},
		{
			id: 'audit-004',
			timestamp: '2026-07-18 20:45:00',
			operation: 'fastboot_wipe_userdata',
			level: 'Warning',
			message: 'Executed factory reset wipe userdata',
			details: 'wiped /data & /cache partitions',
			duration_ms: 3100
		}
	];

	async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
		if (isTauri && invoke) {
			return (await invoke(cmd, args)) as T;
		}
		throw new Error('Tauri API not active');
	}

	onMount(async () => {
		try {
			if (typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window)) {
				const tauriApi = await import('@tauri-apps/api/core');
				invoke = tauriApi.invoke;
				isTauri = true;
			} else {
				isTauri = false;
			}
		} catch {
			isTauri = false;
		}

		await fetchLogs();
	});

	async function fetchLogs() {
		loading = true;
		error = '';
		try {
			if (isTauri && invoke) {
				const rawEntries = await safeInvoke<{timestamp: string; operation: string; serial?: string | null; details: string}[]>('get_audit_log');
				entries = rawEntries && rawEntries.length > 0
					? rawEntries.map((e, i) => ({
						id: `audit-${i}`,
						timestamp: e.timestamp,
						operation: e.operation,
						level: 'Info' as AuditEntry['level'],
						message: e.operation,
						details: e.details,
						duration_ms: null
					}))
					: [];
			} else {
				entries = mockAuditEntries;
			}
		} catch (e) {
			console.warn('Failed to fetch audit logs:', e);
			entries = [];
		} finally {
			loading = false;
		}
	}

	async function clearLogs() {
		if (!confirm('Are you sure you want to clear all persistent audit logs?')) return;
		loading = true;
		try {
			if (isTauri && invoke) {
				await safeInvoke('clear_audit_log');
			}
			entries = [];
			infoMessage = 'Audit log history cleared successfully.';
		} catch (e) {
			error = `Failed to clear audit logs: ${e}`;
		} finally {
			loading = false;
		}
	}

	function toggleExpandRow(id: string) {
		expandedEntryId = expandedEntryId === id ? null : id;
	}

	let filteredEntries = $derived(
		entries.filter((entry) => {
			const matchesLevel = filterLevel === 'All' || entry.level.toLowerCase() === filterLevel.toLowerCase();
			const search = searchQuery.toLowerCase();
			const matchesSearch =
				!searchQuery ||
				entry.operation.toLowerCase().includes(search) ||
				entry.message.toLowerCase().includes(search) ||
				(entry.details && entry.details.toLowerCase().includes(search));
			return matchesLevel && matchesSearch;
		})
	);

	function getLevelBadgeStyle(level: string): string {
		switch (level) {
			case 'Success':
				return 'bg-emerald-500/15 text-emerald-400';
			case 'Error':
				return 'bg-error/15 text-error';
			case 'Warning':
				return 'bg-amber-500/15 text-amber-400';
			case 'Info':
				return 'bg-primary/15 text-primary';
			default:
				return 'bg-surface-container-highest text-on-surface-variant';
		}
	}
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2 h-screen overflow-hidden">
	<div class="flex flex-1 flex-col overflow-hidden rounded-[32px] bg-surface-container-low p-6 lg:p-8 relative gap-5 shadow-sm">
		
		<!-- Page Header -->
		<header class="flex items-center justify-between shrink-0 pb-2">
			<div class="flex items-center gap-4">
				<button
					onclick={() => goto('/')}
					class="flex h-10 w-10 items-center justify-center rounded-xl bg-surface-container hover:bg-surface-container-high text-on-surface-variant transition-all hover:scale-105 active:scale-95 shadow-xs"
					title="Back to dashboard"
				>
					<span class="material-symbols-outlined text-[20px]">arrow_back</span>
				</button>
				<div>
					<div class="flex items-center gap-3">
						<h2 class="text-2xl font-bold tracking-tight text-on-surface">Audit History Log</h2>
						{#if !isTauri}
							<span class="text-[10px] bg-warning/15 text-warning px-2.5 py-0.5 rounded-full font-bold uppercase tracking-wider">MOCK MODE</span>
						{/if}
					</div>
					<p class="text-xs text-on-surface-variant/80 font-medium mt-0.5">Persistent operational audit log history</p>
				</div>
			</div>

			<div class="flex items-center gap-3">
				<button
					onclick={clearLogs}
					class="flex items-center gap-2 rounded-full bg-error/15 text-error hover:bg-error/25 transition-all text-xs font-bold py-2 px-4 shadow-xs"
					disabled={entries.length === 0}
					title="Clear persistent logs"
				>
					<span class="material-symbols-outlined text-[16px]">delete_sweep</span>
					Clear Logs
				</button>
				<button
					onclick={fetchLogs}
					class="flex h-9 w-9 items-center justify-center rounded-full bg-surface-container hover:bg-surface-container-high text-on-surface-variant transition-all hover:scale-105 active:scale-95 shadow-xs"
					title="Refresh audit entries"
				>
					<span class="material-symbols-outlined text-[18px] {loading ? 'animate-spin' : ''}">refresh</span>
				</button>
			</div>
		</header>

		<!-- Alert Messages -->
		{#if error}
			<div class="bg-error/15 text-error p-3.5 rounded-2xl font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[18px]">error</span>
				<div class="flex-1 font-semibold">{error}</div>
				<button onclick={() => (error = '')} class="hover:opacity-80 text-[10px] font-bold uppercase">Dismiss</button>
			</div>
		{/if}

		{#if infoMessage}
			<div class="bg-primary/10 text-primary p-3.5 rounded-2xl font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[18px]">check_circle</span>
				<div class="flex-1 font-semibold">{infoMessage}</div>
				<button onclick={() => (infoMessage = '')} class="hover:opacity-80 text-[10px] font-bold uppercase">Dismiss</button>
			</div>
		{/if}

		<!-- Filter Bar -->
		<section class="flex flex-col sm:flex-row items-center justify-between gap-4 shrink-0">
			<!-- Level Filter Pills -->
			<div class="flex rounded-full bg-surface-container p-1 shadow-xs">
				{#each ['All', 'Info', 'Success', 'Warning', 'Error'] as level}
					<button
						onclick={() => (filterLevel = level)}
						class="px-4 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider transition-all {filterLevel === level ? 'bg-secondary-container text-on-secondary-container shadow-xs' : 'text-on-surface-variant hover:text-on-surface'}"
					>
						{level}
					</button>
				{/each}
			</div>

			<!-- Search input -->
			<div class="relative w-full sm:w-72">
				<span class="material-symbols-outlined absolute left-3.5 top-1/2 -translate-y-1/2 text-on-surface-variant text-[18px]">search</span>
				<input
					type="text"
					bind:value={searchQuery}
					placeholder="Search operation / message..."
					class="bg-surface-container rounded-full pl-10 pr-4 py-2 text-xs font-medium text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 transition-all w-full shadow-xs"
				/>
			</div>
		</section>

		<!-- Scrollable Log Table Container (Zero Borders) -->
		<div class="flex-1 rounded-[32px] bg-surface-container overflow-hidden flex flex-col min-h-0 shadow-sm">
			<!-- Table Header -->
			<div class="grid grid-cols-12 gap-3 px-6 py-3.5 text-[10px] font-bold uppercase tracking-wider text-on-surface-variant shrink-0 bg-surface-container-high select-none">
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
					<div class="flex flex-col items-center justify-center p-12 text-center h-full text-on-surface-variant">
						<span class="material-symbols-outlined text-[48px] opacity-40 mb-3">history</span>
						<p class="text-xs font-semibold">No audit logs found matching criteria</p>
					</div>
				{:else}
					<div class="flex flex-col">
						{#each filteredEntries as entry (entry.id)}
							<div
								role="button"
								tabindex="0"
								onclick={() => toggleExpandRow(entry.id)}
								onkeydown={(e) => { if (e.key === 'Enter') toggleExpandRow(entry.id); }}
								class="grid grid-cols-12 gap-3 px-6 py-3.5 items-center hover:bg-surface-container-high cursor-pointer transition-colors text-xs font-semibold text-on-surface {expandedEntryId === entry.id ? 'bg-surface-container-high' : ''}"
							>
								<!-- Level badge -->
								<div class="col-span-2">
									<span class="px-2.5 py-1 rounded-full text-[10px] font-bold tracking-wide uppercase {getLevelBadgeStyle(entry.level)}">
										{entry.level}
									</span>
								</div>

								<!-- Timestamp -->
								<div class="col-span-3 font-mono text-[11px] text-on-surface-variant">{entry.timestamp}</div>

								<!-- Operation -->
								<div class="col-span-3 font-mono text-[11px] text-primary break-all pr-2">{entry.operation}</div>

								<!-- Message -->
								<div class="col-span-3 truncate pr-2">{entry.message}</div>

								<!-- Duration -->
								<div class="col-span-1 text-right font-mono text-[11px] text-on-surface-variant">
									{entry.duration_ms !== null && entry.duration_ms !== undefined ? `${entry.duration_ms}ms` : '—'}
								</div>
							</div>

							<!-- Expanded Details Pane -->
							{#if expandedEntryId === entry.id && entry.details}
								<div class="bg-surface-container-highest p-4 px-6 font-mono text-xs text-on-surface flex flex-col gap-2 animate-fade-in leading-relaxed select-all">
									<div class="flex justify-between items-center text-[10px] text-on-surface-variant/70 uppercase font-mono tracking-wider">
										<span>Command Exec Execution Details</span>
										<span>Log ID: {entry.id}</span>
									</div>
									<pre class="whitespace-pre-wrap max-h-48 overflow-y-auto m-0 bg-surface-container-high p-3 rounded-2xl">{entry.details}</pre>
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
		from { opacity: 0; transform: scale(0.99); }
		to { opacity: 1; transform: scale(1); }
	}
	.animate-fade-in { animation: fadeIn 0.15s cubic-bezier(0.2, 0.8, 0.2, 1) forwards; }
</style>
