export class Point {
	x: number;
	y: number;

	constructor(x: number, y: number) {
		this.x = x;
		this.y = y;
	}

	copy(x = this.x, y = this.y): Point {
		return new Point(x, y);
	}

	getDistance(): number {
		return Math.sqrt(this.x * this.x + this.y * this.y);
	}

	getDistanceSquared(): number {
		return this.x * this.x + this.y * this.y;
	}

	dotProduct(other: Point): number {
		return this.x * other.x + this.y * other.y;
	}

	dotProductScalar(otherX: number, otherY: number): number {
		return this.x * otherX + this.y * otherY;
	}

	clockwise(other: Point): boolean {
		return this.x * other.y - this.y * other.x > 0;
	}

	getDirection(): Point {
		const d = this.getDistance();
		return d === 0 ? new Point(0, 0) : this.div(d);
	}

	negate(): Point {
		return new Point(-this.x, -this.y);
	}

	minus(other: Point): Point {
		return new Point(this.x - other.x, this.y - other.y);
	}

	plus(other: Point): Point {
		return new Point(this.x + other.x, this.y + other.y);
	}

	times(operand: number): Point {
		return new Point(this.x * operand, this.y * operand);
	}

	div(operand: number): Point {
		return new Point(this.x / operand, this.y / operand);
	}

	rem(operand: number): Point {
		return new Point(this.x % operand, this.y % operand);
	}

	static interpolate(start: Point, stop: Point, fraction: number): Point {
		return new Point(
			start.x + (stop.x - start.x) * fraction,
			start.y + (stop.y - start.y) * fraction
		);
	}

	transformed(f: (x: number, y: number) => { x: number; y: number }): Point {
		const result = f(this.x, this.y);
		return new Point(result.x, result.y);
	}

	rotate90(): Point {
		return new Point(-this.y, this.x);
	}
}
