export class Offset {
	x: number;
	y: number;

	constructor(x: number, y: number) {
		this.x = x;
		this.y = y;
	}

	copy(x = this.x, y = this.y): Offset {
		return new Offset(x, y);
	}

	getDistance(): number {
		return Math.sqrt(this.x * this.x + this.y * this.y);
	}

	getDistanceSquared(): number {
		return this.x * this.x + this.y * this.y;
	}

	isValid(): boolean {
		return Number.isFinite(this.x) && Number.isFinite(this.y);
	}

	get isFinite(): boolean {
		return Number.isFinite(this.x) && Number.isFinite(this.y);
	}

	get isSpecified(): boolean {
		return !this.isUnspecified;
	}

	get isUnspecified(): boolean {
		return Object.is(this.x, NaN) && Object.is(this.y, NaN);
	}

	negate(): Offset {
		return new Offset(-this.x, -this.y);
	}

	minus(other: Offset): Offset {
		return new Offset(this.x - other.x, this.y - other.y);
	}

	plus(other: Offset): Offset {
		return new Offset(this.x + other.x, this.y + other.y);
	}

	times(operand: number): Offset {
		return new Offset(this.x * operand, this.y * operand);
	}

	div(operand: number): Offset {
		return new Offset(this.x / operand, this.y / operand);
	}

	rem(operand: number): Offset {
		return new Offset(this.x % operand, this.y % operand);
	}

	toString(): string {
		if (this.isSpecified) {
			return `Offset(${this.x.toFixed(1)}, ${this.y.toFixed(1)})`;
		}
		return 'Offset.Unspecified';
	}

	static lerp(start: Offset, stop: Offset, fraction: number): Offset {
		return new Offset(
			start.x + (stop.x - start.x) * fraction,
			start.y + (stop.y - start.y) * fraction
		);
	}

	takeOrElse(block: () => Offset): Offset {
		return this.isSpecified ? this : block();
	}

	angleDegrees(): number {
		return (Math.atan2(this.y, this.x) * 180) / Math.PI;
	}

	rotateDegrees(angle: number, center: Offset = Offset.Zero): Offset {
		const a = (angle * Math.PI) / 180;
		const off = this.minus(center);
		const cosA = Math.cos(a);
		const sinA = Math.sin(a);
		const newX = off.x * cosA - off.y * sinA;
		const newY = off.x * sinA + off.y * cosA;
		return new Offset(newX, newY).plus(center);
	}

	static Zero = new Offset(0, 0);
	static Infinite = new Offset(Infinity, Infinity);
	static Unspecified = new Offset(NaN, NaN);
}
