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

	interface GitHubContributor {
		login: string;
		avatar_url: string;
		html_url: string;
		contributions: number;
	}

	// State
	let loading = $state(true);
	let saving = $state(false);
	let error = $state('');
	let infoMessage = $state('');

	// Config Values
	let currentTheme = $state('dark');
	let auditEnabled = $state(true);
	let setupCompleted = $state(true);
	let confirmDangerousActions = $state(true);

	// Expressive Customization States
	let cornerRadius = $state<'expressive' | 'medium' | 'compact'>('expressive');
	let autoWirelessAdb = $state(true);
	let logcatBufferLimit = $state<'1000' | '5000' | '10000' | 'unlimited'>('5000');
	let adbDaemonPort = $state('5037');

	// GitHub Contributors (fetched & cached)
	let contributors = $state<GitHubContributor[]>([]);
	let showAllContributors = $state(false);
	let contributorsLoading = $state(true);

	// Theme Color Seeds
	let selectedSeedColor = $state('#ffb784');
	interface ThemeSeed {
		name: string;
		hex: string;
		colors: [string, string, string, string];
	}

	const themeSeeds: ThemeSeed[] = [
		{ name: 'Peach Amber', hex: '#ffb784', colors: ['#ffb784', '#ffdcc6', '#534338', '#7fd7be'] },
		{ name: 'Rose Coral', hex: '#f43f5e', colors: ['#ffb3b8', '#ffdadc', '#534344', '#7fd7c0'] },
		{ name: 'Neon Emerald', hex: '#34d399', colors: ['#68dcac', '#bfece0', '#384c46', '#a8c8ff'] },
		{ name: 'Cyber Cyan', hex: '#38bdf8', colors: ['#9acbfa', '#d6e3f7', '#3a4856', '#e9b9d6'] },
		{ name: 'Deep Violet', hex: '#c084fc', colors: ['#d8b9ff', '#eedcff', '#494254', '#ffb784'] },
		{ name: 'Pastel Lilac', hex: '#f472b6', colors: ['#f4b9e2', '#fde8f5', '#4e4049', '#ffe08a'] }
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
		await loadContributors();
	});

	async function loadContributors() {
		contributorsLoading = true;
		const cacheKey = 'oxide_contributors_v2';
		const cacheExpiry = 'oxide_contributors_v2_ts';
		const CACHE_TTL = 1000 * 60 * 60; // 1 hour

		try {
			// Try loading from localStorage cache first
			if (typeof localStorage !== 'undefined') {
				const cached = localStorage.getItem(cacheKey);
				const cachedTs = localStorage.getItem(cacheExpiry);
				if (cached && cachedTs) {
					const age = Date.now() - parseInt(cachedTs);
					if (age < CACHE_TTL) {
						contributors = JSON.parse(cached);
						contributorsLoading = false;
						return;
					}
				}
			}

			const res = await fetch('https://api.github.com/repos/pavelc4/Oxide/contributors?per_page=20');
			if (res.ok) {
				const data: GitHubContributor[] = await res.json();
				contributors = data.map((c) => ({
					login: c.login,
					avatar_url: c.avatar_url,
					html_url: c.html_url,
					contributions: c.contributions
				}));
				if (typeof localStorage !== 'undefined') {
					localStorage.setItem(cacheKey, JSON.stringify(contributors));
					localStorage.setItem(cacheExpiry, Date.now().toString());
				}
			}
		} catch {
			// Fallback: at least show the owner
			if (contributors.length === 0) {
				contributors = [{
					login: 'pavelc4',
					avatar_url: 'https://github.com/pavelc4.png',
					html_url: 'https://github.com/pavelc4',
					contributions: 0
				}];
			}
		} finally {
			contributorsLoading = false;
		}
	}

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
					currentTheme = cfg.theme || 'dark';
					auditEnabled = cfg.audit_enabled ?? true;
					setupCompleted = cfg.setup_completed ?? true;
				}
			} else {
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
			adb_path: null,
			fastboot_path: null,
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
								const hex = colorHex.replace('#', '');
				const argb = parseInt(hex, 16) | 0xff000000;
				await safeInvoke('generate_theme', { argb });
			}
			infoMessage = `Matugen palette generated for accent color ${colorHex}`;
		} catch {
			infoMessage = `Selected color accent: ${colorHex}`;
		}
	}

	function resetToDefaults() {
		if (!confirm('Reset all settings to factory default?')) return;
		currentTheme = 'dark';
		auditEnabled = true;
		confirmDangerousActions = true;
		cornerRadius = 'expressive';
		autoWirelessAdb = true;
		logcatBufferLimit = '5000';
		adbDaemonPort = '5037';
		saveSettings();
	}

	// Derived: lead contributor is first, others are the rest
	let leadContributor = $derived(contributors[0] ?? null);
	let otherContributors = $derived(contributors.slice(1));
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2 h-screen overflow-hidden">
	<div class="flex flex-1 flex-col overflow-hidden rounded-[32px] bg-surface-container-low p-6 lg:p-8 relative gap-5 w-full shadow-sm">

		<!-- Header -->
		<header class="flex items-center justify-between gap-4 shrink-0">
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
						<h2 class="text-2xl font-bold tracking-tight text-on-surface">Settings</h2>
						{#if !isTauri}
							<span class="text-[10px] bg-warning/15 text-warning px-2.5 py-0.5 rounded-full font-bold uppercase tracking-wider">MOCK PREVIEW</span>
						{/if}
					</div>
				</div>
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

		<!-- 2-Column Hybrid: Settings Left, About Right -->
		<div class="flex-1 overflow-y-auto pr-1 grid grid-cols-1 lg:grid-cols-[1fr_380px] gap-5 items-start">

			<!-- ═══ Left Column: Settings Stack ═══ -->
			<div class="flex flex-col gap-5">

			<!-- ═══ Appearance & Theme ═══ -->
			<section class="rounded-[28px] bg-surface-container p-6 flex flex-col gap-5 shadow-sm">
				<div class="flex items-center gap-3">
					<div class="flex h-10 w-10 items-center justify-center rounded-2xl bg-primary-container text-on-primary-container">
						<span class="material-symbols-outlined text-[20px]">palette</span>
					</div>
					<div>
						<h3 class="text-sm font-bold text-on-surface">Appearance</h3>
						<p class="text-xs text-on-surface-variant">Theme mode, corner shapes, and Material You colors</p>
					</div>
				</div>

				<!-- Theme Mode -->
				<div class="flex flex-col gap-2">
					<span class="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant">Theme Mode</span>
					<div class="grid grid-cols-3 gap-2">
						{#each [{ id: 'dark', label: 'Dark', icon: 'dark_mode' }, { id: 'light', label: 'Light', icon: 'light_mode' }, { id: 'auto', label: 'System', icon: 'settings_brightness' }] as mode}
							<button
								onclick={() => {
									currentTheme = mode.id;
									document.documentElement.setAttribute('data-theme', mode.id);
								}}
								class="flex items-center justify-center gap-2 p-3 rounded-2xl text-xs font-bold transition-all {currentTheme === mode.id ? 'bg-primary text-on-primary shadow-sm' : 'bg-surface-container-high text-on-surface-variant hover:bg-surface-container-highest hover:text-on-surface'}"
							>
								<span class="material-symbols-outlined text-[18px]">{mode.icon}</span>
								{mode.label}
							</button>
						{/each}
					</div>
				</div>

				<!-- Corner Radius -->
				<div class="flex flex-col gap-2">
					<span class="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant">Corner Radius</span>
					<div class="grid grid-cols-3 gap-2">
						{#each [{ id: 'expressive', label: 'Round (32px)', icon: 'rounded_corner' }, { id: 'medium', label: 'Medium (20px)', icon: 'crop_square' }, { id: 'compact', label: 'Sharp (12px)', icon: 'square' }] as shape}
							<button
								onclick={() => (cornerRadius = shape.id as 'expressive' | 'medium' | 'compact')}
								class="flex flex-col items-center gap-1 p-3 rounded-2xl text-xs font-bold transition-all {cornerRadius === shape.id ? 'bg-primary text-on-primary shadow-sm' : 'bg-surface-container-high text-on-surface-variant hover:bg-surface-container-highest'}"
							>
								<span class="material-symbols-outlined text-[18px]">{shape.icon}</span>
								{shape.label}
							</button>
						{/each}
					</div>
				</div>

				<!-- Theme Color Swatches -->
				<div class="flex flex-col gap-2">
					<span class="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant">Theme Color</span>
					<div class="flex items-center gap-4 py-1 overflow-x-auto">
						{#each themeSeeds as seed}
							<button
								onclick={() => applyThemeColor(seed.hex)}
								class="group relative flex flex-col items-center gap-1.5 focus:outline-none shrink-0"
								title={seed.name}
							>
								<div class="relative w-12 h-12 rounded-full overflow-hidden flex flex-wrap transition-transform duration-200 group-hover:scale-110 active:scale-95 shadow-md cursor-pointer {selectedSeedColor === seed.hex ? 'ring-[3px] ring-primary/50' : ''}">
									<div class="w-1/2 h-1/2" style="background-color: {seed.colors[0]}"></div>
									<div class="w-1/2 h-1/2" style="background-color: {seed.colors[1]}"></div>
									<div class="w-1/2 h-1/2" style="background-color: {seed.colors[2]}"></div>
									<div class="w-1/2 h-1/2" style="background-color: {seed.colors[3]}"></div>
									{#if selectedSeedColor === seed.hex}
										<div class="absolute inset-0 flex items-center justify-center bg-black/25 backdrop-blur-[1px]">
											<span class="material-symbols-outlined text-white text-[20px] font-bold">check</span>
										</div>
									{/if}
								</div>
								<span class="text-[10px] font-semibold text-on-surface-variant group-hover:text-on-surface transition-colors">{seed.name}</span>
							</button>
						{/each}
					</div>
				</div>
			</section>

			<!-- ═══ ADB Protocol ═══ -->
			<section class="rounded-[28px] bg-surface-container p-6 flex flex-col gap-5 shadow-sm">
				<div class="flex items-center gap-3">
					<div class="flex h-10 w-10 items-center justify-center rounded-2xl bg-primary-container text-on-primary-container">
						<span class="material-symbols-outlined text-[20px]">terminal</span>
					</div>
					<div>
						<h3 class="text-sm font-bold text-on-surface">ADB Protocol</h3>
						<p class="text-xs text-on-surface-variant">Daemon port and Logcat buffer streaming</p>
					</div>
				</div>

				<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
					<div class="flex flex-col gap-1.5">
						<label for="adb-port-input" class="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant">Daemon Port</label>
						<input
							id="adb-port-input"
							type="text"
							bind:value={adbDaemonPort}
							placeholder="5037"
							class="bg-surface-container-high rounded-2xl px-4 py-2.5 text-xs font-mono text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 shadow-xs"
						/>
					</div>
					<div class="flex flex-col gap-1.5">
						<label for="logcat-limit-select" class="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant">Logcat Buffer</label>
						<select
							id="logcat-limit-select"
							bind:value={logcatBufferLimit}
							class="bg-surface-container-high rounded-2xl px-4 py-2.5 text-xs font-semibold text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 shadow-xs cursor-pointer"
						>
							<option value="1000">1,000 Lines</option>
							<option value="5000">5,000 Lines (Default)</option>
							<option value="10000">10,000 Lines</option>
							<option value="unlimited">Unlimited (High RAM)</option>
						</select>
					</div>
				</div>
			</section>

			<!-- ═══ Security & Controls ═══ -->
			<section class="rounded-[28px] bg-surface-container p-6 flex flex-col gap-4 shadow-sm">
				<div class="flex items-center gap-3">
					<div class="flex h-10 w-10 items-center justify-center rounded-2xl bg-primary-container text-on-primary-container">
						<span class="material-symbols-outlined text-[20px]">verified_user</span>
					</div>
					<div>
						<h3 class="text-sm font-bold text-on-surface">Security & Controls</h3>
						<p class="text-xs text-on-surface-variant">Audit logging, confirmation prompts, and wireless pairing</p>
					</div>
				</div>

				<div class="flex flex-col gap-2">
					<label class="flex items-center justify-between p-4 rounded-2xl bg-surface-container-high/50 hover:bg-surface-container-high transition-all cursor-pointer">
						<div>
							<span class="text-xs font-bold text-on-surface block">Audit Log</span>
							<span class="text-[11px] text-on-surface-variant">Save ADB, package, and fastboot task history to JSON</span>
						</div>
						<div class="toggle-switch" class:active={auditEnabled}><input type="checkbox" bind:checked={auditEnabled} class="sr-only" /><span class="toggle-track"><span class="toggle-thumb"></span></span></div>
					</label>

					<label class="flex items-center justify-between p-4 rounded-2xl bg-surface-container-high/50 hover:bg-surface-container-high transition-all cursor-pointer">
						<div>
							<span class="text-xs font-bold text-on-surface block">Dangerous Action Prompts</span>
							<span class="text-[11px] text-on-surface-variant">Require confirmation before wipe, flash, or factory reset</span>
						</div>
						<div class="toggle-switch" class:active={confirmDangerousActions}><input type="checkbox" bind:checked={confirmDangerousActions} class="sr-only" /><span class="toggle-track"><span class="toggle-thumb"></span></span></div>
					</label>

					<label class="flex items-center justify-between p-4 rounded-2xl bg-surface-container-high/50 hover:bg-surface-container-high transition-all cursor-pointer">
						<div>
							<span class="text-xs font-bold text-on-surface block">Auto-Connect Wireless ADB</span>
							<span class="text-[11px] text-on-surface-variant">Scan and pair saved Wi-Fi devices on startup</span>
						</div>
						<div class="toggle-switch" class:active={autoWirelessAdb}><input type="checkbox" bind:checked={autoWirelessAdb} class="sr-only" /><span class="toggle-track"><span class="toggle-thumb"></span></span></div>
					</label>
				</div>
			</section>

			<!-- ═══ Danger Zone ═══ -->
			<section class="rounded-[28px] bg-surface-container p-5 flex items-center justify-between shadow-sm">
				<div>
					<span class="text-xs font-bold text-error block">Reset to Defaults</span>
					<span class="text-[11px] text-on-surface-variant">Restore all settings to factory values</span>
				</div>
				<button
					onclick={resetToDefaults}
					class="px-5 py-2.5 rounded-xl bg-error/15 text-error hover:bg-error/25 text-xs font-bold transition-all"
				>
					Reset
				</button>
			</section>

			</div>

			<!-- ═══ Right Column ═══ -->
			<div class="flex flex-col gap-5 lg:sticky lg:top-0">

				<!-- About & Info Card -->
				<section class="rounded-[28px] bg-surface-container p-6 flex flex-col gap-5 shadow-sm">

					<!-- Community & License Grid -->
					<div class="grid grid-cols-2 gap-3">
						<a
							href="https://github.com/pavelc4/Oxide"
							target="_blank"
							rel="noreferrer"
							class="rounded-2xl bg-surface-container-high p-4 flex flex-col gap-2 hover:bg-surface-container-highest transition-all no-underline group"
						>
							<div class="flex h-9 w-9 items-center justify-center rounded-xl bg-surface-container text-on-surface-variant group-hover:text-primary">
								<span class="material-symbols-outlined text-[20px]">code</span>
							</div>
							<span class="text-sm font-bold text-on-surface group-hover:text-primary transition-colors">Repository</span>
							<span class="text-[11px] text-on-surface-variant leading-snug">Source code on GitHub</span>
						</a>

						<div class="rounded-2xl bg-surface-container-high p-4 flex flex-col gap-2">
							<div class="flex h-9 w-9 items-center justify-center rounded-xl bg-surface-container text-on-surface-variant">
								<span class="material-symbols-outlined text-[20px]">info</span>
							</div>
							<span class="text-sm font-bold text-on-surface">License</span>
							<span class="text-[11px] text-on-surface-variant leading-snug">Open Source Software</span>
						</div>
					</div>

					<!-- Version Info -->
					<div class="flex flex-col gap-2 text-xs px-1">
						<div class="flex justify-between items-center py-1">
							<span class="text-on-surface-variant">Version</span>
							<span class="font-bold text-on-surface">Oxide v0.1.0</span>
						</div>
						<div class="flex justify-between items-center py-1">
							<span class="text-on-surface-variant">Frontend</span>
							<span class="font-mono text-on-surface">Svelte 5 + Vite 6</span>
						</div>
						<div class="flex justify-between items-center py-1">
							<span class="text-on-surface-variant">Backend</span>
							<span class="font-mono text-on-surface">Rust (Tauri 2 + adb_client)</span>
						</div>
					</div>

					<!-- Support -->
					<div class="rounded-2xl bg-surface-container-high p-5 flex flex-col gap-3">
						<h3 class="text-base font-bold text-on-surface">Support</h3>
						<p class="text-xs text-on-surface-variant leading-relaxed">
							If Oxide saves your time managing ADB devices, consider supporting the development.
						</p>
						<a
							href="https://github.com/sponsors/pavelc4"
							target="_blank"
							rel="noreferrer"
							class="flex items-center justify-center gap-2.5 py-3 rounded-2xl bg-primary/15 text-primary hover:bg-primary/25 text-sm font-bold transition-all no-underline"
						>
							<span class="material-symbols-outlined text-[20px]">favorite</span>
							Sponsor on GitHub
						</a>
					</div>
				</section>

				<!-- Contributors Card -->
				<section class="rounded-[28px] bg-surface-container p-6 flex flex-col gap-4 shadow-sm">
					<div class="flex items-center gap-3">
						<div class="flex h-9 w-9 items-center justify-center rounded-xl bg-surface-container-high text-on-surface-variant">
							<span class="material-symbols-outlined text-[20px]">group</span>
						</div>
						<h3 class="text-sm font-bold text-on-surface">Contributors</h3>
					</div>

					{#if contributorsLoading}
						<div class="flex items-center justify-center py-6">
							<div class="w-5 h-5 rounded-full border-2 border-primary/30 border-t-primary animate-spin"></div>
						</div>
					{:else}
						{#if leadContributor}
							<a
								href={leadContributor.html_url}
								target="_blank"
								rel="noreferrer"
								class="flex items-center justify-between p-4 rounded-2xl bg-surface-container-high hover:bg-surface-container-highest transition-all group no-underline"
							>
								<div class="flex items-center gap-3.5">
									<img src={leadContributor.avatar_url} alt={leadContributor.login} class="w-11 h-11 rounded-full object-cover shadow-sm bg-surface-container" loading="lazy" />
									<div>
										<span class="text-sm font-bold text-on-surface group-hover:text-primary transition-colors">{leadContributor.login}</span>
										<span class="text-[11px] text-on-surface-variant block">Creator & Lead Developer</span>
									</div>
								</div>
								<span class="material-symbols-outlined text-on-surface-variant text-[20px] group-hover:text-primary transition-colors">chevron_right</span>
							</a>
						{/if}

						{#if otherContributors.length > 0}
							<button
								onclick={() => (showAllContributors = !showAllContributors)}
								class="flex items-center justify-center gap-1.5 py-2 text-xs font-bold text-primary hover:text-primary/80 transition-colors"
							>
								<span class="material-symbols-outlined text-[18px]">{showAllContributors ? 'expand_less' : 'expand_more'}</span>
								{showAllContributors ? 'Hide Contributors' : `Show ${otherContributors.length} more Contributors`}
							</button>

							{#if showAllContributors}
								<div class="flex flex-col gap-2 animate-fade-in">
									{#each otherContributors as c}
										<a
											href={c.html_url}
											target="_blank"
											rel="noreferrer"
											class="flex items-center justify-between p-3.5 rounded-2xl bg-surface-container-high/60 hover:bg-surface-container-high transition-all group no-underline"
										>
											<div class="flex items-center gap-3">
												<img src={c.avatar_url} alt={c.login} class="w-9 h-9 rounded-full object-cover bg-surface-container" loading="lazy" />
												<div>
													<span class="text-xs font-bold text-on-surface group-hover:text-primary transition-colors">{c.login}</span>
													<span class="text-[11px] text-on-surface-variant block">{c.contributions} contribution{c.contributions !== 1 ? 's' : ''}</span>
												</div>
											</div>
											<span class="material-symbols-outlined text-on-surface-variant text-[18px] group-hover:text-primary transition-colors">chevron_right</span>
										</a>
									{/each}
								</div>
							{/if}
						{/if}
					{/if}
				</section>

			</div>

		</div>
	</div>
</main>

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
		animation: fadeIn 0.2s cubic-bezier(0.2, 0.8, 0.2, 1) forwards;
	}
	@keyframes spin {
		to { transform: rotate(360deg); }
	}
	.animate-spin {
		animation: spin 0.6s linear infinite;
	}
	.toggle-switch {
		flex-shrink: 0;
		cursor: pointer;
	}
	.toggle-track {
		display: block;
		width: 44px;
		height: 26px;
		border-radius: 13px;
		background: var(--md-sys-color-surface-container-highest, #444);
		position: relative;
		transition: background 0.2s ease;
	}
	.toggle-thumb {
		position: absolute;
		top: 3px;
		left: 3px;
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: var(--md-sys-color-on-surface-variant, #aaa);
		transition: transform 0.2s ease, background 0.2s ease;
	}
	.toggle-switch.active .toggle-track {
		background: var(--md-sys-color-primary, #ffb784);
	}
	.toggle-switch.active .toggle-thumb {
		transform: translateX(18px);
		background: var(--md-sys-color-on-primary, #1a1a1a);
	}
</style>
