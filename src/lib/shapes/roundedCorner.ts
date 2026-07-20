import { Point } from './point';
import { CornerRounding } from './cornerRounding';
import { DistanceEpsilon, directionVector } from './utils';
import { Cubic } from './cubic';

export class RoundedCorner {
	p0: Point;
	p1: Point;
	p2: Point;
	rounding: CornerRounding | null;
	center: Point;
	d1: Point;
	d2: Point;
	cornerRadius: number;
	smoothing: number;
	cosAngle: number;
	sinAngle: number;
	expectedRoundCut: number;

	constructor(p0: Point, p1: Point, p2: Point, rounding: CornerRounding | null = null) {
		this.p0 = p0;
		this.p1 = p1;
		this.p2 = p2;
		this.rounding = rounding;
		this.center = new Point(0, 0);

		const v01 = p0.minus(p1);
		const v21 = p2.minus(p1);
		const d01 = v01.getDistance();
		const d21 = v21.getDistance();

		if (d01 > 0 && d21 > 0) {
			this.d1 = v01.div(d01);
			this.d2 = v21.div(d21);
			this.cornerRadius = rounding?.radius ?? 0;
			this.smoothing = rounding?.smoothing ?? 0;
			this.cosAngle = this.d1.dotProduct(this.d2);
			this.sinAngle = Math.sqrt(Math.max(0, 1 - Math.pow(this.cosAngle, 2)));
			this.expectedRoundCut =
				this.sinAngle > 1e-3 ? (this.cornerRadius * (this.cosAngle + 1)) / this.sinAngle : 0;
		} else {
			this.d1 = new Point(0, 0);
			this.d2 = new Point(0, 0);
			this.cornerRadius = 0;
			this.smoothing = 0;
			this.cosAngle = 0;
			this.sinAngle = 0;
			this.expectedRoundCut = 0;
		}
	}

	get expectedCut(): number {
		return (1 + this.smoothing) * this.expectedRoundCut;
	}

	getCubics(allowedCut0: number, allowedCut1 = allowedCut0): Cubic[] {
		const allowedCut = Math.min(allowedCut0, allowedCut1);

		if (
			this.expectedRoundCut < DistanceEpsilon ||
			allowedCut < DistanceEpsilon ||
			this.cornerRadius < DistanceEpsilon
		) {
			this.center = this.p1;
			return [Cubic.straightLine(this.p1.x, this.p1.y, this.p1.x, this.p1.y)];
		}

		const actualRoundCut = Math.min(allowedCut, this.expectedRoundCut);
		const actualSmoothing0 = this.calculateActualSmoothingValue(allowedCut0);
		const actualSmoothing1 = this.calculateActualSmoothingValue(allowedCut1);

		const actualR = (this.cornerRadius * actualRoundCut) / this.expectedRoundCut;
		const centerDistance = Math.sqrt(Math.pow(actualR, 2) + Math.pow(actualRoundCut, 2));

		this.center = this.p1.plus(
			this.d1.plus(this.d2).div(2).getDirection().times(centerDistance)
		);

		const circleIntersection0 = this.p1.plus(this.d1.times(actualRoundCut));
		const circleIntersection2 = this.p1.plus(this.d2.times(actualRoundCut));

		const flanking0 = this.computeFlankingCurve(
			actualRoundCut,
			actualSmoothing0,
			this.p1,
			this.p0,
			circleIntersection0,
			circleIntersection2,
			this.center,
			actualR
		);

		const flanking2 = this.computeFlankingCurve(
			actualRoundCut,
			actualSmoothing1,
			this.p1,
			this.p2,
			circleIntersection2,
			circleIntersection0,
			this.center,
			actualR
		).reverse();

		return [
			flanking0,
			Cubic.circularArc(
				this.center.x,
				this.center.y,
				flanking0.anchor1X,
				flanking0.anchor1Y,
				flanking2.anchor0X,
				flanking2.anchor0Y
			),
			flanking2
		];
	}

	private calculateActualSmoothingValue(allowedCut: number): number {
		if (allowedCut > this.expectedCut) {
			return this.smoothing;
		} else if (allowedCut > this.expectedRoundCut) {
			return (
				(this.smoothing * (allowedCut - this.expectedRoundCut)) /
				(this.expectedCut - this.expectedRoundCut)
			);
		} else {
			return 0;
		}
	}

	private computeFlankingCurve(
		actualRoundCut: number,
		actualSmoothingValues: number,
		corner: Point,
		sideStart: Point,
		circleSegmentIntersection: Point,
		otherCircleSegmentIntersection: Point,
		circleCenter: Point,
		actualR: number
	): Cubic {
		const sideDirection = sideStart.minus(corner).getDirection();
		const curveStart = corner.plus(
			sideDirection.times(actualRoundCut * (1 + actualSmoothingValues))
		);

		const p = Point.interpolate(
			circleSegmentIntersection,
			circleSegmentIntersection.plus(otherCircleSegmentIntersection).div(2),
			actualSmoothingValues
		);

		const curveEnd = circleCenter.plus(
			directionVector(p.x - circleCenter.x, p.y - circleCenter.y).times(actualR)
		);

		const circleTangent = curveEnd.minus(circleCenter).rotate90();
		const anchorEnd =
			this.lineIntersection(sideStart, sideDirection, curveEnd, circleTangent) ??
			circleSegmentIntersection;

		const anchorStart = curveStart.plus(anchorEnd.times(2)).div(3);

		return Cubic.create(curveStart, anchorStart, anchorEnd, curveEnd);
	}

	private lineIntersection(p0: Point, d0: Point, p1: Point, d1: Point): Point | null {
		const rotatedD1 = d1.rotate90();
		const den = d0.dotProduct(rotatedD1);
		if (Math.abs(den) < DistanceEpsilon) return null;

		const num = p1.minus(p0).dotProduct(rotatedD1);
		if (Math.abs(den) < DistanceEpsilon * Math.abs(num)) return null;

		const k = num / den;
		return p0.plus(d0.times(k));
	}
}
