<script lang="ts">
	import { onMount } from 'svelte';

	let { data = [0], color = '#4ADE80', height = 60 }: { data: number[]; color?: string; height?: number } = $props();

	let canvas: HTMLCanvasElement;
	let ctx: CanvasRenderingContext2D | null = null;

	function draw() {
		if (!ctx || !canvas) return;
		let w = canvas.width;
		let h = canvas.height;
		let len = data.length;
		if (len < 2) return;

		ctx.clearRect(0, 0, w, h);

		let max = Math.max(...data, 1);
		let min = Math.min(...data, 0);
		let range = max - min || 1;

		let points: { x: number; y: number }[] = [];
		for (let i = 0; i < len; i++) {
			let x = (i / (len - 1)) * w;
			let y = h - ((data[i] - min) / range) * (h - 4) - 2;
			points.push({ x, y });
		}

		ctx.beginPath();
		ctx.moveTo(points[0].x, points[0].y);
		for (let i = 1; i < points.length; i++) {
			ctx.lineTo(points[i].x, points[i].y);
		}
		ctx.strokeStyle = color;
		ctx.lineWidth = 1.5;
		ctx.lineJoin = 'round';
		ctx.lineCap = 'round';
		ctx.stroke();

		ctx.lineTo(points[points.length - 1].x, h);
		ctx.lineTo(points[0].x, h);
		ctx.closePath();
		ctx.fillStyle = color;
		ctx.globalAlpha = 0.1;
		ctx.fill();
		ctx.globalAlpha = 1;
	}

	onMount(() => {
		let dpr = window.devicePixelRatio || 1;
		let rect = canvas.parentElement?.getBoundingClientRect();
		let w = rect ? rect.width : 120;
		canvas.width = w * dpr;
		canvas.height = height * dpr;
		canvas.style.width = w + 'px';
		canvas.style.height = height + 'px';
		ctx = canvas.getContext('2d');
		ctx?.scale(dpr, dpr);
		draw();
	});

	$effect(() => {
		if (data.length) draw();
	});
</script>

<canvas bind:this={canvas} class="w-full" style="height: {height}px"></canvas>
