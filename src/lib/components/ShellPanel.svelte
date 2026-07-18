<script lang="ts">
	import { onMount, tick } from 'svelte';

	interface Props {
		serial: string;
		invoke: any;
		onClose: () => void;
	}

	let { serial, invoke, onClose }: Props = $props();

	interface HistoryEntry {
		type: 'input' | 'stdout' | 'stderr' | 'info';
		content: string;
		durationMs?: number;
	}

	let history: HistoryEntry[] = $state([
		{ type: 'info', content: `ADB Shell connected to ${serial}` },
		{ type: 'info', content: 'Type a command and press Enter. Use ↑/↓ for history.' }
	]);
	let inputValue = $state('');
	let isExecuting = $state(false);
	let commandHistory: string[] = $state([]);
	let historyIndex = $state(-1);
	let outputEl: HTMLElement;
	let inputEl: HTMLInputElement;

	async function scrollToBottom() {
		await tick();
		if (outputEl) {
			outputEl.scrollTop = outputEl.scrollHeight;
		}
	}

	async function executeCommand() {
		const cmd = inputValue.trim();
		if (!cmd || isExecuting) return;

		// Push to command history (dedup consecutive)
		if (commandHistory.length === 0 || commandHistory[commandHistory.length - 1] !== cmd) {
			commandHistory.push(cmd);
			if (commandHistory.length > 100) commandHistory.shift();
		}
		historyIndex = -1;

		history.push({ type: 'input', content: cmd });
		inputValue = '';
		isExecuting = true;
		await scrollToBottom();

		// Handle built-in commands
		if (cmd === 'clear') {
			history = [];
			isExecuting = false;
			return;
		}

		try {
			if (invoke) {
				const result = await invoke('execute_shell_command', {
					serial: serial,
					command: cmd
				});

				if (result.stdout && result.stdout.length > 0) {
					history.push({
						type: 'stdout',
						content: result.stdout,
						durationMs: result.duration_ms
					});
				}
				if (result.stderr && result.stderr.length > 0) {
					history.push({ type: 'stderr', content: result.stderr });
				}
				if (!result.stdout && !result.stderr) {
					history.push({
						type: 'info',
						content: `Command completed (exit: ${result.exit_code}, ${result.duration_ms}ms)`
					});
				}
			} else {
				history.push({
					type: 'stderr',
					content: 'Shell not available in mock mode'
				});
			}
		} catch (e: any) {
			history.push({ type: 'stderr', content: String(e) });
		} finally {
			isExecuting = false;
			await scrollToBottom();
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

	onMount(() => {
		inputEl?.focus();
	});
</script>

<section
	class="mt-4 flex flex-col rounded-[24px] bg-surface-container shadow-lg overflow-hidden animate-in"
	style="animation: shellSlideIn 0.3s cubic-bezier(0.2, 0, 0, 1) forwards;"
>
	<!-- Header -->
	<header class="flex items-center justify-between px-6 py-4 border-b border-outline/20">
		<div class="flex items-center gap-3">
			<div
				class="flex h-9 w-9 items-center justify-center rounded-[12px] bg-primary-container text-on-primary-container"
			>
				<span class="material-symbols-outlined text-[18px]">terminal</span>
			</div>
			<div class="flex flex-col">
				<h3 class="text-sm font-bold text-on-surface tracking-tight">ADB Shell</h3>
				<span class="text-[11px] text-on-surface-variant font-medium">{serial}</span>
			</div>
		</div>
		<div class="flex items-center gap-2">
			<button
				onclick={clearOutput}
				class="flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium text-on-surface-variant transition-colors hover:bg-surface-variant"
				title="Clear output"
			>
				<span class="material-symbols-outlined text-[14px]">delete_sweep</span>
				Clear
			</button>
			<button
				onclick={onClose}
				class="flex h-8 w-8 items-center justify-center rounded-full text-on-surface-variant transition-colors hover:bg-surface-variant hover:text-on-surface"
				title="Close shell"
			>
				<span class="material-symbols-outlined text-[18px]">close</span>
			</button>
		</div>
	</header>

	<!-- Terminal Output -->
	<div
		bind:this={outputEl}
		class="flex-1 overflow-y-auto px-6 py-4 font-mono text-[13px] leading-relaxed max-h-[400px] min-h-[200px] bg-surface-container-highest/40"
	>
		{#each history as entry}
			{#if entry.type === 'input'}
				<div class="flex items-start gap-2 mb-1">
					<span class="text-primary font-bold select-none shrink-0">$</span>
					<span class="text-on-surface font-semibold">{entry.content}</span>
				</div>
			{:else if entry.type === 'stdout'}
				<div class="mb-2">
					<pre class="text-on-surface/90 whitespace-pre-wrap break-all m-0">{entry.content}</pre>
					{#if entry.durationMs !== undefined}
						<span class="text-[10px] text-on-surface-variant/60 font-medium"
							>{entry.durationMs}ms</span
						>
					{/if}
				</div>
			{:else if entry.type === 'stderr'}
				<div class="mb-2">
					<pre class="text-error whitespace-pre-wrap break-all m-0">{entry.content}</pre>
				</div>
			{:else if entry.type === 'info'}
				<div class="mb-1 text-on-surface-variant/70 italic text-[12px]">
					{entry.content}
				</div>
			{/if}
		{/each}

		{#if isExecuting}
			<div class="flex items-center gap-2 text-on-surface-variant">
				<span class="inline-block h-3 w-3 rounded-full bg-primary animate-pulse"></span>
				<span class="text-xs">Executing...</span>
			</div>
		{/if}
	</div>

	<!-- Input Bar -->
	<div
		class="flex items-center gap-3 border-t border-outline/20 px-6 py-3 bg-surface-container-high/60"
	>
		<span class="text-primary font-bold font-mono text-sm select-none">$</span>
		<input
			bind:this={inputEl}
			bind:value={inputValue}
			onkeydown={handleKeydown}
			type="text"
			placeholder={isExecuting ? 'Executing...' : 'Type a command...'}
			disabled={isExecuting}
			autocomplete="off"
			spellcheck="false"
			class="flex-1 bg-transparent font-mono text-[13px] text-on-surface placeholder:text-on-surface-variant/50 focus:outline-none disabled:opacity-50"
		/>
		<button
			onclick={executeCommand}
			disabled={isExecuting || !inputValue.trim()}
			class="flex h-8 w-8 items-center justify-center rounded-full bg-primary text-on-primary transition-all hover:brightness-110 disabled:opacity-30 disabled:cursor-not-allowed"
			title="Run command"
		>
			<span class="material-symbols-outlined text-[16px]">send</span>
		</button>
	</div>
</section>

<style>
	@keyframes shellSlideIn {
		from {
			opacity: 0;
			transform: translateY(-12px);
			max-height: 0;
		}
		to {
			opacity: 1;
			transform: translateY(0);
			max-height: 600px;
		}
	}
</style>
