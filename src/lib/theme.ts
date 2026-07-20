export function hexToHsl(hex: string): { h: number; s: number; l: number } {
	const clean = hex.replace('#', '');
	const r = parseInt(clean.substring(0, 2) || 'ff', 16) / 255;
	const g = parseInt(clean.substring(2, 4) || 'b7', 16) / 255;
	const b = parseInt(clean.substring(4, 6) || '84', 16) / 255;

	const max = Math.max(r, g, b);
	const min = Math.min(r, g, b);
	let h = 0, s = 0;
	const l = (max + min) / 2;

	if (max !== min) {
		const d = max - min;
		s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
		switch (max) {
			case r: h = (g - b) / d + (g < b ? 6 : 0); break;
			case g: h = (b - r) / d + 2; break;
			case b: h = (r - g) / d + 4; break;
		}
		h /= 6;
	}
	return { h: Math.round(h * 360), s: Math.round(s * 100), l: Math.round(l * 100) };
}

export function applyThemeColorToDocument(colorHex: string) {
	if (typeof document === 'undefined') return;
	const root = document.documentElement;

	const { h, s } = hexToHsl(colorHex);
	const bgHue = h;
	const bgSat = Math.min(s, 22);

	const oledMode = typeof localStorage !== 'undefined' && localStorage.getItem('oxide:oledMode') === 'true';
	const isDark = root.getAttribute('data-theme') !== 'light';

	if (oledMode && isDark) {
		root.style.setProperty('--primary', colorHex);
		root.style.setProperty('--primaryContainer', `hsl(${bgHue}, ${bgSat + 15}%, 22%)`);
		root.style.setProperty('--onPrimaryContainer', `hsl(${bgHue}, 90%, 85%)`);

		root.style.setProperty('--background', '#000000');
		root.style.setProperty('--surface', '#000000');
		root.style.setProperty('--surfaceContainerLowest', '#000000');
		root.style.setProperty('--surfaceContainerLow', '#0a0a0a');
		root.style.setProperty('--surfaceContainer', '#121212');
		root.style.setProperty('--surfaceContainerHigh', '#1a1a1a');
		root.style.setProperty('--surfaceContainerHighest', '#242424');
		root.style.setProperty('--onSurface', '#efe0d9');
		root.style.setProperty('--onBackground', '#efe0d9');
		root.style.setProperty('--onSurfaceVariant', '#d7c2b8');
		return;
	}

	if (isDark) {
		root.style.setProperty('--primary', colorHex);
		root.style.setProperty('--primaryContainer', `hsl(${bgHue}, ${bgSat + 15}%, 22%)`);
		root.style.setProperty('--onPrimaryContainer', `hsl(${bgHue}, 90%, 85%)`);

		root.style.setProperty('--background', `hsl(${bgHue}, ${bgSat}%, 6%)`);
		root.style.setProperty('--surface', `hsl(${bgHue}, ${bgSat}%, 6%)`);
		root.style.setProperty('--surfaceContainerLowest', `hsl(${bgHue}, ${bgSat}%, 6%)`);
		root.style.setProperty('--surfaceContainerLow', `hsl(${bgHue}, ${bgSat}%, 9%)`);
		root.style.setProperty('--surfaceContainer', `hsl(${bgHue}, ${bgSat + 5}%, 12%)`);
		root.style.setProperty('--surfaceContainerHigh', `hsl(${bgHue}, ${bgSat + 8}%, 16%)`);
		root.style.setProperty('--surfaceContainerHighest', `hsl(${bgHue}, ${bgSat + 10}%, 20%)`);
		root.style.setProperty('--onSurface', `hsl(${bgHue}, 15%, 92%)`);
		root.style.setProperty('--onBackground', `hsl(${bgHue}, 15%, 92%)`);
		root.style.setProperty('--onSurfaceVariant', `hsl(${bgHue}, 15%, 78%)`);
	} else {
		root.style.setProperty('--primary', colorHex);
		root.style.setProperty('--primaryContainer', `hsl(${bgHue}, 80%, 90%)`);
		root.style.setProperty('--onPrimaryContainer', `hsl(${bgHue}, 90%, 15%)`);

		root.style.setProperty('--background', `hsl(${bgHue}, 30%, 98%)`);
		root.style.setProperty('--surface', `hsl(${bgHue}, 30%, 98%)`);
		root.style.setProperty('--surfaceContainerLowest', '#ffffff');
		root.style.setProperty('--surfaceContainerLow', `hsl(${bgHue}, 25%, 95%)`);
		root.style.setProperty('--surfaceContainer', `hsl(${bgHue}, 25%, 92%)`);
		root.style.setProperty('--surfaceContainerHigh', `hsl(${bgHue}, 25%, 89%)`);
		root.style.setProperty('--surfaceContainerHighest', `hsl(${bgHue}, 25%, 85%)`);
		root.style.setProperty('--onSurface', `hsl(${bgHue}, 20%, 12%)`);
		root.style.setProperty('--onBackground', `hsl(${bgHue}, 20%, 12%)`);
		root.style.setProperty('--onSurfaceVariant', `hsl(${bgHue}, 20%, 30%)`);
	}
}

export function initGlobalTheme() {
	if (typeof window === 'undefined' || typeof localStorage === 'undefined') return;

	const root = document.documentElement;

	// 1. Restore Theme Mode
	const mode = localStorage.getItem('oxide:themeMode') || 'dark';
	root.setAttribute('data-theme', mode);

	// 2. Restore Corner Radius
	const corner = localStorage.getItem('oxide:cornerRadius') || 'expressive';
	root.setAttribute('data-corner', corner);

	// 3. Restore Theme Seed Color
	const seed = localStorage.getItem('oxide:seedColor') || '#ffb784';
	applyThemeColorToDocument(seed);
}
