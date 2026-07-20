import { Point } from './point';
import { Cubic } from './cubic';
import { Feature, Corner, Edge } from './feature';
import { CornerRounding } from './cornerRounding';
import { RoundedCorner } from './roundedCorner';
import * as Utils from './utils';

export class RoundedPolygon {
	features: Feature[];
	center: Point;
	cubics: Cubic[];

	constructor(features: Feature[], center: Point) {
		this.features = features;
		this.center = center;
		this.cubics = this.buildCubicList();
	}

	get centerX(): number {
		return this.center.x;
	}

	get centerY(): number {
		return this.center.y;
	}

	transformed(f: (x: number, y: number) => { x: number; y: number }): RoundedPolygon {
		const center = this.center.transformed(f);
		return new RoundedPolygon(
			this.features.map((x) => x.transformed(f)),
			center
		);
	}

	normalized(padding = 0.05): RoundedPolygon {
		const bounds = this.calculateBounds();
		const minX = bounds[0];
		const minY = bounds[1];
		const maxX = bounds[2];
		const maxY = bounds[3];
		const width = maxX - minX || 1;
		const height = maxY - minY || 1;

		const side = Math.max(width, height);
		const availableSize = 1 - 2 * padding;
		const scale = availableSize / side;

		const offsetX = padding + (availableSize - width * scale) / 2 - minX * scale;
		const offsetY = padding + (availableSize - height * scale) / 2 - minY * scale;

		return this.transformed((x, y) => {
			return new Point(x * scale + offsetX, y * scale + offsetY);
		});
	}

	calculateMaxBounds(bounds: number[] = []): number[] {
		let maxDistSquared = 0;
		for (let i = 0; i < this.cubics.length; i++) {
			const cubic = this.cubics[i];
			const anchorDistance = Utils.distanceSquared(
				cubic.anchor0X - this.centerX,
				cubic.anchor0Y - this.centerY
			);
			const middlePoint = cubic.pointOnCurve(0.5);
			const middleDistance = Utils.distanceSquared(
				middlePoint.x - this.centerX,
				middlePoint.y - this.centerY
			);
			maxDistSquared = Math.max(maxDistSquared, Math.max(anchorDistance, middleDistance));
		}
		const dist = Math.sqrt(maxDistSquared);
		bounds[0] = this.centerX - dist;
		bounds[1] = this.centerY - dist;
		bounds[2] = this.centerX + dist;
		bounds[3] = this.centerY + dist;
		return bounds;
	}

	calculateBounds(bounds: number[] = [], approximate = true): number[] {
		let minX = Number.MAX_SAFE_INTEGER;
		let minY = Number.MAX_SAFE_INTEGER;
		let maxX = Number.MIN_SAFE_INTEGER;
		let maxY = Number.MIN_SAFE_INTEGER;
		for (let i = 0; i < this.cubics.length; i++) {
			const cubic = this.cubics[i];
			cubic.calculateBounds(bounds, approximate);
			minX = Math.min(minX, bounds[0]);
			minY = Math.min(minY, bounds[1]);
			maxX = Math.max(maxX, bounds[2]);
			maxY = Math.max(maxY, bounds[3]);
		}
		bounds[0] = minX;
		bounds[1] = minY;
		bounds[2] = maxX;
		bounds[3] = maxY;
		return bounds;
	}

	buildCubicList(): Cubic[] {
		const result: Cubic[] = [];
		let firstCubic: Cubic | null = null;
		let lastCubic: Cubic | null = null;
		let firstFeatureSplitStart: Cubic[] | null = null;
		let firstFeatureSplitEnd: Cubic[] | null = null;

		if (this.features.length > 0 && this.features[0].cubics.length === 3) {
			const centerCubic = this.features[0].cubics[1];
			const { a: start, b: end } = centerCubic.split(0.5);
			firstFeatureSplitStart = [this.features[0].cubics[0], start];
			firstFeatureSplitEnd = [end, this.features[0].cubics[2]];
		}

		for (let i = 0; i <= this.features.length; i++) {
			let featureCubics: Cubic[];
			if (i === 0 && firstFeatureSplitEnd != null) {
				featureCubics = firstFeatureSplitEnd;
			} else if (i === this.features.length) {
				if (firstFeatureSplitStart != null) {
					featureCubics = firstFeatureSplitStart;
				} else {
					break;
				}
			} else {
				featureCubics = this.features[i].cubics;
			}

			for (let j = 0; j < featureCubics.length; j++) {
				const cubic = featureCubics[j];
				if (!cubic.zeroLength()) {
					if (lastCubic != null) result.push(lastCubic);
					lastCubic = cubic;
					if (firstCubic == null) firstCubic = cubic;
				} else {
					if (lastCubic != null) {
						lastCubic = new Cubic([...lastCubic.points]);
						lastCubic.points[6] = cubic.anchor1X;
						lastCubic.points[7] = cubic.anchor1Y;
					}
				}
			}
		}

		if (lastCubic != null && firstCubic != null) {
			result.push(
				new Cubic([
					lastCubic.anchor0X,
					lastCubic.anchor0Y,
					lastCubic.control0X,
					lastCubic.control0Y,
					lastCubic.control1X,
					lastCubic.control1Y,
					firstCubic.anchor0X,
					firstCubic.anchor0Y
				])
			);
		} else {
			result.push(
				new Cubic([
					this.centerX,
					this.centerY,
					this.centerX,
					this.centerY,
					this.centerX,
					this.centerY,
					this.centerX,
					this.centerY
				])
			);
		}

		return result;
	}

