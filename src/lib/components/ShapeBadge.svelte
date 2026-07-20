<script lang="ts">
	import { getMaterialShapeSvgPath, type MaterialShapeType } from '$lib/shapes/materialShapes';

	interface Props {
		icon?: string;
		imgUrl?: string;
		shape?: MaterialShapeType | 'rounded';
		size?: number;
		iconSize?: number;
		variant?: 'primary' | 'container' | 'surface' | 'accent';
		class?: string;
	}

	let {
		icon = '',
		imgUrl = '',
		shape = 'cookie7',
		size = 44,
		iconSize = 24,
		variant = 'container',
		class: className = ''
	}: Props = $props();

	let svgPath = $derived(
		shape === 'rounded' ? '' : getMaterialShapeSvgPath(shape as MaterialShapeType, size)
	);

	// Pair shape fill & icon color using exact Material 3 dynamic CSS custom properties
	let shapeBgColor = $derived(
		variant === 'primary'
			? 'var(--primary)'
			: variant === 'surface'
			? 'var(--surfaceContainerHighest)'
			: variant === 'accent'
			? 'var(--secondaryContainer)'
			: 'var(--primaryContainer)'
	);

	let iconTextColor = $derived(
		variant === 'primary'
			? 'var(--onPrimary)'
			: variant === 'surface'
			? 'var(--onSurface)'
			: variant === 'accent'
			? 'var(--onSecondaryContainer, var(--onSurface))'
			: 'var(--onPrimaryContainer)'
	);

	let clipId = $derived(`clip-${(imgUrl || shape).replace(/[^a-zA-Z0-9]/g, '')}-${size}`);
</script>

{#if shape === 'rounded'}
	<div
		class="flex items-center justify-center rounded-2xl shrink-0 overflow-hidden {className}"
		style="width: {size}px; height: {size}px; background-color: {shapeBgColor}; color: {iconTextColor};"
	>
		{#if imgUrl}
			<img src={imgUrl} alt="" class="w-full h-full object-cover" />
		{:else if icon}
			<span class="material-symbols-outlined" style="font-size: {iconSize}px;">{icon}</span>
		{/if}
	</div>
{:else}
	<div
		class="relative flex items-center justify-center shrink-0 {className}"
		style="width: {size}px; height: {size}px; color: {iconTextColor};"
	>
		<svg
			width={size}
			height={size}
			viewBox="0 0 {size} {size}"
			class="absolute inset-0 w-full h-full transition-transform duration-300 hover:scale-105 overflow-visible"
		>
			{#if imgUrl}
				<defs>
					<clipPath id={clipId}>
						<path d={svgPath} />
					</clipPath>
				</defs>
				<image
					href={imgUrl}
					width={size}
					height={size}
					clip-path="url(#{clipId})"
					preserveAspectRatio="xMidYMid slice"
				/>
			{:else}
				<path d={svgPath} style="fill: {shapeBgColor};" />
			{/if}
		</svg>
		{#if !imgUrl && icon}
			<span class="material-symbols-outlined relative z-10" style="font-size: {iconSize}px; color: {iconTextColor};">{icon}</span>
		{/if}
	</div>
{/if}
