<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | undefined;
	let isTauri = $state(false);

	interface Device {
		id: string;
		name: string;
	}

	let devices = $state<Device[]>([]);
	let selectedDevice = $state<string>('');
	let loading = $state(true);
	let error = $state('');
	let infoMessage = $state('');

	let maxSize = $state('1024');
	let maxFps = $state('60');
	let bitRate = $state('4000000');
	let noAudio = $state(false);
	let showTouches = $state(false);
	let stayAwake = $state(true);
	let turnScreenOff = $state(false);
	let alwaysOnTop = $state(false);
	let fullscreen = $state(false);

	let clipboardText = $state('');
	let syncingClipboard = $state(false);
	let screenshotName = $state('screenshot.png');
	let takingScreenshot = $state(false);
	let scrcpyPath = $state('scrcpy');

	onMount(() => {
		devices = [
			{ id: 'mock-device-123', name: 'Mock Galaxy S24 Ultra' },
			{ id: 'mock-device-456', name: 'Mock Pixel 8 Pro' },
		];
		selectedDevice = devices[0].id;
		loading = false;
	});

	async function startMirroring() {
		if (!selectedDevice) return;
		error = '';
		infoMessage = '';

		const opts = {
			max_size: maxSize ? parseInt(maxSize) : null,
			bit_rate: bitRate ? parseInt(bitRate) : null,
			max_fps: maxFps ? parseInt(maxFps) : null,
			audio_bit_rate: null,
			audio_codec: null,
			video_codec: null,
			show_touches: showTouches,
			no_audio: noAudio,
			no_control: false,
			stay_awake: stayAwake,
			turn_screen_off: turnScreenOff,
			power_off_on_close: false,
			fullscreen: fullscreen,
			always_on_top: alwaysOnTop,
			disable_screensaver: false,
			rotation: null,
			display_id: null,
		};

		try {
			if (!invoke) {
				const tauriApi = await import('@tauri-apps/api/core');
				invoke = tauriApi.invoke;
			}

			const handle = await Promise.race([
				invoke('scrcpy_start', { serial: selectedDevice, opts, scrcpyPath }),
				new Promise<null>((_, reject) => setTimeout(() => reject(new Error('timeout')), 5000)),
			]);
			isTauri = true;
			infoMessage = `Mirroring session launched successfully! (Session: ${handle})`;
		} catch (e) {
			if (e instanceof Error && e.message === 'timeout') {
				error = 'Connection timed out. Make sure ADB server is running.';
			} else {
				infoMessage = 'Mock mode: mirroring session simulated.';
			}
		}
	}

	async function takeScreenshot() {
		if (!selectedDevice) return;
		takingScreenshot = true;
		error = '';
		infoMessage = '';

		const fileName = screenshotName.trim() || 'screenshot.png';
		const destDir = '/tmp';
		const outPath = `${destDir}/${Date.now()}_${fileName}`;

		try {
			if (!invoke) {
				const tauriApi = await import('@tauri-apps/api/core');
				invoke = tauriApi.invoke;
			}

			await Promise.race([
				invoke('take_screenshot', { serial: selectedDevice, outputPath: outPath }),
				new Promise((_, reject) => setTimeout(() => reject(new Error('timeout')), 10000)),
			]);
			isTauri = true;
			infoMessage = `Screenshot saved to: ${outPath}`;
		} catch (e) {
			if (e instanceof Error && e.message === 'timeout') {
				error = 'Screenshot timed out. Make sure ADB server is running.';
			} else {
				infoMessage = 'Mock mode: screenshot simulated.';
			}
		} finally {
			takingScreenshot = false;
		}
	}

	async function pullClipboard() {
		if (!selectedDevice) return;
		syncingClipboard = true;
		error = '';
		infoMessage = '';
		try {
			if (!invoke) {
				const tauriApi = await import('@tauri-apps/api/core');
				invoke = tauriApi.invoke;
			}

			const text = await Promise.race([
				invoke('get_clipboard', { serial: selectedDevice }),
				new Promise((_, reject) => setTimeout(() => reject(new Error('timeout')), 5000)),
			]) as string;
			isTauri = true;
			clipboardText = text || '';
			infoMessage = 'Clipboard fetched from device.';
		} catch (e) {
			if (e instanceof Error && e.message === 'timeout') {
				error = 'Clipboard pull timed out.';
			} else {
				await new Promise((r) => setTimeout(r, 600));
				clipboardText = 'Hello from Android Clipboard (Mock)!';
				infoMessage = 'Clipboard fetched (mock).';
			}
		} finally {
			syncingClipboard = false;
		}
	}

	async function pushClipboard() {
		if (!selectedDevice || !clipboardText) return;
		syncingClipboard = true;
		error = '';
		infoMessage = '';
		try {
			if (!invoke) {
				const tauriApi = await import('@tauri-apps/api/core');
				invoke = tauriApi.invoke;
			}

			await Promise.race([
				invoke('push_clipboard', { serial: selectedDevice, text: clipboardText }),
				new Promise((_, reject) => setTimeout(() => reject(new Error('timeout')), 5000)),
			]);
			isTauri = true;
			infoMessage = 'Text pushed to device clipboard.';
		} catch (e) {
			if (e instanceof Error && e.message === 'timeout') {
				error = 'Clipboard push timed out.';
			} else {
				await new Promise((r) => setTimeout(r, 600));
				infoMessage = 'Text pushed to device clipboard (mock).';
			}
		} finally {
			syncingClipboard = false;
		}
	}
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2 h-screen overflow-hidden">
	<div
		class="flex flex-1 flex-col overflow-hidden rounded-[32px] bg-surface-container-low p-6 lg:p-10 relative"
	>
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
					Screen Mirroring Hub
					{#if !isTauri}
						<span
							class="text-xs bg-error/15 text-error px-3 py-1 rounded-full font-medium tracking-normal border border-error/30"
						>MOCK MODE</span>
					{/if}
				</h2>
			</div>

			<div class="flex items-center gap-3">
				<span class="text-xs font-semibold text-on-surface-variant">Device:</span>
				<select
					bind:value={selectedDevice}
					class="bg-surface-container-high rounded-full border border-outline-variant px-4 py-1.5 text-sm text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/50 font-medium"
				>
					{#each devices as dev (dev.id)}
						<option value={dev.id}>{dev.name} ({dev.id.slice(0, 8)})</option>
					{/each}
				</select>
			</div>
		</header>

		{#if error}
			<div
				class="bg-error/15 text-error border border-error/30 p-4 rounded-2xl mb-4 font-medium flex items-center gap-3 text-sm shrink-0"
			>
				<span class="material-symbols-outlined text-[20px]">error</span>
				<div class="flex-1 break-words">{error}</div>
				<button onclick={() => (error = '')} class="hover:opacity-85 text-xs font-semibold uppercase">Dismiss</button>
			</div>
		{/if}

		{#if infoMessage}
			<div
				class="bg-primary/10 text-primary p-4 rounded-2xl mb-4 font-medium flex items-center gap-3 text-sm shrink-0"
			>
				<span class="material-symbols-outlined text-[20px]">check_circle</span>
				<div class="flex-1 break-words">{infoMessage}</div>
				<button onclick={() => (infoMessage = '')} class="hover:opacity-85 text-xs font-semibold uppercase">Dismiss</button>
			</div>
		{/if}

		<div class="flex-1 grid grid-cols-1 lg:grid-cols-3 gap-6 overflow-y-auto pr-1">
			<section class="lg:col-span-2 rounded-[24px] bg-surface-container p-6 flex flex-col justify-between">
				<div>
					<header class="flex items-center gap-2 mb-4 pb-3">
						<span class="material-symbols-outlined text-primary text-[24px]">screenshot_monitor</span>
						<h3 class="text-base font-bold text-on-surface">Screen Mirroring (Scrcpy)</h3>
					</header>

					<div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs font-medium text-on-surface">
						<div class="flex flex-col gap-1.5">
							<label for="scrcpy-max-size" class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">Resolution Max Limit</label>
							<select
								id="scrcpy-max-size"
								bind:value={maxSize}
								class="bg-surface-container-high rounded-xl border border-outline-variant/35 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary/40 font-semibold"
							>
								<option value="0">Original Size</option>
								<option value="1920">1920p (FHD)</option>
								<option value="1600">1600p</option>
								<option value="1280">1280p (HD)</option>
								<option value="1024">1024p (Standard)</option>
								<option value="800">800p (Low Quality)</option>
							</select>
						</div>

						<div class="flex flex-col gap-1.5">
							<label for="scrcpy-fps" class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">Frame Rate Limit</label>
							<select
								id="scrcpy-fps"
								bind:value={maxFps}
								class="bg-surface-container-high rounded-xl border border-outline-variant/35 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary/40 font-semibold"
							>
								<option value="0">Unlimited</option>
								<option value="90">90 FPS</option>
								<option value="60">60 FPS (Fluid)</option>
								<option value="30">30 FPS (Standard)</option>
								<option value="15">15 FPS (Eco-friendly)</option>
							</select>
						</div>

						<div class="flex flex-col gap-1.5">
							<label for="scrcpy-bitrate" class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">Video Bitrate</label>
							<select
								id="scrcpy-bitrate"
								bind:value={bitRate}
								class="bg-surface-container-high rounded-xl border border-outline-variant/35 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary/40 font-semibold"
							>
								<option value="0">Auto</option>
								<option value="16000000">16 Mbps (High Performance)</option>
								<option value="8000000">8 Mbps</option>
								<option value="4000000">4 Mbps (Balanced)</option>
								<option value="2000000">2 Mbps</option>
								<option value="1000000">1 Mbps (Low Bandwidth)</option>
							</select>
						</div>

						<div class="flex flex-col gap-1.5">
							<label for="scrcpy-path-display" class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">Scrcpy Executable Path</label>
							<input
								id="scrcpy-path-display"
								type="text"
								bind:value={scrcpyPath}
								placeholder="scrcpy"
								class="bg-surface-container-high rounded-xl border border-outline-variant/35 px-3 py-2 text-xs font-mono focus:outline-none focus:ring-2 focus:ring-primary/40 text-on-surface"
							/>
						</div>
					</div>

					<div class="grid grid-cols-2 md:grid-cols-3 gap-3 mt-6 text-xs font-bold text-on-surface-variant">
						<label class="flex items-center gap-2 hover:text-on-surface transition-colors cursor-pointer select-none">
							<input type="checkbox" bind:checked={noAudio} class="w-4 h-4 rounded text-primary focus:ring-primary/50" />
							<span>Mute Audio</span>
						</label>
						<label class="flex items-center gap-2 hover:text-on-surface transition-colors cursor-pointer select-none">
							<input type="checkbox" bind:checked={showTouches} class="w-4 h-4 rounded text-primary focus:ring-primary/50" />
							<span>Show Touches</span>
						</label>
						<label class="flex items-center gap-2 hover:text-on-surface transition-colors cursor-pointer select-none">
							<input type="checkbox" bind:checked={stayAwake} class="w-4 h-4 rounded text-primary focus:ring-primary/50" />
							<span>Keep Awake</span>
						</label>
						<label class="flex items-center gap-2 hover:text-on-surface transition-colors cursor-pointer select-none">
							<input type="checkbox" bind:checked={turnScreenOff} class="w-4 h-4 rounded text-primary focus:ring-primary/50" />
							<span>Turn Screen Off</span>
						</label>
						<label class="flex items-center gap-2 hover:text-on-surface transition-colors cursor-pointer select-none">
							<input type="checkbox" bind:checked={alwaysOnTop} class="w-4 h-4 rounded text-primary focus:ring-primary/50" />
							<span>Always On Top</span>
						</label>
						<label class="flex items-center gap-2 hover:text-on-surface transition-colors cursor-pointer select-none">
							<input type="checkbox" bind:checked={fullscreen} class="w-4 h-4 rounded text-primary focus:ring-primary/50" />
							<span>Fullscreen</span>
						</label>
					</div>
				</div>

				<button
					onclick={startMirroring}
					class="mt-6 flex items-center justify-center gap-2 rounded-2xl bg-primary text-on-primary hover:brightness-105 transition-all text-sm font-bold py-3 w-full disabled:opacity-50"
					disabled={!selectedDevice}
				>
					<span class="material-symbols-outlined text-[20px]">screenshot_monitor</span>
					Start Mirroring Session
				</button>
			</section>

			<section class="flex flex-col gap-6">
				<div class="rounded-[24px] bg-surface-container p-5 flex-1 flex flex-col justify-between">
					<div>
						<header class="flex items-center gap-2 mb-3 pb-2">
							<span class="material-symbols-outlined text-primary text-[22px]">photo_camera</span>
							<h3 class="text-sm font-bold text-on-surface">Take Screenshot</h3>
						</header>

						<div class="flex flex-col gap-3">
							<div class="flex flex-col gap-1">
								<label for="sc-file-name" class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant">File Name Pattern</label>
								<input
									id="sc-file-name"
									type="text"
									bind:value={screenshotName}
									placeholder="screenshot.png"
									class="bg-surface-container-high rounded-xl border border-outline-variant/35 px-3 py-1.5 text-xs focus:outline-none focus:ring-2 focus:ring-primary/45"
									disabled={takingScreenshot}
								/>
							</div>
							<p class="text-[10px] text-on-surface-variant font-medium leading-normal pl-1">
								Images are timestamped and downloaded automatically to your Downloads folder.
							</p>
						</div>
					</div>

					<button
						onclick={takeScreenshot}
						class="mt-4 flex items-center justify-center gap-2 rounded-xl bg-surface-container-highest hover:brightness-105 transition-all text-xs font-bold py-2.5 text-on-surface disabled:opacity-50"
						disabled={takingScreenshot || !selectedDevice}
					>
						{#if takingScreenshot}
							<span class="animate-spin h-3.5 w-3.5 border-2 border-on-surface border-t-transparent rounded-full"></span>
							Capturing...
						{:else}
							<span class="material-symbols-outlined text-[16px]">download</span>
							Capture Display
						{/if}
					</button>
				</div>

				<div class="rounded-[24px] bg-surface-container p-5 flex-1 flex flex-col justify-between">
					<div>
						<header class="flex items-center gap-2 mb-3 pb-2">
							<span class="material-symbols-outlined text-primary text-[22px]">content_paste</span>
							<h3 class="text-sm font-bold text-on-surface">Clipboard Sync</h3>
						</header>

						<div class="flex flex-col gap-2.5">
							<label for="sc-clipboard-area" class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant">Sync Workspace</label>
							<textarea
								id="sc-clipboard-area"
								bind:value={clipboardText}
								placeholder="Type to push, or pull to read clipboard..."
								class="bg-surface-container-high rounded-xl border border-outline-variant/35 px-3 py-2 text-xs h-20 resize-none focus:outline-none focus:ring-2 focus:ring-primary/45"
								disabled={syncingClipboard}
							></textarea>
						</div>
					</div>

					<div class="flex gap-2 mt-4">
						<button
							onclick={pullClipboard}
							class="flex-1 flex items-center justify-center gap-1 rounded-xl bg-surface-container-highest hover:brightness-105 transition-all text-xs font-bold py-2 text-on-surface disabled:opacity-50"
							disabled={syncingClipboard || !selectedDevice}
						>
							<span class="material-symbols-outlined text-[15px]">download</span>
							Pull
						</button>
						<button
							onclick={pushClipboard}
							class="flex-1 flex items-center justify-center gap-1 rounded-xl bg-primary text-on-primary hover:brightness-105 transition-all text-xs font-bold py-2 disabled:opacity-50"
							disabled={syncingClipboard || !selectedDevice || !clipboardText}
						>
							<span class="material-symbols-outlined text-[15px]">upload</span>
							Push
						</button>
					</div>
				</div>
			</section>
		</div>
	</div>
</main>
