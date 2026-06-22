/// Unicode Mantra Oscillator — maps VM state to a symbolic waveform.
///
/// Φ(t) = sin(τ·t) × cos(ρ·t) × (1 − ε)
///
/// Trust drives amplitude stability.
/// Resonance drives frequency structure.
/// Entropy damps and eventually collapses the signal.

/// Map an oscillator output value in [−1, 1] to a Unicode glyph.
pub fn glyph(value: f64) -> char {
    let v = value.abs();
    match v {
        x if x > 0.9 => '☉',
        x if x > 0.7 => '◉',
        x if x > 0.5 => '◇',
        x if x > 0.3 => '▣',
        x if x > 0.1 => '▒',
        _             => '⛔',
    }
}

/// The oscillator state: trust τ, entropy ε, resonance ρ.
pub struct Umo {
    pub trust:     f64,
    pub entropy:   f64,
    pub resonance: f64,
}

impl Umo {
    pub fn new(trust: f64, entropy: f64, resonance: f64) -> Self {
        Self { trust, entropy, resonance }
    }

    /// Compute the oscillator signal at time t.
    /// Φ(t) = sin(τ·t) × cos(ρ·t) × (1 − ε)
    pub fn signal(&self, t: f64) -> f64 {
        (self.trust * t).sin() * (self.resonance * t).cos() * (1.0 - self.entropy)
    }

    /// Render a waveform of `width` samples starting at t=0 with step dt.
    pub fn waveform(&self, width: usize, dt: f64) -> String {
        (0..width)
            .map(|i| glyph(self.signal(i as f64 * dt)))
            .collect()
    }

    /// Render a multi-line oscillation stream over `steps` time ticks.
    pub fn stream(&self, steps: usize, width: usize, dt: f64) -> Vec<String> {
        (0..steps)
            .map(|t| {
                let t0 = t as f64;
                let line: String = (0..width)
                    .map(|i| glyph(self.signal(t0 + i as f64 * dt)))
                    .collect();
                format!("t={:<2} : {}", t, line)
            })
            .collect()
    }

    /// Returns true if the field is coherent (signal amplitude > noise floor).
    pub fn coherent(&self) -> bool {
        self.entropy < 0.21 && self.trust > 0.5
    }

    /// Deed-sealed output glyphs.
    pub fn sealed_output(&self) -> String {
        if self.coherent() {
            "☉◉◇◉☉◉◇◉☉ → SYSTEM COHERENT → DEED SEALED".into()
        } else {
            "▒░⛔⚠️⛔░▒ → RESONANCE FAILURE → EXECUTION BLOCKED".into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glyph_high() {
        assert_eq!(glyph(0.95), '☉');
    }

    #[test]
    fn test_glyph_low() {
        assert_eq!(glyph(0.05), '⛔');
    }

    #[test]
    fn test_coherent() {
        let umo = Umo::new(1.0, 0.05, 1.0);
        assert!(umo.coherent());
    }

    #[test]
    fn test_degraded() {
        let umo = Umo::new(1.0, 0.99, 1.0);
        assert!(!umo.coherent());
    }

    #[test]
    fn test_waveform_length() {
        let umo = Umo::new(1.0, 0.05, 1.0);
        assert_eq!(umo.waveform(10, 0.3).chars().count(), 10);
    }
}
