<script lang="ts">
	/* eslint-disable svelte/no-navigation-without-resolve */
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { playChime } from '$lib/utils/audio';
	import Select from '$lib/components/Select.svelte';
	import ShapeBadge from '$lib/components/ShapeBadge.svelte';
	import type { MaterialShapeType } from '$lib/shapes/materialShapes';
	import { applyThemeColorToDocument } from '$lib/theme';

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
		type?: string;
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

	// Sound & Notifications Customization States
	let soundEnabled = $state(true);
	let deviceSoundEnabled = $state(true);
	let installSoundEnabled = $state(true);
	let toastDuration = $state<'2' | '4' | '6'>('4');
	let perfRefreshInterval = $state<'1' | '2' | '5'>('2');

	// Extended Customization States
	let customHexColor = $state('#ffb784');
	let oledMode = $state(false);
	let terminalFontSize = $state<'12' | '14' | '16'>('14');
	let terminalCursorStyle = $state<'block' | 'underline' | 'bar'>('block');
	let autoGrantPermissions = $state(true);
	let iconShape = $state<MaterialShapeType | 'rounded'>('cookie7');

	$effect(() => {
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem('oxide:soundEnabled', String(soundEnabled));
			localStorage.setItem('oxide:deviceSoundEnabled', String(deviceSoundEnabled));
			localStorage.setItem('oxide:installSoundEnabled', String(installSoundEnabled));
			localStorage.setItem('oxide:toastDuration', toastDuration);
			localStorage.setItem('oxide:perfRefreshInterval', perfRefreshInterval);
			localStorage.setItem('oxide:oledMode', String(oledMode));
			localStorage.setItem('oxide:terminalFontSize', terminalFontSize);
			localStorage.setItem('oxide:terminalCursorStyle', terminalCursorStyle);
			localStorage.setItem('oxide:autoGrantPermissions', String(autoGrantPermissions));
			localStorage.setItem('oxide:iconShape', iconShape);
		}
	});

	// Dynamic OLED True Black Mode Effect
	$effect(() => {
		if (typeof document !== 'undefined') {
			const root = document.documentElement;
			if (oledMode) {
				root.style.setProperty('--background', '#000000');
				root.style.setProperty('--surface', '#000000');
				root.style.setProperty('--surfaceContainerLowest', '#000000');
			} else if (selectedSeedColor) {
				applyThemeColor(selectedSeedColor);
			}
		}
	});

	// GitHub Contributors (fetched & cached)
	let contributors = $state<GitHubContributor[]>([]);
	let contributorsLoading = $state(true);

	// Theme Color Seeds
	let selectedSeedColor = $state(
		typeof localStorage !== 'undefined'
			? localStorage.getItem('oxide:seedColor') || '#ffb784'
			: '#ffb784'
	);
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
		{ name: 'Pastel Lilac', hex: '#f472b6', colors: ['#f4b9e2', '#fde8f5', '#4e4049', '#ffe08a'] },
		{ name: 'Sapphire Blue', hex: '#3b82f6', colors: ['#93c5fd', '#dbeafe', '#1e3a8a', '#93c5fd'] },
		{ name: 'Sunset Orange', hex: '#f97316', colors: ['#fdba74', '#ffedd5', '#7c2d12', '#fdba74'] },
		{ name: 'Electric Indigo', hex: '#6366f1', colors: ['#a5b4fc', '#e0e7ff', '#312e81', '#a5b4fc'] },
		{ name: 'Crimson Ruby', hex: '#e11d48', colors: ['#fda4af', '#ffe4e6', '#881337', '#fda4af'] },
		{ name: 'Mint Jade', hex: '#14b8a6', colors: ['#5eead4', '#ccfbf1', '#134e4a', '#5eead4'] },
		{ name: 'Golden Canary', hex: '#eab308', colors: ['#fde047', '#fef9c3', '#713f12', '#fde047'] }
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

		if (typeof localStorage !== 'undefined') {
			soundEnabled = localStorage.getItem('oxide:soundEnabled') !== 'false';
			deviceSoundEnabled = localStorage.getItem('oxide:deviceSoundEnabled') !== 'false';
			installSoundEnabled = localStorage.getItem('oxide:installSoundEnabled') !== 'false';
			toastDuration = (localStorage.getItem('oxide:toastDuration') as '2' | '4' | '6') || '4';
			perfRefreshInterval = (localStorage.getItem('oxide:perfRefreshInterval') as '1' | '2' | '5') || '2';
			oledMode = localStorage.getItem('oxide:oledMode') === 'true';
			terminalFontSize = (localStorage.getItem('oxide:terminalFontSize') as '12' | '14' | '16') || '14';
			terminalCursorStyle = (localStorage.getItem('oxide:terminalCursorStyle') as 'block' | 'underline' | 'bar') || 'block';
			autoGrantPermissions = localStorage.getItem('oxide:autoGrantPermissions') !== 'false';
			iconShape = (localStorage.getItem('oxide:iconShape') as MaterialShapeType | 'rounded') || 'cookie7';
			const savedCorner = (localStorage.getItem('oxide:cornerRadius') as 'expressive' | 'medium' | 'compact') || 'expressive';
			applyCornerRadius(savedCorner);

			const savedSeed = localStorage.getItem('oxide:seedColor');
			if (savedSeed) {
				applyThemeColor(savedSeed);
			}
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
				const data: (GitHubContributor & { type?: string })[] = await res.json();
				contributors = data.map((c) => ({
					login: c.login,
					avatar_url: c.avatar_url,
					html_url: c.html_url,
					contributions: c.contributions,
					type: c.type || (c.login.toLowerCase().includes('bot') ? 'Bot' : 'User')
				}));
				if (typeof localStorage !== 'undefined') {
					localStorage.setItem(cacheKey, JSON.stringify(contributors));
					localStorage.setItem(cacheExpiry, Date.now().toString());
				}
			}
		} catch {
			// Fallback: default mock contributors if offline / API rate limited
			if (contributors.length === 0) {
				contributors = [
					{
						login: 'pavelc4',
						avatar_url: 'https://github.com/pavelc4.png',
						html_url: 'https://github.com/pavelc4',
						contributions: 42,
						type: 'User'
					},
					{
						login: 'dependabot[bot]',
						avatar_url: 'https://avatars.githubusercontent.com/in/29110?v=4',
						html_url: 'https://github.com/apps/dependabot',
						contributions: 7,
						type: 'Bot'
					}
				];
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
			const savedMode = typeof localStorage !== 'undefined' ? localStorage.getItem('oxide:themeMode') : null;
			if (isTauri && invoke) {
				const cfg = await safeInvoke<AppConfig>('get_config');
				if (cfg) {
					currentTheme = savedMode || cfg.theme || 'dark';
					auditEnabled = cfg.audit_enabled ?? true;
					setupCompleted = cfg.setup_completed ?? true;
				}
			} else {
				currentTheme = savedMode || 'dark';
				auditEnabled = true;
			}
			if (typeof document !== 'undefined') {
				document.documentElement.setAttribute('data-theme', currentTheme);
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

	function hexToHsl(hex: string): { h: number; s: number; l: number } {
		const clean = hex.replace('#', '');
		const r = parseInt(clean.substring(0, 2), 16) / 255;
		const g = parseInt(clean.substring(2, 4), 16) / 255;
		const b = parseInt(clean.substring(4, 6), 16) / 255;

		const max = Math.max(r, g, b);
		const min = Math.min(r, g, b);
		let h = 0, s = 0;
		const l = (max + min) / 2;

		if (max !== min) {
			const d = max - min;
			s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
			switch (max) {
				case r: h = (g - b) / d + (g < b ? 6 : 0); break;
				case g: h = (b - r) / d + 2; break;
				case b: h = (r - g) / d + 4; break;
			}
			h /= 6;
		}
		return { h: Math.round(h * 360), s: Math.round(s * 100), l: Math.round(l * 100) };
	}

	async function applyThemeColor(colorHex: string) {
		selectedSeedColor = colorHex;
		customHexColor = colorHex;
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem('oxide:seedColor', colorHex);
		}

		applyThemeColorToDocument(colorHex);

		infoMessage = `Theme color updated to ${colorHex}`;
		setTimeout(() => { if (infoMessage.includes(colorHex)) infoMessage = ''; }, 2500);
	}

	function selectThemeMode(modeId: string) {
		currentTheme = modeId;
		if (typeof document !== 'undefined') {
			document.documentElement.setAttribute('data-theme', modeId);
		}
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem('oxide:themeMode', modeId);
		}
		if (selectedSeedColor) {
			applyThemeColor(selectedSeedColor);
		}
		saveSettings();
	}

	function applyCornerRadius(radius: 'expressive' | 'medium' | 'compact') {
		cornerRadius = radius;
		if (typeof document !== 'undefined') {
			document.documentElement.setAttribute('data-corner', radius);
		}
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem('oxide:cornerRadius', radius);
		}
		saveSettings();
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

	// Derived: sorted contributors separated into humans and bots
	let sortedContributors = $derived(
		[...contributors].sort((a, b) => b.contributions - a.contributions)
	);
	let humanContributors = $derived(
		sortedContributors.filter((c) => c.type !== 'Bot' && !c.login.toLowerCase().includes('bot'))
	);
	let botContributors = $derived(
		sortedContributors.filter((c) => c.type === 'Bot' || c.login.toLowerCase().includes('bot'))
	);
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
					<ShapeBadge icon="palette" shape="fan" size={40} iconSize={20} />
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
								onclick={() => selectThemeMode(mode.id)}
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
								onclick={() => applyCornerRadius(shape.id as 'expressive' | 'medium' | 'compact')}
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
					<div class="flex items-center justify-between">
						<span class="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant">Theme Color</span>
					</div>
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



				<!-- OLED True Black Toggle -->
				<label class="flex items-center justify-between p-4 rounded-2xl bg-surface-container-high/50 hover:bg-surface-container-high transition-all cursor-pointer">
					<div>
						<span class="text-xs font-bold text-on-surface block">OLED True Black Mode</span>
						<span class="text-[11px] text-on-surface-variant">Use pitch black (#000000) background for OLED energy saving</span>
					</div>
					<div class="toggle-switch" class:active={oledMode}><input type="checkbox" bind:checked={oledMode} class="sr-only" /><span class="toggle-track"><span class="toggle-thumb"></span></span></div>
				</label>
			</section>

			<!-- ═══ ADB Protocol ═══ -->
			<section class="rounded-[28px] bg-surface-container p-6 flex flex-col gap-5 shadow-sm">
				<div class="flex items-center gap-3">
					<ShapeBadge icon="terminal" shape="arch" size={40} iconSize={20} />
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
							class="bg-surface-container-high rounded-2xl px-4 py-2.5 text-xs font-mono text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/40 shadow-xs border-0 outline-none"
						/>
					</div>
					<div class="flex flex-col gap-1.5">
						<span class="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant">Logcat Buffer</span>
						<Select
							id="logcat-limit-select"
							bind:value={logcatBufferLimit}
							options={[
								{ value: '1000', label: '1,000 Lines' },
								{ value: '5000', label: '5,000 Lines (Default)' },
								{ value: '10000', label: '10,000 Lines' },
								{ value: 'unlimited', label: 'Unlimited (High RAM)' }
							]}
						/>
					</div>
				</div>


			</section>

			<!-- ═══ Security & Controls ═══ -->
			<section class="rounded-[28px] bg-surface-container p-6 flex flex-col gap-4 shadow-sm">
				<div class="flex items-center gap-3">
					<ShapeBadge icon="verified_user" shape="gem" size={40} iconSize={20} />
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

			<!-- ═══ Sound & Notifications ═══ -->
			<section class="rounded-[28px] bg-surface-container p-6 flex flex-col gap-5 shadow-sm">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-3">
						<ShapeBadge icon="notifications_active" shape="sunny" size={40} iconSize={20} />
						<div>
							<h3 class="text-sm font-bold text-on-surface">Sound & Notifications</h3>
							<p class="text-xs text-on-surface-variant">Audio chimes, toast duration, and refresh intervals</p>
						</div>
					</div>
					<button
						onclick={() => playChime('connect')}
						class="flex items-center gap-2 px-3 py-1.5 rounded-2xl bg-surface-container-high hover:bg-surface-container-highest text-on-surface text-xs font-bold transition-all shadow-xs cursor-pointer group border-0 outline-none active:scale-95"
						title="Test audio chime"
					>
						<ShapeBadge icon="volume_up" shape="cookie7" size={28} iconSize={14} variant="primary" />
						<span>Test Sound</span>
					</button>
				</div>

				<div class="flex flex-col gap-2">
					<label class="flex items-center justify-between p-4 rounded-2xl bg-surface-container-high/50 hover:bg-surface-container-high transition-all cursor-pointer">
						<div>
							<span class="text-xs font-bold text-on-surface block">Audio Chimes & Sound FX</span>
							<span class="text-[11px] text-on-surface-variant">Play subtle Web Audio chimes for events</span>
						</div>
						<div class="toggle-switch" class:active={soundEnabled}><input type="checkbox" bind:checked={soundEnabled} class="sr-only" /><span class="toggle-track"><span class="toggle-thumb"></span></span></div>
					</label>

					<label class="flex items-center justify-between p-4 rounded-2xl bg-surface-container-high/50 hover:bg-surface-container-high transition-all cursor-pointer">
						<div>
							<span class="text-xs font-bold text-on-surface block">Device Connection Sounds</span>
							<span class="text-[11px] text-on-surface-variant">Play sound when ADB device connects or disconnects</span>
						</div>
						<div class="toggle-switch" class:active={deviceSoundEnabled}><input type="checkbox" bind:checked={deviceSoundEnabled} class="sr-only" /><span class="toggle-track"><span class="toggle-thumb"></span></span></div>
					</label>

					<label class="flex items-center justify-between p-4 rounded-2xl bg-surface-container-high/50 hover:bg-surface-container-high transition-all cursor-pointer">
						<div>
							<span class="text-xs font-bold text-on-surface block">APK Install Success Chime</span>
							<span class="text-[11px] text-on-surface-variant">Play chime when app finishes installing</span>
						</div>
						<div class="toggle-switch" class:active={installSoundEnabled}><input type="checkbox" bind:checked={installSoundEnabled} class="sr-only" /><span class="toggle-track"><span class="toggle-thumb"></span></span></div>
					</label>
				</div>

				<div class="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-1">
					<div class="flex flex-col gap-1.5">
						<span class="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant">Toast Dismiss Time</span>
						<Select
							id="toast-duration-select"
							bind:value={toastDuration}
							options={[
								{ value: '2', label: '2 Seconds (Fast)' },
								{ value: '4', label: '4 Seconds (Default)' },
								{ value: '6', label: '6 Seconds (Relaxed)' }
							]}
						/>
					</div>
					<div class="flex flex-col gap-1.5">
						<span class="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant">Dashboard Monitor Rate</span>
						<Select
							id="perf-interval-select"
							bind:value={perfRefreshInterval}
							options={[
								{ value: '1', label: '1 Second (High Speed)' },
								{ value: '2', label: '2 Seconds (Default)' },
								{ value: '5', label: '5 Seconds (Battery Saver)' }
							]}
						/>
					</div>
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
					<div class="flex items-center gap-3">
						<ShapeBadge icon="info" shape="cookie7" size={40} iconSize={20} />
						<div>
							<h3 class="text-base font-bold text-on-surface">About Oxide</h3>
							<p class="text-xs text-on-surface-variant">Version 0.1.0</p>
						</div>
					</div>

					<!-- Technology Stack Icons & Badges -->
					<div class="flex flex-col gap-2.5">
						<span class="text-[11px] font-bold uppercase tracking-wider text-on-surface-variant">Tech Stack & Architecture</span>

						<div class="grid grid-cols-2 gap-2 text-xs font-semibold">
							<!-- Svelte 5 -->
							<div class="flex items-center gap-3 p-2.5 rounded-2xl bg-surface-container-high border border-surface-container-highest/40">
								<img src="https://skillicons.dev/icons?i=svelte" alt="Svelte" class="w-7 h-7 shrink-0 drop-shadow-xs" />
								<div>
									<span class="block text-on-surface font-bold text-xs">Svelte 5</span>
									<span class="block text-[10px] text-on-surface-variant font-normal">Frontend SPA</span>
								</div>
							</div>

							<!-- Rust -->
							<div class="flex items-center gap-3 p-2.5 rounded-2xl bg-surface-container-high border border-surface-container-highest/40">
								<img src="https://skillicons.dev/icons?i=rust" alt="Rust" class="w-7 h-7 shrink-0 drop-shadow-xs" />
								<div>
									<span class="block text-on-surface font-bold text-xs">Rust</span>
									<span class="block text-[10px] text-on-surface-variant font-normal">Async Engine</span>
								</div>
							</div>

							<!-- Tauri 2 -->
							<div class="flex items-center gap-3 p-2.5 rounded-2xl bg-surface-container-high border border-surface-container-highest/40">
								<img src="https://skillicons.dev/icons?i=tauri" alt="Tauri" class="w-7 h-7 shrink-0 drop-shadow-xs" />
								<div>
									<span class="block text-on-surface font-bold text-xs">Tauri 2</span>
									<span class="block text-[10px] text-on-surface-variant font-normal">Desktop Shell</span>
								</div>
							</div>

							<!-- Vite 6 -->
							<div class="flex items-center gap-3 p-2.5 rounded-2xl bg-surface-container-high border border-surface-container-highest/40">
								<img src="https://skillicons.dev/icons?i=vite" alt="Vite" class="w-7 h-7 shrink-0 drop-shadow-xs" />
								<div>
									<span class="block text-on-surface font-bold text-xs">Vite 6</span>
									<span class="block text-[10px] text-on-surface-variant font-normal">Fast Bundler</span>
								</div>
							</div>

							<!-- TypeScript -->
							<div class="flex items-center gap-3 p-2.5 rounded-2xl bg-surface-container-high border border-surface-container-highest/40">
								<img src="https://skillicons.dev/icons?i=ts" alt="TypeScript" class="w-7 h-7 shrink-0 drop-shadow-xs" />
								<div>
									<span class="block text-on-surface font-bold text-xs">TypeScript</span>
									<span class="block text-[10px] text-on-surface-variant font-normal">Type Safety</span>
								</div>
							</div>

							<!-- Tailwind CSS -->
							<div class="flex items-center gap-3 p-2.5 rounded-2xl bg-surface-container-high border border-surface-container-highest/40">
								<img src="https://skillicons.dev/icons?i=tailwind" alt="Tailwind" class="w-7 h-7 shrink-0 drop-shadow-xs" />
								<div>
									<span class="block text-on-surface font-bold text-xs">Tailwind</span>
									<span class="block text-[10px] text-on-surface-variant font-normal">Modern CSS</span>
								</div>
							</div>
						</div>
					</div>

					<!-- Community & License Grid -->
					<div class="grid grid-cols-2 gap-3 pt-1">
						<a
							href="https://github.com/pavelc4/Oxide"
							target="_blank"
							rel="noreferrer"
							class="rounded-2xl bg-surface-container-high p-3.5 flex flex-col gap-1.5 hover:bg-surface-container-highest transition-all no-underline group"
						>
							<div class="flex h-8 w-8 items-center justify-center rounded-xl bg-surface-container group-hover:bg-primary-container text-on-surface-variant group-hover:text-on-primary-container transition-colors">
								<img src="https://skillicons.dev/icons?i=github" alt="GitHub" class="w-5 h-5 shrink-0" />
							</div>
							<span class="text-xs font-bold text-on-surface group-hover:text-primary transition-colors">Repository</span>
							<span class="text-[10px] text-on-surface-variant leading-snug">Source on GitHub</span>
						</a>

						<div class="rounded-2xl bg-surface-container-high p-3.5 flex flex-col gap-1.5">
							<div class="flex h-8 w-8 items-center justify-center rounded-xl bg-surface-container text-on-surface-variant">
								<span class="material-symbols-outlined text-[18px]">gavel</span>
							</div>
							<span class="text-xs font-bold text-on-surface">GPL-3.0 License</span>
							<span class="text-[10px] text-on-surface-variant leading-snug">GNU General Public</span>
						</div>
					</div>

					<!-- Support -->
					<div class="rounded-2xl bg-surface-container-high p-4 flex flex-col gap-2.5">
						<h4 class="text-xs font-bold text-on-surface flex items-center gap-2">
							<span class="material-symbols-outlined text-primary text-[16px]">favorite</span> Support Oxide
						</h4>
						<p class="text-[11px] text-on-surface-variant leading-relaxed">
							If Oxide saves your time managing ADB devices, consider supporting the project.
						</p>
						<a
							href="https://github.com/sponsors/pavelc4"
							target="_blank"
							rel="noreferrer"
							class="flex items-center justify-center gap-2 py-2.5 rounded-xl bg-primary/15 text-primary hover:bg-primary/25 text-xs font-bold transition-all no-underline"
						>
							<span class="material-symbols-outlined text-[16px]">favorite</span>
							Sponsor on GitHub
						</a>
					</div>
				</section>

				<!-- Contributors Card (ALL Displayed Directly with Ranks & Stats) -->
				<section class="rounded-[28px] bg-surface-container p-6 flex flex-col gap-4 shadow-sm relative overflow-hidden">
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-3">
							<ShapeBadge icon="groups" shape="clover" size={38} iconSize={20} />
							<div>
								<h3 class="text-sm font-bold text-on-surface">Contributors</h3>
								<span class="text-[10px] text-on-surface-variant block font-medium">{sortedContributors.length} active developer{sortedContributors.length !== 1 ? 's' : ''}</span>
							</div>
						</div>
					</div>

					{#if contributorsLoading}
						<div class="flex items-center justify-center py-6">
							<div class="w-5 h-5 rounded-full border-2 border-primary/30 border-t-primary animate-spin"></div>
						</div>
					{:else}
						<div class="flex flex-col gap-4">
							<!-- Human Developers Section -->
							{#if humanContributors.length > 0}
								<div class="flex flex-col gap-2">
									<span class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant px-1">Developers</span>
									<div class="flex flex-col gap-2">
										{#each humanContributors as c, index (c.login)}
											<a
												href={c.html_url}
												target="_blank"
												rel="noreferrer"
												class="flex items-center justify-between p-3.5 rounded-2xl bg-surface-container-high hover:bg-surface-container-highest transition-all group no-underline"
											>
												<div class="flex items-center gap-3">
													<!-- Rank Badge -->
													<span class="flex h-6 w-6 items-center justify-center rounded-full text-[10px] font-bold shrink-0 bg-surface-container text-on-surface-variant">
														#{index + 1}
													</span>

													<!-- Avatar with Material 3 Shape (gem for pavelc4) -->
													<ShapeBadge
														imgUrl={c.avatar_url}
														shape="gem"
														size={44}
														class="shrink-0 shadow-xs"
													/>

													<!-- Info -->
													<div>
														<div class="flex items-center gap-2">
															<span class="text-xs font-bold text-on-surface group-hover:text-primary transition-colors">{c.login}</span>
															{#if index === 0}
																<span class="text-[9px] font-bold bg-primary/15 text-primary px-2 py-0.5 rounded-full uppercase tracking-wider">
																	Lead
																</span>
															{/if}
														</div>
														<span class="text-[11px] text-on-surface-variant block font-medium mt-0.5">
															{c.contributions > 0 ? `${c.contributions} contribution${c.contributions !== 1 ? 's' : ''}` : 'Creator & Lead Developer'}
														</span>
													</div>
												</div>
											</a>
										{/each}
									</div>
								</div>
							{/if}

							<!-- Bots Section (Separate Sub-card) -->
							{#if botContributors.length > 0}
								<div class="flex flex-col gap-2 pt-1">
									<span class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant px-1">Automated Bots</span>
									<div class="flex flex-col gap-2">
										{#each botContributors as c, index (c.login)}
											<a
												href={c.html_url}
												target="_blank"
												rel="noreferrer"
												class="flex items-center justify-between p-3.5 rounded-2xl bg-surface-container-high/60 hover:bg-surface-container-high transition-all group no-underline"
											>
												<div class="flex items-center gap-3">
													<!-- Rank Badge -->
													<span class="flex h-6 w-6 items-center justify-center rounded-full text-[10px] font-bold shrink-0 bg-surface-container/60 text-on-surface-variant/70">
														#{humanContributors.length + index + 1}
													</span>

													<!-- Avatar with Material 3 Shape (sunny for bots) -->
													<ShapeBadge
														imgUrl={c.avatar_url}
														shape="sunny"
														size={44}
														class="shrink-0 shadow-xs"
													/>

													<!-- Info -->
													<div>
														<div class="flex items-center gap-2">
															<span class="text-xs font-bold text-on-surface group-hover:text-primary transition-colors">{c.login}</span>
															<span class="text-[9px] font-bold bg-sky-500/15 text-sky-400 px-2 py-0.5 rounded-full uppercase tracking-wider">
																Bot
															</span>
														</div>
														<span class="text-[11px] text-on-surface-variant block font-medium mt-0.5">
															{c.contributions} contribution{c.contributions !== 1 ? 's' : ''}
														</span>
													</div>
												</div>
											</a>
										{/each}
									</div>
								</div>
							{/if}
						</div>
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
		background: var(--surfaceContainerHighest, #3b2d25);
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
		background: var(--onSurfaceVariant, #9f8c85);
		transition: transform 0.2s cubic-bezier(0.2, 0.8, 0.2, 1), background 0.2s ease;
	}
	.toggle-switch.active .toggle-track {
		background: var(--primary);
	}
	.toggle-switch.active .toggle-thumb {
		transform: translateX(18px);
		background: var(--onPrimary);
	}
	input[type="text"] {
		background-color: var(--surfaceContainerHigh, #2f241d);
		color: var(--onSurface, #efe0d9);
	}
</style>
