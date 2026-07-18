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
			const tauriApi = await import('@tauri-apps/api/core');
			invoke = tauriApi.invoke;
			isTauri = true;
		} catch {
			console.warn('Tauri not available, using mock mode');
			isTauri = false;
		}

		await loadDevices();
	});

	async function loadDevices() {
		loading = true;
		error = '';
		try {
			if (isTauri && invoke) {
				const rustDevices = (await invoke('get_devices')) as Array<{ serial: string; model?: string }>;
				if (rustDevices) {
					// We only care about USB connected devices for TCP/IP enablement
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

			// Pre-select USB device
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
				await invoke('connect_wireless', {
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
				await invoke('disconnect_wireless', {
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
				await invoke('enable_wireless_tcpip', {
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
					Wireless ADB Setup
					{#if !isTauri}
						<span class="text-xs bg-error text-on-error px-3 py-1 rounded-full font-medium tracking-normal border border-error/30">MOCK MODE</span>
					{/if}
				</h2>
			</div>

			<button
				onclick={loadDevices}
				class="flex h-8 w-8 items-center justify-center rounded-full bg-surface-container-highest text-on-surface-variant hover:text-on-surface transition-all hover:scale-105 active:scale-95"
				title="Refresh USB Devices"
			>
				<span class="material-symbols-outlined text-[18px] {loading ? 'animate-spin' : ''}">refresh</span>
			</button>
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

		<!-- Grid Layout -->
		<div class="flex-1 grid grid-cols-1 md:grid-cols-2 gap-6 overflow-y-auto pr-1">
			<!-- Col 1: Connect Form -->
			<section class="rounded-[24px] bg-surface-container p-6 flex flex-col justify-between h-[300px]">
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
								class="bg-surface-container-high border border-outline-variant/40 rounded-xl px-3 py-2 text-sm text-on-surface font-mono"
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
								class="bg-surface-container-high border border-outline-variant/40 rounded-xl px-3 py-2 text-sm text-on-surface font-mono"
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
						class="flex-1 rounded-xl bg-surface-container-highest hover:brightness-105 py-2.5 text-xs font-bold text-on-surface disabled:opacity-50"
						disabled={connecting || !connectIp.trim()}
					>
						Disconnect
					</button>
					<button
						onclick={connectWireless}
						class="flex-1 rounded-xl bg-primary text-on-primary hover:brightness-105 py-2.5 text-xs font-bold disabled:opacity-50"
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

			<!-- Col 2: Enable TCP/IP Mode over USB -->
			<section class="rounded-[24px] bg-surface-container p-6 flex flex-col justify-between h-[300px]">
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
								class="bg-surface-container-high rounded-xl border border-outline-variant px-3 py-2 text-sm text-on-surface focus:outline-none font-semibold"
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
								class="bg-surface-container-high border border-outline-variant/40 rounded-xl px-3 py-2 text-sm text-on-surface font-mono"
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
					class="rounded-xl bg-primary text-on-primary hover:brightness-105 py-2.5 text-xs font-bold disabled:opacity-50 w-full"
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
