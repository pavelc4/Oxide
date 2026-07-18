<script lang="ts">
	/* eslint-disable svelte/no-navigation-without-resolve */
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | undefined;
	let isTauri = $state(false);

	interface AppConfig {
		adb_path: string | null;
		fastboot_path: string | null;
		theme: string | null;
		audit_enabled: boolean;
		setup_completed: boolean;
	}

	// State
	let loading = $state(true);
	let saving = $state(false);
	let error = $state('');
	let infoMessage = $state('');

	// Config Values
	let adbPath = $state('');
	let fastbootPath = $state('');
	let currentTheme = $state('dark');
	let auditEnabled = $state(false);
	let setupCompleted = $state(true);
	let confirmDangerousActions = $state(true);

	// Theme Color Seeds
	let selectedSeedColor = $state('#ffb784');
	const themeSeeds = [
		{ name: 'Sunset Amber', hex: '#ffb784' },
		{ name: 'Neon Emerald', hex: '#34d399' },
		{ name: 'Cyber Cyan', hex: '#38bdf8' },
		{ name: 'Deep Violet', hex: '#c084fc' },
		{ name: 'Crimson Flame', hex: '#f43f5e' }
	];

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

		await loadSettings();
	});

	async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
		if (isTauri && invoke) {
			return (await invoke(cmd, args)) as T;
		}
		throw new Error('Tauri API not active');
	}

	async function loadSettings() {
		loading = true;
		error = '';
		try {
			if (isTauri && invoke) {
				const cfg = await safeInvoke<AppConfig>('get_config');
				if (cfg) {
					adbPath = cfg.adb_path || '';
					fastbootPath = cfg.fastboot_path || '';
					currentTheme = cfg.theme || 'dark';
					auditEnabled = cfg.audit_enabled || false;
					setupCompleted = cfg.setup_completed ?? true;
				}
			} else {
				adbPath = '/usr/bin/adb';
				fastbootPath = '/usr/bin/fastboot';
				currentTheme = 'dark';
				auditEnabled = true;
			}
		} catch (e) {
			console.warn('Could not load config:', e);
		} finally {
			loading = false;
		}
	}

	async function saveSettings() {
		saving = true;
		error = '';
		infoMessage = '';

		const configData: AppConfig = {
			adb_path: adbPath.trim() || null,
			fastboot_path: fastbootPath.trim() || null,
			theme: currentTheme,
			audit_enabled: auditEnabled,
			setup_completed: setupCompleted
		};

		try {
			if (isTauri && invoke) {
				await safeInvoke('save_config', { config: configData });
			} else {
				await new Promise((resolve) => setTimeout(resolve, 600));
			}
			infoMessage = 'Application settings saved successfully!';
		} catch (e) {
			error = `Failed to save settings: ${e}`;
		} finally {
			saving = false;
		}
	}

	async function applyThemeColor(colorHex: string) {
		selectedSeedColor = colorHex;
		infoMessage = '';
		try {
			if (isTauri && invoke) {
				await safeInvoke('generate_theme', { hexColor: colorHex });
			}
			infoMessage = `Matugen palette generated for accent color ${colorHex}`;
		} catch {
			infoMessage = `Selected color accent: ${colorHex}`;
		}
	}

	function resetToDefaults() {
		if (!confirm('Reset all settings to factory default?')) return;
		adbPath = '';
		fastbootPath = '';
		currentTheme = 'dark';
		auditEnabled = false;
		confirmDangerousActions = true;
		saveSettings();
	}
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2 h-screen overflow-hidden">
	<div class="flex flex-1 flex-col overflow-hidden rounded-[32px] bg-surface-container-low p-6 lg:p-8 relative gap-6 w-full shadow-sm">

		<!-- Header -->
		<header class="flex items-center justify-between gap-4 shrink-0 pb-2">
			<div class="flex items-center gap-4">
				<button
					onclick={() => goto('/')}
					class="flex h-10 w-10 items-center justify-center rounded-full bg-surface-container hover:bg-surface-container-high text-on-surface-variant transition-all hover:scale-105 active:scale-95 shrink-0"
					title="Back to dashboard"
				>
					<span class="material-symbols-outlined text-[20px]">arrow_back</span>
				</button>
				<div>
					<div class="flex items-center gap-3">
						<h2 class="text-2xl font-bold tracking-tight text-on-surface">Oxide Settings</h2>
						{#if !isTauri}
							<span class="text-[10px] bg-warning/15 text-warning px-2.5 py-0.5 rounded-full font-bold uppercase tracking-wider">MOCK PREVIEW</span>
						{/if}
					</div>
					<p class="text-xs text-on-surface-variant/80 font-medium mt-0.5">Configure system binaries, Material themes, audit logging & preferences</p>
				</div>
			</div>

			<div class="flex items-center gap-2">
				<button
					onclick={saveSettings}
					class="flex items-center gap-2 rounded-xl bg-primary text-on-primary hover:brightness-110 px-5 py-2.5 text-xs font-bold transition-all shadow-sm disabled:opacity-50"
					disabled={saving}
				>
					<span class="material-symbols-outlined text-[18px]">save</span>
					{#if saving}
						Saving...
					{:else}
						Save Changes
					{/if}
				</button>
			</div>
		</header>

		<!-- Alert Messages -->
		{#if error}
			<div class="bg-error/15 text-error p-3.5 rounded-2xl font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[20px]">error</span>
				<div class="flex-1 break-words font-semibold">{error}</div>
				<button onclick={() => (error = '')} class="hover:opacity-80 text-[10px] font-bold uppercase tracking-wider bg-error/20 px-2.5 py-1 rounded-lg">Dismiss</button>
			</div>
		{/if}

		{#if infoMessage}
			<div class="bg-primary/10 text-primary p-3.5 rounded-2xl font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[20px]">check_circle</span>
				<div class="flex-1 break-words font-semibold">{infoMessage}</div>
				<button onclick={() => (infoMessage = '')} class="hover:opacity-80 text-[10px] font-bold uppercase tracking-wider bg-primary/20 px-2.5 py-1 rounded-lg">Dismiss</button>
			</div>
		{/if}

		<!-- Settings Grid -->
		<div class="flex-1 grid grid-cols-1 lg:grid-cols-2 gap-6 overflow-y-auto pr-1">

			<!-- Section 1: Appearance & Theme (Matugen Color Generator) -->
			<section class="rounded-[24px] bg-surface-container p-6 flex flex-col gap-5 shadow-sm">
				<div class="flex items-center gap-3">
					<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary-container text-on-primary-container">
						<span class="material-symbols-outlined text-[20px]">palette</span>
					</div>
					<div>
						<h3 class="text-sm font-bold text-on-surface">Appearance & Theme Engine</h3>
						<p class="text-xs text-on-surface-variant">Customize UI theme and Matugen dynamic color palettes</p>
					</div>
				</div>

				<!-- Theme Mode Selector -->
				<div class="flex flex-col gap-2">
					<span class="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant">Base Theme Mode</span>
					<div class="grid grid-cols-3 gap-2">
						{#each [{ id: 'dark', label: 'Dark Mode', icon: 'dark_mode' }, { id: 'light', label: 'Light Mode', icon: 'light_mode' }, { id: 'auto', label: 'System Auto', icon: 'settings_brightness' }] as mode}
							<button
								onclick={() => {
									currentTheme = mode.id;
									document.documentElement.setAttribute('data-theme', mode.id);
								}}
								class="flex items-center justify-center gap-2 p-3 rounded-xl text-xs font-bold transition-all {currentTheme === mode.id ? 'bg-primary text-on-primary shadow-sm' : 'bg-surface-container-high text-on-surface-variant hover:bg-surface-container-highest hover:text-on-surface'}"
							>
								<span class="material-symbols-outlined text-[18px]">{mode.icon}</span>
								{mode.label}
							</button>
						{/each}
					</div>
				</div>

				<!-- Matugen Seed Colors -->
				<div class="flex flex-col gap-2">
					<span class="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant">Matugen Color Seed Generator</span>
					<div class="flex flex-wrap gap-3">
						{#each themeSeeds as seed}
							<button
								onclick={() => applyThemeColor(seed.hex)}
								class="flex items-center gap-2 px-3.5 py-2 rounded-xl text-xs font-semibold bg-surface-container-high hover:bg-surface-container-highest transition-all group {selectedSeedColor === seed.hex ? 'ring-2 ring-primary' : ''}"
							>
								<span class="w-4 h-4 rounded-full shadow-xs" style="background-color: {seed.hex}"></span>
								<span class="text-on-surface group-hover:text-primary transition-colors">{seed.name}</span>
							</button>
						{/each}
					</div>
				</div>
			</section>

			<!-- Section 2: Binary Paths & Protocol Settings -->
			<section class="rounded-[24px] bg-surface-container p-6 flex flex-col gap-5 shadow-sm">
				<div class="flex items-center gap-3">
					<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary-container text-on-primary-container">
						<span class="material-symbols-outlined text-[20px]">terminal</span>
					</div>
					<div>
						<h3 class="text-sm font-bold text-on-surface">Executable Binary Paths</h3>
						<p class="text-xs text-on-surface-variant">Configure custom binary paths or use default environment daemon</p>
					</div>
				</div>

				<div class="flex flex-col gap-4">
					<div class="flex flex-col gap-1.5">
						<label for="adb-path-input" class="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant">Custom ADB Path</label>
						<input
							id="adb-path-input"
							type="text"
							bind:value={adbPath}
							placeholder="Leave empty for default (127.0.0.1:5037 / PATH)"
							class="bg-surface-container-high rounded-xl px-3.5 py-2.5 text-xs font-mono text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40"
						/>
					</div>

					<div class="flex flex-col gap-1.5">
						<label for="fastboot-path-input" class="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant">Custom Fastboot Path</label>
						<input
							id="fastboot-path-input"
							type="text"
							bind:value={fastbootPath}
							placeholder="Leave empty for default PATH binary"
							class="bg-surface-container-high rounded-xl px-3.5 py-2.5 text-xs font-mono text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40"
						/>
					</div>
				</div>
			</section>

			<!-- Section 3: Audit Logging & Safety Controls -->
			<section class="rounded-[24px] bg-surface-container p-6 flex flex-col gap-5 shadow-sm">
				<div class="flex items-center gap-3">
					<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary-container text-on-primary-container">
						<span class="material-symbols-outlined text-[20px]">verified_user</span>
					</div>
					<div>
						<h3 class="text-sm font-bold text-on-surface">Security & Audit Controls</h3>
						<p class="text-xs text-on-surface-variant">Manage persistent operation logs and dangerous action prompts</p>
					</div>
				</div>

				<div class="flex flex-col gap-3">
					<label class="flex items-center justify-between p-3.5 rounded-2xl bg-surface-container-high/50 hover:bg-surface-container-high transition-all cursor-pointer">
						<div>
							<span class="text-xs font-bold text-on-surface block">Enable Persistent Operation Audit Log</span>
							<span class="text-[11px] text-on-surface-variant">Save execution history of ADB shell, package, and fastboot tasks to JSON log</span>
						</div>
						<input
							type="checkbox"
							bind:checked={auditEnabled}
							class="w-5 h-5 rounded text-primary focus:ring-primary/50"
						/>
					</label>

					<label class="flex items-center justify-between p-3.5 rounded-2xl bg-surface-container-high/50 hover:bg-surface-container-high transition-all cursor-pointer">
						<div>
							<span class="text-xs font-bold text-on-surface block">Dangerous Action Confirmation Prompts</span>
							<span class="text-[11px] text-on-surface-variant">Require confirmation dialogs before factory reset, wiping, or partition flashing</span>
						</div>
						<input
							type="checkbox"
							bind:checked={confirmDangerousActions}
							class="w-5 h-5 rounded text-primary focus:ring-primary/50"
						/>
					</label>
				</div>
			</section>

			<!-- Section 4: Maintenance & System Info -->
			<section class="rounded-[24px] bg-surface-container p-6 flex flex-col justify-between gap-5 shadow-sm">
				<div class="flex flex-col gap-4">
					<div class="flex items-center gap-3">
						<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary-container text-on-primary-container">
							<span class="material-symbols-outlined text-[20px]">info</span>
						</div>
						<div>
							<h3 class="text-sm font-bold text-on-surface">System & Framework Info</h3>
							<p class="text-xs text-on-surface-variant">Core stack versions and platform data path</p>
						</div>
					</div>

					<div class="flex flex-col gap-2 text-xs">
						<div class="flex justify-between py-1.5">
							<span class="text-on-surface-variant">Application Version</span>
							<span class="font-bold text-on-surface">Oxide v0.1.0</span>
						</div>
						<div class="flex justify-between py-1.5">
							<span class="text-on-surface-variant">Frontend Framework</span>
							<span class="font-mono text-on-surface">Svelte 5 + Vite 6</span>
						</div>
						<div class="flex justify-between py-1.5">
							<span class="text-on-surface-variant">Backend Core</span>
							<span class="font-mono text-on-surface">Rust (Tauri 2 + adb_client)</span>
						</div>
					</div>
				</div>

				<div class="flex gap-3 pt-2">
					<button
						onclick={resetToDefaults}
						class="flex-1 py-2.5 px-4 rounded-xl bg-error/15 text-error hover:bg-error/25 text-xs font-bold transition-all"
					>
						Reset to Defaults
					</button>
					<button
						onclick={saveSettings}
						class="flex-1 py-2.5 px-4 rounded-xl bg-primary text-on-primary hover:brightness-110 text-xs font-bold transition-all shadow-sm"
					>
						Save All Settings
					</button>
				</div>
			</section>

		</div>
	</div>
</main>

<style>
	@keyframes fadeIn {
		from {
			opacity: 0;
			transform: scale(0.99);
		}
		to {
			opacity: 1;
			transform: scale(1);
		}
	}
	.animate-fade-in {
		animation: fadeIn 0.15s cubic-bezier(0.2, 0.8, 0.2, 1) forwards;
	}
</style>
