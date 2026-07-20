import { Offset } from './offset';
import { Matrix } from './matrix';
import { CornerRounding } from './cornerRounding';
import { RoundedPolygon, roundedPolygonToSvgPath } from './roundedPolygon';

export class PointNRound {
	o: Offset;
	r: CornerRounding;

	constructor(o: Offset, r = CornerRounding.Unrounded) {
		this.o = o;
		this.r = r;
	}
}

const cornerRound15 = new CornerRounding(0.15);
const cornerRound20 = new CornerRounding(0.2);
const cornerRound30 = new CornerRounding(0.3);
const cornerRound50 = new CornerRounding(0.5);
const cornerRound100 = new CornerRounding(1.0);

export const rotateNeg30 = new Matrix(); rotateNeg30.rotateZ(-30);
export const rotateNeg45 = new Matrix(); rotateNeg45.rotateZ(-45);
export const rotateNeg90 = new Matrix(); rotateNeg90.rotateZ(-90);
export const rotateNeg135 = new Matrix(); rotateNeg135.rotateZ(-135);
export const rotate30 = new Matrix(); rotate30.rotateZ(30);
export const rotate45 = new Matrix(); rotate45.rotateZ(45);
export const rotate60 = new Matrix(); rotate60.rotateZ(60);
export const rotate90 = new Matrix(); rotate90.rotateZ(90);
export const rotate120 = new Matrix(); rotate120.rotateZ(120);
export const rotate135 = new Matrix(); rotate135.rotateZ(135);
export const rotate180 = new Matrix(); rotate180.rotateZ(180);

export function doRepeat(
	points: PointNRound[],
	reps: number,
	center: Offset,
	mirroring = false
): PointNRound[] {
	if (mirroring) {
		const result: PointNRound[] = [];
		const angles = points.map((p) => p.o.minus(center).angleDegrees());
		const distances = points.map((p) => p.o.minus(center).getDistance());
		const actualReps = reps * 2;
		const sectionAngle = 360 / actualReps;
		for (let it = 0; it < actualReps; it++) {
			for (let index = 0; index < points.length; index++) {
				const i = it % 2 === 0 ? index : points.length - 1 - index;
				if (i > 0 || it % 2 === 0) {
					const baseAngle = angles[i];
					const angle =
						it * sectionAngle + (it % 2 === 0 ? baseAngle : 2 * angles[0] - baseAngle);
					const dist = distances[i];
					const rad = (angle * Math.PI) / 180;
					const x = center.x + dist * Math.cos(rad);
					const y = center.y + dist * Math.sin(rad);
					result.push(new PointNRound(new Offset(x, y), points[i].r));
				}
			}
		}
		return result;
	} else {
		const np = points.length;
		const result: PointNRound[] = [];
		for (let i = 0; i < np * reps; i++) {
			const point = points[i % np].o.rotateDegrees(Math.floor(i / np) * (360 / reps), center);
			result.push(new PointNRound(point, points[i % np].r));
		}
		return result;
	}
}

export function customPolygon(
	pnr: PointNRound[],
	reps = 1,
	center = new Offset(0.5, 0.5),
	mirroring = false
): RoundedPolygon {
	const actualPoints = doRepeat(pnr, reps, center, mirroring);
	const vertices: number[] = [];
	for (const p of actualPoints) {
		vertices.push(p.o.x);
		vertices.push(p.o.y);
	}
	const perVertexRounding = actualPoints.map((p) => p.r);
	return RoundedPolygon.fromVertices(
		vertices,
		CornerRounding.Unrounded,
		perVertexRounding,
		center.x,
		center.y
	);
}

// Key Material 3 Expressive Shapes
export function getCircle(): RoundedPolygon {
	return RoundedPolygon.circle(8, 0.5, 0.5, 0.5).normalized();
}

export function getSquare(): RoundedPolygon {
	return RoundedPolygon.rectangle(1, 1, cornerRound30, null, 0.5, 0.5).normalized();
}

export function getSlanted(): RoundedPolygon {
	return customPolygon([
		new PointNRound(new Offset(0.0, 0.0), cornerRound30),
		new PointNRound(new Offset(1.0, 0.15), cornerRound30),
		new PointNRound(new Offset(1.0, 1.0), cornerRound30),
		new PointNRound(new Offset(0.0, 0.85), cornerRound30)
	]).normalized();
}

export function getArch(): RoundedPolygon {
	return customPolygon([
		new PointNRound(new Offset(0.0, 0.0), cornerRound100),
		new PointNRound(new Offset(1.0, 0.0), cornerRound100),
		new PointNRound(new Offset(1.0, 1.0), cornerRound20),
		new PointNRound(new Offset(0.0, 1.0), cornerRound20)
	]).normalized();
}

