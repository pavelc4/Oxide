<script lang="ts">
	/* eslint-disable svelte/no-navigation-without-resolve */
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | undefined;
	let isTauri = $state(false);

	interface Device {
		id: string;
		name: string;
		status: string;
		connection: string;
	}

	// State
	let devices = $state<Device[]>([]);
	let selectedDevice = $state<string>('');
	let loading = $state(true);
	let error = $state('');
	let infoMessage = $state('');

	// Connection form
	let connectIp = $state('192.168.1.');
	let connectPort = $state('5555');
	let connecting = $state(false);

	// TCP/IP form
	let tcpipPort = $state('5555');
	let activatingTcpip = $state(false);

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
	});

	async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
		if (isTauri && invoke) {
			return (await invoke(cmd, args)) as T;
		}
		throw new Error('Tauri API not active');
	}

	async function loadDevices() {
		loading = true;
		error = '';
		try {
			if (isTauri && invoke) {
				const rustDevices = await safeInvoke<Array<{ serial: string; model?: string }>>('get_devices');
				if (rustDevices) {
					devices = rustDevices.map((d) => ({
						id: d.serial,
						name: d.model || d.serial,
						status: 'Online',
						connection: d.serial.includes('.') ? 'Wireless' : 'USB'
					}));
				} else {
					devices = [];
				}
			} else {
				devices = [
					{ id: 'usb-device-123', name: 'Mock Galaxy S24 (USB)', status: 'Online', connection: 'USB' }
				];
			}

			const usbDev = devices.find((d) => d.connection === 'USB');
			if (usbDev) {
				selectedDevice = usbDev.id;
			} else if (devices.length > 0) {
				selectedDevice = devices[0].id;
			} else {
				selectedDevice = '';
			}
		} catch (e) {
			error = String(e);
			devices = [];
		} finally {
			loading = false;
		}
	}

	async function connectWireless() {
		if (!connectIp.trim() || !connectPort.trim()) return;
		connecting = true;
		error = '';
		infoMessage = '';
		
		const portNum = parseInt(connectPort.trim());
		const ipAddr = connectIp.trim();

		try {
			if (isTauri && invoke) {
				await safeInvoke('connect_wireless', {
					ip: ipAddr,
					port: portNum
				});
			} else {
				await new Promise((resolve) => setTimeout(resolve, 1500));
			}
			infoMessage = `Successfully connected to ${ipAddr}:${portNum} over Wireless ADB.`;
			await loadDevices();
		} catch (e) {
			error = `Failed to connect: ${e}`;
		} finally {
			connecting = false;
		}
	}

	async function disconnectWireless() {
		if (!connectIp.trim() || !connectPort.trim()) return;
		connecting = true;
		error = '';
		infoMessage = '';

		const portNum = parseInt(connectPort.trim());
		const ipAddr = connectIp.trim();

		try {
			if (isTauri && invoke) {
				await safeInvoke('disconnect_wireless', {
					ip: ipAddr,
					port: portNum
				});
			} else {
				await new Promise((resolve) => setTimeout(resolve, 1000));
			}
			infoMessage = `Disconnected from ${ipAddr}:${portNum} successfully.`;
			await loadDevices();
		} catch (e) {
			error = `Failed to disconnect: ${e}`;
		} finally {
			connecting = false;
		}
	}

	async function enableTcpipMode() {
		if (!selectedDevice || !tcpipPort.trim()) return;
		activatingTcpip = true;
		error = '';
		infoMessage = '';

		const portNum = parseInt(tcpipPort.trim());

		try {
			if (isTauri && invoke) {
				await safeInvoke('enable_wireless_tcpip', {
					serial: selectedDevice,
					port: portNum
				});
			} else {
				await new Promise((resolve) => setTimeout(resolve, 1500));
			}
			infoMessage = `TCP/IP port ${portNum} enabled on device successfully! You can now unplug the USB cable and connect using its IP address.`;
		} catch (e) {
			error = `Failed to enable TCP/IP: ${e}`;
		} finally {
			activatingTcpip = false;
		}
	}
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2 h-screen overflow-hidden">
	<div
		class="flex flex-1 flex-col overflow-hidden rounded-[32px] bg-surface-container-low p-6 lg:p-10 relative shadow-sm"
	>
		<!-- Page Header -->
		<header class="mb-6 flex justify-between items-center shrink-0">
			<div class="flex items-center gap-4">
				<button
					onclick={() => goto('/')}
					class="flex h-10 w-10 items-center justify-center rounded-full bg-surface-container hover:bg-surface-container-high text-on-surface-variant transition-all hover:scale-105 active:scale-95"
					title="Back to dashboard"
				>
					<span class="material-symbols-outlined text-[20px]">arrow_back</span>
				</button>
				<div>
					<div class="flex items-center gap-3">
						<h2 class="text-2xl font-bold tracking-tight text-on-surface">Wireless ADB Setup</h2>
						{#if !isTauri}
							<span class="text-[10px] bg-warning/15 text-warning px-2.5 py-0.5 rounded-full font-bold uppercase tracking-wider">MOCK PREVIEW</span>
						{/if}
					</div>
					<p class="text-xs text-on-surface-variant/80 font-medium mt-0.5">Pair & connect Android devices wirelessly over Wi-Fi network</p>
				</div>
			</div>

			<button
				onclick={loadDevices}
				class="flex h-9 w-9 items-center justify-center rounded-full bg-surface-container hover:bg-surface-container-high text-on-surface-variant transition-all hover:scale-105 active:scale-95 shadow-xs"
				title="Refresh USB Devices"
			>
				<span class="material-symbols-outlined text-[18px] {loading ? 'animate-spin' : ''}">refresh</span>
			</button>
		</header>

		<!-- Alert Messages -->
		{#if error}
			<div class="bg-error/15 text-error p-3.5 rounded-2xl mb-4 font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[20px]">error</span>
				<div class="flex-1 break-words font-semibold">{error}</div>
				<button onclick={() => (error = '')} class="hover:opacity-80 text-[10px] font-bold uppercase tracking-wider bg-error/20 px-2.5 py-1 rounded-lg">Dismiss</button>
			</div>
		{/if}

		{#if infoMessage}
			<div class="bg-primary/10 text-primary p-3.5 rounded-2xl mb-4 font-medium flex items-center gap-3 text-xs shrink-0 animate-fade-in">
				<span class="material-symbols-outlined text-[20px]">check_circle</span>
				<div class="flex-1 break-words font-semibold">{infoMessage}</div>
				<button onclick={() => (infoMessage = '')} class="hover:opacity-80 text-[10px] font-bold uppercase tracking-wider bg-primary/20 px-2.5 py-1 rounded-lg">Dismiss</button>
			</div>
		{/if}

		<!-- Grid Layout -->
		<div class="flex-1 grid grid-cols-1 md:grid-cols-2 gap-6 overflow-y-auto pr-1">
			<!-- Col 1: Connect Form (Borderless Card) -->
			<section class="rounded-[24px] bg-surface-container p-6 flex flex-col justify-between h-[300px] shadow-sm">
				<div>
					<header class="flex items-center gap-2 pb-2.5 mb-4">
						<span class="material-symbols-outlined text-primary text-[22px]">settings_input_antenna</span>
						<h3 class="text-sm font-bold text-on-surface">Connect via IP Address</h3>
					</header>

					<div class="grid grid-cols-3 gap-3 text-xs font-medium mb-4">
						<div class="col-span-2 flex flex-col gap-1.5">
							<label for="wireless-ip" class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant pl-0.5">Device IP Address</label>
							<input
								id="wireless-ip"
								type="text"
								bind:value={connectIp}
								placeholder="e.g. 192.168.1.10"
								class="bg-surface-container-high rounded-xl px-3.5 py-2.5 text-xs text-on-surface font-mono focus:outline-none focus:ring-2 focus:ring-primary/40"
								disabled={connecting}
							/>
						</div>
						<div class="flex flex-col gap-1.5">
							<label for="wireless-port" class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant pl-0.5">Port</label>
							<input
								id="wireless-port"
								type="text"
								bind:value={connectPort}
								placeholder="5555"
								class="bg-surface-container-high rounded-xl px-3.5 py-2.5 text-xs text-on-surface font-mono focus:outline-none focus:ring-2 focus:ring-primary/40"
								disabled={connecting}
							/>
						</div>
					</div>

					<p class="text-[10px] text-on-surface-variant font-medium leading-relaxed pl-0.5">
						Ensure your computer and the Android device are connected to the same Wi-Fi network.
					</p>
				</div>

				<div class="flex gap-3">
					<button
						onclick={disconnectWireless}
						class="flex-1 rounded-xl bg-surface-container-highest hover:brightness-105 py-2.5 text-xs font-bold text-on-surface disabled:opacity-50 transition-all"
						disabled={connecting || !connectIp.trim()}
					>
						Disconnect
					</button>
					<button
						onclick={connectWireless}
						class="flex-1 rounded-xl bg-primary text-on-primary hover:brightness-105 py-2.5 text-xs font-bold disabled:opacity-50 transition-all shadow-sm"
						disabled={connecting || !connectIp.trim() || !connectPort.trim()}
					>
						{#if connecting}
							Connecting...
						{:else}
							Connect Device
						{/if}
					</button>
				</div>
			</section>

			<!-- Col 2: Enable TCP/IP Mode over USB (Borderless Card) -->
			<section class="rounded-[24px] bg-surface-container p-6 flex flex-col justify-between h-[300px] shadow-sm">
				<div>
					<header class="flex items-center gap-2 pb-2.5 mb-4">
						<span class="material-symbols-outlined text-primary text-[22px]">usb</span>
						<h3 class="text-sm font-bold text-on-surface">Enable TCP/IP port (Requires USB)</h3>
					</header>

					<div class="grid grid-cols-3 gap-3 text-xs font-medium mb-4">
						<div class="col-span-2 flex flex-col gap-1.5">
							<span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant pl-0.5">USB Connected Device</span>
							<select
								bind:value={selectedDevice}
								class="bg-surface-container-high rounded-xl px-3.5 py-2.5 text-xs text-on-surface focus:outline-none font-semibold cursor-pointer"
								disabled={activatingTcpip || devices.length === 0}
							>
								{#if devices.length === 0}
									<option value="">No USB Devices connected</option>
								{:else}
									{#each devices as dev (dev.id)}
										<option value={dev.id}>{dev.name} ({dev.id.slice(0, 8)})</option>
									{/each}
								{/if}
							</select>
						</div>
						<div class="flex flex-col gap-1.5">
							<label for="tcpip-port-val" class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant pl-0.5">Port</label>
							<input
								id="tcpip-port-val"
								type="text"
								bind:value={tcpipPort}
								placeholder="5555"
								class="bg-surface-container-high rounded-xl px-3.5 py-2.5 text-xs text-on-surface font-mono focus:outline-none focus:ring-2 focus:ring-primary/40"
								disabled={activatingTcpip}
							/>
						</div>
					</div>

					<p class="text-[10px] text-on-surface-variant font-medium leading-relaxed pl-0.5">
						Connect the device via USB first, specify a port (usually 5555), and click below. Once activated, you can safely unplug the cable.
					</p>
				</div>

				<button
					onclick={enableTcpipMode}
					class="rounded-xl bg-primary text-on-primary hover:brightness-105 py-2.5 text-xs font-bold disabled:opacity-50 w-full transition-all shadow-sm"
					disabled={activatingTcpip || !selectedDevice || !tcpipPort.trim()}
				>
					{#if activatingTcpip}
						Activating Port...
					{:else}
						Enable TCP/IP Port
					{/if}
				</button>
			</section>
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
