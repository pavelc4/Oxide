// Native Web Audio API sound generator (Ponytail: zero dependencies, zero audio files)
export function playChime(type: 'connect' | 'disconnect' | 'success' | 'error' = 'connect'): void {
	if (typeof window === 'undefined') return;
	const soundEnabled = localStorage.getItem('oxide:soundEnabled') !== 'false';
	if (!soundEnabled) return;

	try {
		const AudioCtx = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
		if (!AudioCtx) return;
		const ctx = new AudioCtx();
		const osc = ctx.createOscillator();
		const gain = ctx.createGain();
		osc.connect(gain);
		gain.connect(ctx.destination);

		const now = ctx.currentTime;
		if (type === 'connect' || type === 'success') {
			osc.type = 'sine';
			osc.frequency.setValueAtTime(523.25, now); // C5
			osc.frequency.exponentialRampToValueAtTime(659.25, now + 0.08); // E5
			osc.frequency.exponentialRampToValueAtTime(783.99, now + 0.16); // G5
			gain.gain.setValueAtTime(0.12, now);
			gain.gain.exponentialRampToValueAtTime(0.001, now + 0.3);
			osc.start(now);
			osc.stop(now + 0.3);
		} else if (type === 'disconnect') {
			osc.type = 'sine';
			osc.frequency.setValueAtTime(659.25, now); // E5
			osc.frequency.exponentialRampToValueAtTime(440.00, now + 0.15); // A4
			gain.gain.setValueAtTime(0.12, now);
			gain.gain.exponentialRampToValueAtTime(0.001, now + 0.3);
			osc.start(now);
			osc.stop(now + 0.3);
		} else {
			osc.type = 'triangle';
			osc.frequency.setValueAtTime(320, now);
			osc.frequency.linearRampToValueAtTime(180, now + 0.15);
			gain.gain.setValueAtTime(0.15, now);
			gain.gain.exponentialRampToValueAtTime(0.001, now + 0.25);
			osc.start(now);
			osc.stop(now + 0.25);
		}
	} catch {
		// AudioContext blocked by browser policy until user gesture
	}
}
