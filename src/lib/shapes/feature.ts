import { Cubic } from './cubic';

export abstract class Feature {
	cubics: Cubic[];

	constructor(cubics: Cubic[]) {
		this.cubics = cubics;
	}

	abstract transformed(f: (x: number, y: number) => { x: number; y: number }): Feature;

	buildIgnorableFeature(cubics: Cubic[]): Edge {
		return new Edge(cubics);
	}

	buildEdge(cubic: Cubic): Edge {
		return new Edge([cubic]);
	}

	buildConvexCorner(cubics: Cubic[]): Corner {
		return new Corner(cubics, true);
	}

	buildConcaveCorner(cubics: Cubic[]): Corner {
		return new Corner(cubics, false);
	}
}

export class Edge extends Feature {
	isIgnorableFeature = true;
	isEdge = true;
	isConvexCorner = false;
	isConcaveCorner = false;

	constructor(cubics: Cubic[]) {
		super(cubics);
	}

	transformed(f: (x: number, y: number) => { x: number; y: number }): Feature {
		return new Edge(this.cubics.map((c) => c.transformed(f)));
	}

	reversed(): Feature {
		return new Edge(this.cubics.map((c) => c.reverse()));
	}
}

export class Corner extends Feature {
	convex: boolean;
	isIgnorableFeature = false;
	isEdge = false;
	isConvexCorner: boolean;
	isConcaveCorner: boolean;

	constructor(cubics: Cubic[], convex: boolean) {
		super(cubics);
		this.convex = convex;
		this.isConvexCorner = convex;
		this.isConcaveCorner = !convex;
	}

	transformed(f: (x: number, y: number) => { x: number; y: number }): Feature {
		return new Corner(
			this.cubics.map((c) => c.transformed(f)),
			this.convex
		);
	}

	reversed(): Feature {
		return new Corner(
			this.cubics.map((c) => c.reverse()),
			!this.convex
		);
	}
}
