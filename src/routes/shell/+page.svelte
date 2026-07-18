<script lang="ts">
	/* eslint-disable svelte/no-navigation-without-resolve */
	import { onMount, tick } from 'svelte';
	import { goto } from '$app/navigation';

	let invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | undefined;
	let isTauri = $state(false);

	interface HistoryEntry {
		id: number;
		type: 'input' | 'stdout' | 'stderr' | 'info';
		content: string;
		durationMs?: number;
		timestamp?: string;
	}

	interface Device {
		id: string;
		name: string;
	}

	let devices = $state<Device[]>([]);
	let selectedDevice = $state('');
	let deviceName = $state('');
	let loading = $state(false);
	let error = $state('');
	let infoMessage = $state('');
	let autoScroll = $state(true);

	let history = $state<HistoryEntry[]>([]);
	let inputValue = $state('');
	let isExecuting = $state(false);
	let commandHistory = $state<string[]>([]);
	let historyIndex = $state(-1);
	let logCounter = 1;

	let outputEl: HTMLDivElement | undefined;
	let inputEl: HTMLInputElement | undefined;

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

		await loadDevices();

		if (selectedDevice) {
			history = [
				{ id: logCounter++, type: 'info', content: `Connected interactive ADB shell to ${deviceName} (${selectedDevice})`, timestamp: getNowTime() },
				{ id: logCounter++, type: 'info', content: 'Type shell commands below or click quick command presets. Use ↑/↓ for command history.', timestamp: getNowTime() }
			];
		}

		await tick();
		inputEl?.focus();
	});

	function getNowTime(): string {
		return new Date().toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
	}

	async function loadDevices() {
		loading = true;
		try {
			if (isTauri && invoke) {
				const rustDevices = await safeInvoke<Array<{ serial: string; model?: string }>>('get_devices');
				if (rustDevices && rustDevices.length > 0) {
					devices = rustDevices.map((d) => ({ id: d.serial, name: d.model || d.serial }));
				} else {
					devices = [];
				}
			} else {
				devices = [
					{ id: 'fastboot-pixel-8', name: 'Google Pixel 8 Pro' },
					{ id: 'adb-galaxy-s24', name: 'Samsung Galaxy S24 Ultra' }
				];
			}

			if (devices.length > 0) {
				selectedDevice = devices[0].id;
				deviceName = devices[0].name;
			} else {
				selectedDevice = '';
				deviceName = 'No Device Connected';
			}
		} catch (e) {
			error = String(e);
			devices = [];
		} finally {
			loading = false;
		}
	}

	async function scrollToBottom() {
		if (!autoScroll) return;
		await tick();
		if (outputEl) {
			outputEl.scrollTop = outputEl.scrollHeight;
		}
	}

	async function executeCommand(cmdToRun?: string) {
		const cmd = (cmdToRun || inputValue).trim();
		if (!cmd || isExecuting) return;

		if (commandHistory.length === 0 || commandHistory[commandHistory.length - 1] !== cmd) {
			commandHistory = [...commandHistory, cmd];
			if (commandHistory.length > 100) commandHistory = commandHistory.slice(1);
		}
		historyIndex = -1;

		history = [...history, { id: logCounter++, type: 'input', content: cmd, timestamp: getNowTime() }];
		if (!cmdToRun) inputValue = '';
		isExecuting = true;
		await scrollToBottom();

		if (cmd === 'clear') {
			history = [];
			isExecuting = false;
			return;
		}

		try {
			let output = '';
			const startTime = Date.now();

			if (isTauri && invoke && selectedDevice) {
				const result = await safeInvoke<{ stdout?: string; stderr?: string; exit_code?: number; duration_ms?: number }>('execute_shell_command', {
					serial: selectedDevice,
					command: cmd
				});

				const duration = result.duration_ms || (Date.now() - startTime);

				if (result.stdout && result.stdout.length > 0) {
					history = [...history, { id: logCounter++, type: 'stdout', content: result.stdout, durationMs: duration, timestamp: getNowTime() }];
				}
				if (result.stderr && result.stderr.length > 0) {
					history = [...history, { id: logCounter++, type: 'stderr', content: result.stderr, timestamp: getNowTime() }];
				}
				if (!result.stdout && !result.stderr) {
					history = [...history, { id: logCounter++, type: 'info', content: `✓ exit code: ${result.exit_code || 0} (${duration}ms)`, timestamp: getNowTime() }];
				}
			} else {
				// Realistic Mock Shell Output Emulator
				await new Promise((r) => setTimeout(r, 350));
				const duration = Date.now() - startTime;

				if (cmd.includes('getprop ro.product.model')) {
					output = 'Pixel 8 Pro';
				} else if (cmd.includes('getprop')) {
					output = '[ro.product.model]: [Pixel 8 Pro]\n[ro.product.brand]: [google]\n[ro.build.version.release]: [14]\n[ro.build.version.sdk]: [34]\n[ro.product.cpu.abi]: [arm64-v8a]';
				} else if (cmd.includes('dumpsys battery')) {
					output = 'Current Battery Service state:\n  AC powered: false\n  USB powered: true\n  status: 2 (Charging)\n  health: 2 (Good)\n  level: 92\n  scale: 100\n  voltage: 4312mV\n  temperature: 314 (31.4°C)';
				} else if (cmd.includes('pm list packages')) {
					output = 'package:com.android.chrome\npackage:com.whatsapp\npackage:com.spotify.music\npackage:com.instagram.android\npackage:com.mobile.legends\npackage:com.google.android.youtube';
				} else if (cmd.includes('df')) {
					output = 'Filesystem           1K-blocks      Used Available Use% Mounted on\n/dev/block/dm-0      124912000  62456000  62456000  50% /data\n/dev/block/by-name/sdcard 256000000 112000000 144000000 44% /sdcard';
				} else if (cmd.includes('cat /proc/meminfo')) {
					output = 'MemTotal:        8192000 kB\nMemFree:         2456000 kB\nMemAvailable:    3890000 kB\nBuffers:          189000 kB\nCached:          2100000 kB';
				} else if (cmd.includes('wm size')) {
					output = 'Physical size: 1440x3120';
				} else if (cmd.includes('ip addr')) {
					output = '1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN\n    inet 127.0.0.1/8 scope host lo\n2: wlan0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc mq state UP\n    inet 192.168.1.104/24 brd 192.168.1.255 scope global wlan0';
				} else if (cmd.includes('whoami') || cmd.includes('id')) {
					output = 'uid=2000(shell) gid=2000(shell) groups=2000(shell),1004(input),1007(log),1011(adb),1015(sdcard_rw)';
				} else if (cmd.includes('uname')) {
					output = 'Linux localhost 6.1.57-android14-11-g9a02b #1 SMP PREEMPT Wed Apr 15 2026 x86_64 Android';
				} else {
					output = `Command '${cmd}' executed successfully.\nOKAY [ 0.042s ]`;
				}

				history = [...history, { id: logCounter++, type: 'stdout', content: output, durationMs: duration, timestamp: getNowTime() }];
			}
		} catch (e) {
			history = [...history, { id: logCounter++, type: 'stderr', content: String(e), timestamp: getNowTime() }];
		} finally {
			isExecuting = false;
			await scrollToBottom();
			inputEl?.focus();
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			executeCommand();
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			if (commandHistory.length === 0) return;
			if (historyIndex === -1) {
				historyIndex = commandHistory.length - 1;
			} else if (historyIndex > 0) {
				historyIndex--;
			}
			inputValue = commandHistory[historyIndex];
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			if (historyIndex === -1) return;
			if (historyIndex < commandHistory.length - 1) {
				historyIndex++;
				inputValue = commandHistory[historyIndex];
			} else {
				historyIndex = -1;
				inputValue = '';
			}
		}
	}

	function clearOutput() {
		history = [];
	}

	function copySingleOutput(content: string) {
		navigator.clipboard.writeText(content);
		infoMessage = 'Output payload copied to clipboard!';
	}

	function copyTerminalOutput() {
		if (history.length === 0) return;
		const text = history.map((h) => (h.type === 'input' ? `$ ${h.content}` : h.content)).join('\n');
		navigator.clipboard.writeText(text);
		infoMessage = 'Terminal transcript copied to clipboard!';
	}

	function exportTerminalLog() {
		if (history.length === 0) return;
		const text = history.map((h) => `[${h.timestamp || ''}] ${h.type === 'input' ? '$ ' : ''}${h.content}`).join('\n');
		const blob = new Blob([text], { type: 'text/plain;charset=utf-8' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `oxide_shell_log_${selectedDevice || 'device'}_${Date.now()}.txt`;
		a.click();
		URL.revokeObjectURL(url);
		infoMessage = 'Terminal transcript log file exported successfully!';
	}
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2 h-screen overflow-hidden">
	<div class="flex flex-1 flex-col overflow-hidden rounded-[32px] bg-surface-container-low p-6 lg:p-8 relative gap-5 shadow-sm">
		
		<!-- Page Header -->
		<header class="flex flex-col md:flex-row md:items-center justify-between gap-4 shrink-0 pb-2">
			<div class="flex items-center gap-3">
				<button
					onclick={() => goto('/')}
					class="flex h-10 w-10 items-center justify-center rounded-xl bg-surface-container hover:bg-surface-container-high text-on-surface-variant transition-all hover:scale-105 active:scale-95 shadow-xs"
					title="Back to dashboard"
				>
					<span class="material-symbols-outlined text-[20px]">arrow_back</span>
				</button>
				<div>
					<div class="flex items-center gap-3">
						<h2 class="text-2xl font-bold tracking-tight text-on-surface">ADB Shell Studio</h2>
						{#if !isTauri}
							<span class="text-[10px] bg-warning/15 text-warning px-2.5 py-0.5 rounded-full font-bold uppercase tracking-wider">MOCK MODE</span>
						{/if}
					</div>
					<div class="flex items-center gap-2 text-xs text-on-surface-variant/80 font-medium mt-0.5">
						<span>Target: <strong class="text-on-surface">{deviceName}</strong> ({selectedDevice || 'No Device'})</span>
						<span class="inline-block h-1.5 w-1.5 rounded-full bg-emerald-500"></span>
						<span class="text-[11px] text-emerald-400 font-mono">12ms latency</span>
					</div>
				</div>
			</div>

			<!-- Quick Toolbar Controls -->
			<div class="flex items-center gap-2.5">
				<!-- Auto-scroll Toggle Pill -->
				<button
					onclick={() => (autoScroll = !autoScroll)}
					class="flex items-center gap-1.5 rounded-full px-3.5 py-1.5 text-xs font-bold transition-all shadow-xs {autoScroll ? 'bg-primary/15 text-primary' : 'bg-surface-container text-on-surface-variant hover:bg-surface-container-high'}"
					title="Toggle Auto-Scroll"
				>
					<span class="material-symbols-outlined text-[15px]">{autoScroll ? 'vertical_align_bottom' : 'pause'}</span>
					{autoScroll ? 'Auto-scroll On' : 'Scroll Paused'}
				</button>

				<!-- Export Log File -->
				<button
					onclick={exportTerminalLog}
					class="flex items-center gap-1.5 rounded-full bg-surface-container hover:bg-surface-container-high px-3.5 py-1.5 text-xs font-bold text-on-surface transition-all shadow-xs"
					title="Export Terminal Log File (.txt)"
				>
					<span class="material-symbols-outlined text-[15px]">download</span>
					Export Log
				</button>

				<!-- Copy Transcript -->
				<button
					onclick={copyTerminalOutput}
					class="flex items-center gap-1.5 rounded-full bg-surface-container hover:bg-surface-container-high px-3.5 py-1.5 text-xs font-bold text-on-surface transition-all shadow-xs"
					title="Copy Terminal Text"
				>
					<span class="material-symbols-outlined text-[15px]">content_copy</span>
					Copy All
				</button>

				<!-- Clear Output -->
				<button
					onclick={clearOutput}
					class="flex items-center gap-1.5 rounded-full bg-surface-container hover:bg-surface-container-high px-3.5 py-1.5 text-xs font-bold text-on-surface hover:text-error transition-all shadow-xs"
					title="Clear Terminal Output"
				>
					<span class="material-symbols-outlined text-[15px]">delete_sweep</span>
					Clear
				</button>
			</div>
		</header>

		<!-- Alert Banner -->
		{#if infoMessage}
			<div class="bg-primary/10 text-primary p-3 rounded-2xl font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[18px]">check_circle</span>
				<div class="flex-1 font-semibold">{infoMessage}</div>
				<button onclick={() => (infoMessage = '')} class="hover:opacity-80 text-[10px] font-bold uppercase">Dismiss</button>
			</div>
		{/if}



		<!-- Oxide Material 3 Terminal Output Console Container (With Per-Output Copy Button!) -->
		<div
			bind:this={outputEl}
			class="flex-1 bg-surface-container rounded-[32px] p-6 font-mono text-xs overflow-y-auto flex flex-col gap-3 shadow-sm leading-relaxed select-text min-h-0"
		>
			{#if history.length === 0}
				<div class="flex flex-col items-center justify-center h-full text-on-surface-variant/60 py-12">
					<span class="material-symbols-outlined text-[48px] opacity-40 mb-3">terminal</span>
					<p class="text-xs font-semibold">ADB Shell session initialized. Type commands below.</p>
				</div>
			{:else}
				{#each history as entry (entry.id)}
					{#if entry.type === 'input'}
						<div class="flex items-center justify-between mt-2 first:mt-0 font-mono">
							<div class="flex items-center gap-2">
								<span class="text-primary font-bold select-none">$ adb shell</span>
								<span class="text-on-surface font-bold">{entry.content}</span>
							</div>
							{#if entry.timestamp}
								<span class="text-[10px] text-on-surface-variant/40 font-mono select-none">{entry.timestamp}</span>
							{/if}
						</div>
					{:else if entry.type === 'stdout'}
						<div class="pl-4">
							<div class="bg-surface-container-high/60 p-4 rounded-2xl flex flex-col gap-2 relative">
								<div class="flex items-center justify-between text-[10px] text-on-surface-variant/70 font-mono pb-1">
									<span>Standard Output Buffer {#if entry.durationMs !== undefined}({entry.durationMs}ms){/if}</span>
									<button
										onclick={() => copySingleOutput(entry.content)}
										class="flex items-center gap-1.5 text-[10px] font-bold bg-surface-container hover:bg-primary hover:text-on-primary text-on-surface px-2.5 py-1 rounded-lg transition-all shadow-xs"
										title="Copy this output payload"
									>
										<span class="material-symbols-outlined text-[13px]">content_copy</span>
										Copy Output
									</button>
								</div>
								<pre class="text-on-surface/90 font-mono text-xs whitespace-pre-wrap break-all leading-relaxed m-0">{entry.content}</pre>
							</div>
						</div>
					{:else if entry.type === 'stderr'}
						<div class="pl-4">
							<div class="bg-error/10 p-4 rounded-2xl flex flex-col gap-2 font-semibold">
								<div class="flex items-center justify-between text-[10px] text-error font-mono pb-1">
									<span>Error Stream Output</span>
									<button
										onclick={() => copySingleOutput(entry.content)}
										class="flex items-center gap-1.5 text-[10px] font-bold bg-error/20 hover:bg-error text-error hover:text-on-error px-2.5 py-1 rounded-lg transition-all shadow-xs"
										title="Copy error text"
									>
										<span class="material-symbols-outlined text-[13px]">content_copy</span>
										Copy Error
									</button>
								</div>
								<pre class="text-error font-mono text-xs whitespace-pre-wrap break-all leading-relaxed m-0">{entry.content}</pre>
							</div>
						</div>
					{:else if entry.type === 'info'}
						<div class="pl-4 text-on-surface-variant/70 italic text-[11px]">
							{entry.content}
						</div>
					{/if}
				{/each}

				{#if isExecuting}
					<div class="flex items-center gap-2 text-primary pl-4 py-1">
						<span class="inline-block h-2.5 w-2.5 rounded-full bg-primary animate-pulse"></span>
						<span class="text-xs font-bold font-mono">Executing command on {deviceName}...</span>
					</div>
				{/if}
			{/if}
		</div>

		<!-- Interactive Command Input Bar -->
		<form
			onsubmit={(e) => { e.preventDefault(); executeCommand(); }}
			class="flex items-center gap-3 bg-surface-container rounded-full px-5 py-2.5 shadow-sm shrink-0"
		>
			<span class="text-primary font-bold font-mono text-xs select-none">$ adb shell</span>
			<input
				bind:this={inputEl}
				bind:value={inputValue}
				onkeydown={handleKeydown}
				type="text"
				placeholder={isExecuting ? 'Executing command...' : 'Type shell command (e.g. getprop, pm list packages)...'}
				disabled={isExecuting}
				autocomplete="off"
				spellcheck="false"
				class="flex-1 bg-transparent font-mono text-xs text-on-surface placeholder:text-on-surface-variant/50 focus:outline-none disabled:opacity-40"
			/>
			<button
				type="submit"
				disabled={isExecuting || !inputValue.trim()}
				class="flex h-9 w-9 items-center justify-center rounded-full bg-primary text-on-primary hover:brightness-110 transition-all shadow-sm disabled:opacity-40 shrink-0"
				title="Run Shell Command"
			>
				<span class="material-symbols-outlined text-[16px]">send</span>
			</button>
		</form>

	</div>
</main>

<style>
	@keyframes fadeIn {
		from { opacity: 0; transform: scale(0.99); }
		to { opacity: 1; transform: scale(1); }
	}
	.animate-fade-in { animation: fadeIn 0.15s cubic-bezier(0.2, 0.8, 0.2, 1) forwards; }
</style>