	static calculateCenter(vertices: number[]): Point {
		let cumulativeX = 0;
		let cumulativeY = 0;
		let index = 0;
		while (index < vertices.length) {
			cumulativeX += vertices[index++];
			cumulativeY += vertices[index++];
		}
		return new Point(cumulativeX / (vertices.length / 2), cumulativeY / (vertices.length / 2));
	}

	static verticesFromNumVerts(numVertices: number, radius: number, centerX: number, centerY: number): number[] {
		const result: number[] = [];
		let arrayIndex = 0;
		for (let i = 0; i < numVertices; i++) {
			const vertex = Utils.radialToCartesian(radius, (Math.PI / numVertices) * 2 * i).plus(
				new Point(centerX, centerY)
			);
			result[arrayIndex++] = vertex.x;
			result[arrayIndex++] = vertex.y;
		}
		return result;
	}

	static fromNumVertices(
		numVertices: number,
		radius = 1,
		centerX = 0,
		centerY = 0,
		rounding = CornerRounding.Unrounded,
		perVertexRounding: CornerRounding[] | null = null
	): RoundedPolygon {
		return RoundedPolygon.fromVertices(
			this.verticesFromNumVerts(numVertices, radius, centerX, centerY),
			rounding,
			perVertexRounding,
			centerX,
			centerY
		);
	}

	static fromVertices(
		vertices: number[],
		rounding = CornerRounding.Unrounded,
		perVertexRounding: CornerRounding[] | null = null,
		centerX = Number.MIN_SAFE_INTEGER,
		centerY = Number.MAX_SAFE_INTEGER
	): RoundedPolygon {
		const corners: Cubic[][] = [];
		const n = vertices.length / 2;
		const roundedCorners: RoundedCorner[] = [];

		for (let i = 0; i < n; i++) {
			const vtxRounding = perVertexRounding?.[i] ?? rounding;
			const prevIndex = ((i + n - 1) % n) * 2;
			const nextIndex = ((i + 1) % n) * 2;
			roundedCorners.push(
				new RoundedCorner(
					new Point(vertices[prevIndex], vertices[prevIndex + 1]),
					new Point(vertices[i * 2], vertices[i * 2 + 1]),
					new Point(vertices[nextIndex], vertices[nextIndex + 1]),
					vtxRounding
				)
			);
		}

		const cutAdjusts = Array.from({ length: n }).map((_, ix) => {
			const expectedRoundCut =
				roundedCorners[ix].expectedRoundCut + roundedCorners[(ix + 1) % n].expectedRoundCut;
			const expectedCut =
				roundedCorners[ix].expectedCut + roundedCorners[(ix + 1) % n].expectedCut;
			const vtxX = vertices[ix * 2];
			const vtxY = vertices[ix * 2 + 1];
			const nextVtxX = vertices[((ix + 1) % n) * 2];
			const nextVtxY = vertices[((ix + 1) % n) * 2 + 1];
			const sideSize = Utils.distance(vtxX - nextVtxX, vtxY - nextVtxY);

			if (expectedRoundCut > sideSize) {
				return { a: sideSize / expectedRoundCut, b: 0 };
			} else if (expectedCut > sideSize) {
				return { a: 1, b: (sideSize - expectedRoundCut) / (expectedCut - expectedRoundCut) };
			} else {
				return { a: 1, b: 1 };
			}
		});

		for (let i = 0; i < n; i++) {
			const allowedCuts: number[] = [];
			for (const delta of [0, 1]) {
				const { a: roundCutRatio, b: cutRatio } = cutAdjusts[(i + n - 1 + delta) % n];
				allowedCuts.push(
					roundedCorners[i].expectedRoundCut * roundCutRatio +
						(roundedCorners[i].expectedCut - roundedCorners[i].expectedRoundCut) * cutRatio
				);
			}
			corners.push(roundedCorners[i].getCubics(allowedCuts[0], allowedCuts[1]));
		}

		const tempFeatures: Feature[] = [];
		for (let i = 0; i < n; i++) {
			const prevVtxIndex = (i + n - 1) % n;
			const nextVtxIndex = (i + 1) % n;
			const currVertex = new Point(vertices[i * 2], vertices[i * 2 + 1]);
			const prevVertex = new Point(vertices[prevVtxIndex * 2], vertices[prevVtxIndex * 2 + 1]);
			const nextVertex = new Point(vertices[nextVtxIndex * 2], vertices[nextVtxIndex * 2 + 1]);
			const cnvx = Utils.convex(prevVertex, currVertex, nextVertex);
			tempFeatures.push(new Corner(corners[i], cnvx));
			tempFeatures.push(
				new Edge([
					Cubic.straightLine(
						corners[i][corners[i].length - 1].anchor1X,
						corners[i][corners[i].length - 1].anchor1Y,
						corners[(i + 1) % n][0].anchor0X,
						corners[(i + 1) % n][0].anchor0Y
					)
				])
			);
		}

		let center: Point;
		if (centerX === Number.MIN_SAFE_INTEGER || centerY === Number.MIN_SAFE_INTEGER) {
			center = RoundedPolygon.calculateCenter(vertices);
		} else {
			center = new Point(centerX, centerY);
		}

		return RoundedPolygon.fromFeatures(tempFeatures, center.x, center.y);
	}

