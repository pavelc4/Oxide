<script lang="ts">
	let { data = [0], color = '#4ADE80', height = 60 }: { data: number[]; color?: string; height?: number } = $props();

	const viewBoxWidth = 300;
	const viewBoxHeight = 100;

	const gradientId = $derived(`sparkline-grad-${color.replace(/[^a-zA-Z0-9]/g, '')}`);

	let points = $derived.by(() => {
		if (!data || data.length < 2) return [];
		const len = data.length;
		const max = Math.max(...data, 1);
		const min = Math.min(...data, 0);
		const range = max - min || 1;

		return data.map((val, i) => {
			const x = (i / (len - 1)) * viewBoxWidth;
			const y = viewBoxHeight - ((val - min) / range) * (viewBoxHeight - 16) - 8;
			return { x: Number(x.toFixed(1)), y: Number(y.toFixed(1)) };
		});
	});

	let linePathStr = $derived.by(() => {
		if (points.length < 2) return '';
		return points.reduce((acc, p, i) => (i === 0 ? `M ${p.x},${p.y}` : `${acc} L ${p.x},${p.y}`), '');
	});

	let areaPathStr = $derived.by(() => {
		if (points.length < 2) return '';
		const lastX = points[points.length - 1].x;
		return `${linePathStr} L ${lastX},${viewBoxHeight} L 0,${viewBoxHeight} Z`;
	});
</script>

<div class="w-full overflow-hidden rounded-xl relative" style="height: {height}px;">
	<svg
		viewBox="0 0 {viewBoxWidth} {viewBoxHeight}"
		preserveAspectRatio="none"
		class="w-full h-full block overflow-hidden"
	>
		<defs>
			<linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
				<stop offset="0%" stop-color={color} stop-opacity="0.3" />
				<stop offset="100%" stop-color={color} stop-opacity="0.0" />
			</linearGradient>
		</defs>

		{#if areaPathStr}
			<path d={areaPathStr} fill="url(#{gradientId})" />
		{/if}

		{#if linePathStr}
			<path
				d={linePathStr}
				fill="none"
				stroke={color}
				stroke-width="2.5"
				stroke-linecap="round"
				stroke-linejoin="round"
				vector-effect="non-scaling-stroke"
			/>
		{/if}
	</svg>
</div>
