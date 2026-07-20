<script lang="ts">
	import '../app.css';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { initGlobalTheme } from '$lib/theme';
	import { onMount } from 'svelte';

	let { children } = $props();

	onMount(() => {
		initGlobalTheme();
	});
</script>

<div class="flex h-screen w-full bg-surface">
	<Sidebar />
	{@render children()}
</div>
