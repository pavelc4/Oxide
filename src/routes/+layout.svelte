<script lang="ts">
	import '../app.css';
	import Sidebar from '$lib/components/Sidebar.svelte';
	let { children } = $props();
</script>

<div class="flex h-screen w-full bg-surface">
	<Sidebar />
	{@render children()}
</div>
