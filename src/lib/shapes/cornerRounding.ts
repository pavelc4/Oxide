export class CornerRounding {
	radius: number;
	smoothing: number;

	constructor(radius = 0, smoothing = 0) {
		this.radius = radius;
		this.smoothing = smoothing;
	}

	static Unrounded = new CornerRounding(0, 0);
}