export function getFan(): RoundedPolygon {
	return customPolygon([
		new PointNRound(new Offset(0.0, 0.0), cornerRound20),
		new PointNRound(new Offset(1.0, 0.0), cornerRound100),
		new PointNRound(new Offset(1.0, 1.0), cornerRound100),
		new PointNRound(new Offset(0.0, 1.0), cornerRound100)
	]).normalized();
}

export function getArrow(): RoundedPolygon {
	return customPolygon([
		new PointNRound(new Offset(0.5, 0.0), cornerRound30),
		new PointNRound(new Offset(1.0, 0.5), cornerRound30),
		new PointNRound(new Offset(0.8, 0.5), cornerRound15),
		new PointNRound(new Offset(0.8, 1.0), cornerRound30),
		new PointNRound(new Offset(0.2, 1.0), cornerRound30),
		new PointNRound(new Offset(0.2, 0.5), cornerRound15),
		new PointNRound(new Offset(0.0, 0.5), cornerRound30)
	]).normalized();
}

export function getSunny(): RoundedPolygon {
	return customPolygon([
		new PointNRound(new Offset(0.5, 0.0), cornerRound100),
		new PointNRound(new Offset(0.63, 0.12), cornerRound15),
		new PointNRound(new Offset(0.85, 0.15), cornerRound100),
		new PointNRound(new Offset(0.88, 0.37), cornerRound15)
	], 4).normalized();
}

export function getGhostish(): RoundedPolygon {
	return customPolygon([
		new PointNRound(new Offset(0.5, 0.0), cornerRound100),
		new PointNRound(new Offset(1.0, 0.5), cornerRound100),
		new PointNRound(new Offset(0.8, 0.95), cornerRound30),
		new PointNRound(new Offset(0.5, 0.8), cornerRound30),
		new PointNRound(new Offset(0.2, 0.95), cornerRound30),
		new PointNRound(new Offset(0.0, 0.5), cornerRound100)
	]).normalized();
}

export function getCookie7Sided(): RoundedPolygon {
	return customPolygon([
		new PointNRound(new Offset(0.5, 0.0), cornerRound50),
		new PointNRound(new Offset(0.72, 0.1), cornerRound15)
	], 7).normalized();
}

export function getGem(): RoundedPolygon {
	return customPolygon([
		new PointNRound(new Offset(0.5, 0.0), cornerRound30),
		new PointNRound(new Offset(1.0, 0.35), cornerRound30),
		new PointNRound(new Offset(0.8, 1.0), cornerRound30),
		new PointNRound(new Offset(0.2, 1.0), cornerRound30),
		new PointNRound(new Offset(0.0, 0.35), cornerRound30)
	]).normalized();
}

export function getClamShell(): RoundedPolygon {
	return customPolygon([
		new PointNRound(new Offset(0.5, 0.0), cornerRound100),
		new PointNRound(new Offset(1.0, 0.6), cornerRound50),
		new PointNRound(new Offset(0.75, 1.0), cornerRound20),
		new PointNRound(new Offset(0.25, 1.0), cornerRound20),
		new PointNRound(new Offset(0.0, 0.6), cornerRound50)
	]).normalized();
}

export function getHeart(): RoundedPolygon {
	return customPolygon([
		new PointNRound(new Offset(0.5, 0.25), new CornerRounding(0.016)),
		new PointNRound(new Offset(0.792, -0.066), new CornerRounding(0.958)),
		new PointNRound(new Offset(1.064, 0.276), new CornerRounding(1.0)),
		new PointNRound(new Offset(0.782, 0.611), new CornerRounding(0.5)),
		new PointNRound(new Offset(0.5, 0.946), new CornerRounding(0.1)),
		new PointNRound(new Offset(0.2175, 0.611), new CornerRounding(0.5)),
		new PointNRound(new Offset(-0.064, 0.276), new CornerRounding(1.0)),
		new PointNRound(new Offset(0.208, -0.066), new CornerRounding(0.958))
	]).normalized();
}

export function getClover4Leaf(): RoundedPolygon {
	return customPolygon([
		new PointNRound(new Offset(0.5, 0.0), cornerRound100),
		new PointNRound(new Offset(0.65, 0.35), cornerRound15)
	], 4).normalized();
}

export function getBurst(): RoundedPolygon {
	return customPolygon([
		new PointNRound(new Offset(0.5, 0.0), cornerRound15),
		new PointNRound(new Offset(0.6, 0.2), cornerRound15)
	], 8).normalized();
}

