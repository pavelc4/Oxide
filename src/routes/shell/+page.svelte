<script lang="ts">
	import { onMount, tick } from 'svelte';

	let invoke: any;
	let isTauri = false;
	let selectedDevice = '';
	let deviceName = '';

	interface HistoryEntry {
		type: 'input' | 'stdout' | 'stderr' | 'info';
		content: string;
		durationMs?: number;
	}

	let history: HistoryEntry[] = [];
	let inputValue = '';
	let isExecuting = false;
	let commandHistory: string[] = [];
	let historyIndex = -1;
	let outputEl: HTMLElement;
	let inputEl: HTMLInputElement;

	onMount(async () => {
		try {
			const tauriApi = await import('@tauri-apps/api/core');
			invoke = tauriApi.invoke;
			isTauri = true;
		} catch (e) {
			console.warn('Tauri not available, using mock mode');
			isTauri = false;
		}

		// Get device serial from URL query or discover first device
		const params = new URLSearchParams(window.location.search);
		selectedDevice = params.get('serial') || '';
		deviceName = params.get('name') || selectedDevice;

		if (!selectedDevice && isTauri && invoke) {
			try {
				const devices = await invoke('get_devices');
				if (devices && devices.length > 0) {
					selectedDevice = devices[0].serial;
					deviceName = devices[0].model || devices[0].serial;
				}
			} catch (e) {
				console.error('Failed to get devices', e);
			}
		}

		if (selectedDevice) {
			history = [
				{ type: 'info', content: `Connected to ${deviceName} (${selectedDevice})` },
				{
					type: 'info',
					content: 'Type a command and press Enter. Use ↑/↓ for history. Type "clear" to reset.'
				}
			];
		} else {
			history = [
				{ type: 'stderr', content: 'No device connected. Go back and connect a device first.' }
			];
		}

		await tick();
		inputEl?.focus();
	});

	async function scrollToBottom() {
		await tick();
		if (outputEl) {
			outputEl.scrollTop = outputEl.scrollHeight;
		}
	}

	async function executeCommand() {
		const cmd = inputValue.trim();
		if (!cmd || isExecuting) return;

		if (commandHistory.length === 0 || commandHistory[commandHistory.length - 1] !== cmd) {
			commandHistory = [...commandHistory, cmd];
			if (commandHistory.length > 100) commandHistory = commandHistory.slice(1);
		}
		historyIndex = -1;

		history = [...history, { type: 'input', content: cmd }];
		inputValue = '';
		isExecuting = true;
		await scrollToBottom();

		if (cmd === 'clear') {
			history = [];
			isExecuting = false;
			return;
		}

		if (cmd === 'exit') {
			window.history.back();
			return;
		}

		try {
			if (invoke && selectedDevice) {
				const result = await invoke('execute_shell_command', {
					serial: selectedDevice,
					command: cmd
				});

				if (result.stdout && result.stdout.length > 0) {
					history = [
						...history,
						{
							type: 'stdout',
							content: result.stdout,
							durationMs: result.duration_ms
						}
					];
				}
				if (result.stderr && result.stderr.length > 0) {
					history = [...history, { type: 'stderr', content: result.stderr }];
				}
				if (!result.stdout && !result.stderr) {
					history = [
						...history,
						{
							type: 'info',
							content: `✓ exit: ${result.exit_code} (${result.duration_ms}ms)`
						}
					];
				}
			} else {
				history = [
					...history,
					{
						type: 'stderr',
						content: isTauri ? 'No device selected' : 'Shell not available in mock mode'
					}
				];
			}
		} catch (e: any) {
			history = [...history, { type: 'stderr', content: String(e) }];
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

	function goBack() {
		window.history.back();
	}
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2 h-screen">
	<div
		class="flex flex-1 flex-col overflow-hidden rounded-[32px] bg-surface-container-low"
	>
		<!-- Header Bar -->
		<header class="flex items-center justify-between px-8 py-5">
			<div class="flex items-center gap-4">
				<button
					on:click={goBack}
					class="flex h-10 w-10 items-center justify-center rounded-full text-on-surface-variant transition-colors hover:bg-surface-variant hover:text-on-surface"
					title="Back to dashboard"
				>
					<span class="material-symbols-outlined text-[22px]">arrow_back</span>
				</button>
				<div class="flex items-center gap-3">
					<div
						class="flex h-10 w-10 items-center justify-center rounded-[14px] bg-primary-container text-on-primary-container"
					>
						<span class="material-symbols-outlined text-[20px]">terminal</span>
					</div>
					<div class="flex flex-col">
						<h1 class="text-lg font-bold text-on-surface tracking-tight">ADB Shell</h1>
						<span class="text-[11px] text-on-surface-variant font-medium">
							{deviceName || 'No device'}
							{#if selectedDevice}
								<span class="opacity-50 ml-1">({selectedDevice})</span>
							{/if}
						</span>
					</div>
				</div>
			</div>
			<div class="flex items-center gap-2">
				{#if !isTauri}
					<span class="text-xs bg-error text-on-error px-3 py-1 rounded-full font-medium"
						>MOCK MODE</span
					>
				{/if}
				<button
					on:click={clearOutput}
					class="flex items-center gap-1.5 rounded-full px-4 py-2 text-xs font-medium text-on-surface-variant transition-colors hover:bg-surface-variant"
					title="Clear output"
				>
					<span class="material-symbols-outlined text-[16px]">delete_sweep</span>
					Clear
				</button>
			</div>
		</header>

		<!-- Terminal Output Area -->
		<div
			bind:this={outputEl}
			class="flex-1 overflow-y-auto px-8 py-6 font-mono text-[13px] leading-[1.7] bg-surface-container-highest/30"
		>
			{#if history.length === 0}
				<div class="flex flex-col items-center justify-center h-full opacity-30">
					<span class="material-symbols-outlined text-[64px] text-on-surface-variant mb-4"
						>terminal</span
					>
					<p class="text-sm text-on-surface-variant font-medium">Start typing a command below</p>
				</div>
			{:else}
				{#each history as entry}
					{#if entry.type === 'input'}
						<div class="flex items-start gap-2 mb-1 mt-3 first:mt-0">
							<span class="text-primary font-bold select-none shrink-0">$</span>
							<span class="text-on-surface font-semibold">{entry.content}</span>
						</div>
					{:else if entry.type === 'stdout'}
						<div class="mb-1 pl-5">
							<pre
								class="text-on-surface/85 whitespace-pre-wrap break-all m-0 text-[12.5px]">{entry.content}</pre>
							{#if entry.durationMs !== undefined}
								<span class="text-[10px] text-on-surface-variant/50 font-medium"
									>{entry.durationMs}ms</span
								>
							{/if}
						</div>
					{:else if entry.type === 'stderr'}
						<div class="mb-1 pl-5">
							<pre
								class="text-error whitespace-pre-wrap break-all m-0 text-[12.5px]">{entry.content}</pre>
						</div>
					{:else if entry.type === 'info'}
						<div class="mb-1 text-on-surface-variant/60 italic text-[11.5px] pl-5">
							{entry.content}
						</div>
					{/if}
				{/each}

				{#if isExecuting}
					<div class="flex items-center gap-2 text-on-surface-variant mt-2 pl-5">
						<span class="inline-block h-2.5 w-2.5 rounded-full bg-primary animate-pulse"></span>
						<span class="text-xs font-medium">Executing...</span>
					</div>
				{/if}
			{/if}
		</div>

		<!-- Input Bar -->
		<div
			class="flex items-center gap-3 px-8 py-4 bg-surface-container/80"
		>
			<span class="text-primary font-bold font-mono text-sm select-none">$</span>
			<input
				bind:this={inputEl}
				bind:value={inputValue}
				on:keydown={handleKeydown}
				type="text"
				placeholder={isExecuting
					? 'Executing...'
					: selectedDevice
						? 'Type a command...'
						: 'No device connected'}
				disabled={isExecuting || !selectedDevice}
				autocomplete="off"
				spellcheck="false"
				class="flex-1 bg-transparent font-mono text-[13px] text-on-surface placeholder:text-on-surface-variant/40 focus:outline-none disabled:opacity-40"
			/>
			<button
				on:click={executeCommand}
				disabled={isExecuting || !inputValue.trim()}
				class="flex h-9 w-9 items-center justify-center rounded-full bg-primary text-on-primary transition-all hover:brightness-110 disabled:opacity-25 disabled:cursor-not-allowed shadow-sm"
				title="Run command (Enter)"
			>
				<span class="material-symbols-outlined text-[16px]">send</span>
			</button>
		</div>
	</div>
</main>
