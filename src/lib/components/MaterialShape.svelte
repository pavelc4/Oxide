<script lang="ts">
	import { getMaterialShapeSvgPath, type MaterialShapeType } from '$lib/shapes/materialShapes';

	interface Props {
		shape?: MaterialShapeType;
		size?: number;
		color?: string;
		class?: string;
	}

	let {
		shape = 'cookie7',
		size = 48,
		color = 'currentColor',
		class: className = ''
	}: Props = $props();

	let svgPath = $derived(getMaterialShapeSvgPath(shape, size));
</script>

<svg
	width={size}
	height={size}
	viewBox="0 0 {size} {size}"
	class="inline-block transition-transform duration-300 hover:scale-105 {className}"
>
	<path d={svgPath} fill={color} />
</svg>
