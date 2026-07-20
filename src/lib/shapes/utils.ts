import { Point } from './point';

export const DistanceEpsilon = 1e-4;
export const AngleEpsilon = 1e-6;

export function convex(previous: Point, current: Point, next: Point): boolean {
	return current.minus(previous).clockwise(next.minus(current));
}

export function interpolate(start: number, stop: number, fraction: number): number {
	return (1 - fraction) * start + fraction * stop;
}

export function directionVector(x: number, y: number): Point {
	const d = distance(x, y);
	return d === 0 ? new Point(0, 0) : new Point(x / d, y / d);
}

export function distance(x: number, y: number): number {
	return Math.sqrt(x * x + y * y);
}

export function distanceSquared(x: number, y: number): number {
	return x * x + y * y;
}

export function radialToCartesian(radius: number, angleRadians: number, center = new Point(0, 0)): Point {
	return new Point(Math.cos(angleRadians), Math.sin(angleRadians))
		.times(radius)
		.plus(center);
}

export function coerceIn(
	value: number,
	min: number | { start: number; endInclusive: number },
	max?: number
): number {
	if (max === undefined) {
		if (typeof min === 'object' && 'start' in min && 'endInclusive' in min) {
			return Math.max(min.start, Math.min(min.endInclusive, value));
		}
		throw new Error('Invalid arguments for coerceIn');
	}
	if (typeof min === 'number') {
		const [actualMin, actualMax] = min <= max ? [min, max] : [max, min];
		return Math.max(actualMin, Math.min(actualMax, value));
	}
	throw new Error('Invalid arguments for coerceIn');
}

export function positiveModulo(value: number, mod: number): number {
	return ((value % mod) + mod) % mod;
}