export function getBun(): RoundedPolygon {
	return customPolygon([
		new PointNRound(new Offset(0.5, 0.0), cornerRound100),
		new PointNRound(new Offset(0.95, 0.4), cornerRound50),
		new PointNRound(new Offset(0.75, 0.95), cornerRound30),
		new PointNRound(new Offset(0.25, 0.95), cornerRound30),
		new PointNRound(new Offset(0.05, 0.4), cornerRound50)
	]).normalized();
}

export function getTriangle(): RoundedPolygon {
	return customPolygon([
		new PointNRound(new Offset(0.5, 0.0), cornerRound50),
		new PointNRound(new Offset(1.0, 0.9), cornerRound30),
		new PointNRound(new Offset(0.0, 0.9), cornerRound30)
	]).normalized();
}

export function getDiamond(): RoundedPolygon {
	return customPolygon([
		new PointNRound(new Offset(0.5, 0.0), cornerRound30),
		new PointNRound(new Offset(1.0, 0.5), cornerRound30),
		new PointNRound(new Offset(0.5, 1.0), cornerRound30),
		new PointNRound(new Offset(0.0, 0.5), cornerRound30)
	]).normalized();
}

export function getHexagon(): RoundedPolygon {
	return customPolygon([
		new PointNRound(new Offset(0.5, 0.0), cornerRound30),
		new PointNRound(new Offset(0.93, 0.25), cornerRound30),
		new PointNRound(new Offset(0.93, 0.75), cornerRound30),
		new PointNRound(new Offset(0.5, 1.0), cornerRound30),
		new PointNRound(new Offset(0.07, 0.75), cornerRound30),
		new PointNRound(new Offset(0.07, 0.25), cornerRound30)
	]).normalized();
}

export function getScallop(): RoundedPolygon {
	return customPolygon([
		new PointNRound(new Offset(0.5, 0.0), cornerRound100),
		new PointNRound(new Offset(0.62, 0.08), cornerRound15)
	], 12).normalized();
}

export function getPixelCircle(): RoundedPolygon {
	const raw = [
		{ x: 0.35, y: 0.0 }, { x: 0.65, y: 0.0 },
		{ x: 0.65, y: 0.12 }, { x: 0.88, y: 0.12 },
		{ x: 0.88, y: 0.35 }, { x: 1.0, y: 0.35 },
		{ x: 1.0, y: 0.65 }, { x: 0.88, y: 0.65 },
		{ x: 0.88, y: 0.88 }, { x: 0.65, y: 0.88 },
		{ x: 0.65, y: 1.0 }, { x: 0.35, y: 1.0 },
		{ x: 0.35, y: 0.88 }, { x: 0.12, y: 0.88 },
		{ x: 0.12, y: 0.65 }, { x: 0.0, y: 0.65 },
		{ x: 0.0, y: 0.35 }, { x: 0.12, y: 0.35 },
		{ x: 0.12, y: 0.12 }, { x: 0.35, y: 0.12 }
	];
	const pts = raw.map((p) => new PointNRound(new Offset(p.x, p.y), CornerRounding.Unrounded));
	return customPolygon(pts).normalized();
}

export type MaterialShapeType =
	| 'circle'
	| 'square'
	| 'slanted'
	| 'arch'
	| 'fan'
	| 'arrow'
	| 'sunny'
	| 'ghostish'
	| 'cookie7'
	| 'gem'
	| 'clamshell'
	| 'heart'
	| 'clover'
	| 'burst'
	| 'bun'
	| 'triangle'
	| 'diamond'
	| 'hexagon'
	| 'scallop'
	| 'pixelCircle';

export function getMaterialShape(type: MaterialShapeType): RoundedPolygon {
	switch (type) {
		case 'circle': return getCircle();
		case 'square': return getSquare();
		case 'slanted': return getSlanted();
		case 'arch': return getArch();
		case 'fan': return getFan();
		case 'arrow': return getArrow();
		case 'sunny': return getSunny();
		case 'ghostish': return getGhostish();
		case 'cookie7': return getCookie7Sided();
		case 'gem': return getGem();
		case 'clamshell': return getClamShell();
		case 'heart': return getHeart();
		case 'clover': return getClover4Leaf();
		case 'burst': return getBurst();
		case 'bun': return getBun();
		case 'triangle': return getTriangle();
		case 'diamond': return getDiamond();
		case 'hexagon': return getHexagon();
		case 'scallop': return getScallop();
		case 'pixelCircle': return getPixelCircle();
		default: return getCircle();
	}
}

export function getMaterialShapeSvgPath(type: MaterialShapeType, size = 100): string {
	const polygon = getMaterialShape(type);
	return roundedPolygonToSvgPath(polygon, size);
}