	static fromFeatures(features: Feature[], centerX: number, centerY: number): RoundedPolygon {
		const vertices: number[] = [];
		for (const feature of features) {
			for (const cubic of feature.cubics) {
				vertices.push(cubic.anchor0X);
				vertices.push(cubic.anchor0Y);
			}
		}

		if (Number.isNaN(centerX)) {
			centerX = this.calculateCenter(vertices).x;
		}
		if (Number.isNaN(centerY)) {
			centerY = this.calculateCenter(vertices).y;
		}

		return new RoundedPolygon(features, new Point(centerX, centerY));
	}

	static circle(numVertices = 8, radius = 1, centerX = 0, centerY = 0): RoundedPolygon {
		const theta = Math.PI / numVertices;
		const polygonRadius = radius / Math.cos(theta);
		return RoundedPolygon.fromNumVertices(
			numVertices,
			polygonRadius,
			centerX,
			centerY,
			new CornerRounding(radius)
		);
	}

	static rectangle(
		width: number,
		height: number,
		rounding = CornerRounding.Unrounded,
		perVertexRounding: CornerRounding[] | null = null,
		centerX = 0,
		centerY = 0
	): RoundedPolygon {
		const left = centerX - width / 2;
		const top = centerY - height / 2;
		const right = centerX + width / 2;
		const bottom = centerY + height / 2;

		return RoundedPolygon.fromVertices(
			[right, bottom, left, bottom, left, top, right, top],
			rounding,
			perVertexRounding,
			centerX,
			centerY
		);
	}

	static star(
		numVerticesPerRadius: number,
		radius = 1,
		innerRadius = 0.5,
		rounding = CornerRounding.Unrounded,
		innerRounding: CornerRounding | null = null,
		perVertexRounding: CornerRounding[] | null = null,
		centerX = 0,
		centerY = 0
	): RoundedPolygon {
		let pvRounding = perVertexRounding;
		if (pvRounding == null && innerRounding != null) {
			pvRounding = Array.from({ length: numVerticesPerRadius * 2 }).flatMap(() => [
				rounding,
				innerRounding
			]);
		}

		return RoundedPolygon.fromVertices(
			RoundedPolygon.starVerticesFromNumVerts(
				numVerticesPerRadius,
				radius,
				innerRadius,
				centerX,
				centerY
			),
			rounding,
			pvRounding,
			centerX,
			centerY
		);
	}

	static starVerticesFromNumVerts(
		numVerticesPerRadius: number,
		radius: number,
		innerRadius: number,
		centerX: number,
		centerY: number
	): number[] {
		const result: number[] = [];
		let arrayIndex = 0;
		for (let i = 0; i < numVerticesPerRadius; i++) {
			let vertex = Utils.radialToCartesian(radius, (Math.PI / numVerticesPerRadius) * 2 * i);
			result[arrayIndex++] = vertex.x + centerX;
			result[arrayIndex++] = vertex.y + centerY;
			vertex = Utils.radialToCartesian(innerRadius, (Math.PI / numVerticesPerRadius) * (2 * i + 1));
			result[arrayIndex++] = vertex.x + centerX;
			result[arrayIndex++] = vertex.y + centerY;
		}
		return result;
	}
}

export function roundedPolygonToSvgPath(polygon: RoundedPolygon, size = 100): string {
	if (!polygon.cubics || polygon.cubics.length === 0) return '';
	const c0 = polygon.cubics[0];
	let d = `M ${(c0.anchor0X * size).toFixed(3)} ${(c0.anchor0Y * size).toFixed(3)}`;
	for (const c of polygon.cubics) {
		d += ` C ${(c.control0X * size).toFixed(3)} ${(c.control0Y * size).toFixed(3)}, ${(c.control1X * size).toFixed(3)} ${(c.control1Y * size).toFixed(3)}, ${(c.anchor1X * size).toFixed(3)} ${(c.anchor1Y * size).toFixed(3)}`;
	}
	d += ' Z';
	return d;
}
