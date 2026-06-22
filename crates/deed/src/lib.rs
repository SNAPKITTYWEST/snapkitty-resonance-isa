use sha2::{Sha256, Digest};
use vm::Vm;

pub const ENTROPY_THRESHOLD: f64 = vm::ENTROPY_THRESHOLD;

/// A Resonance Block Trust Deed — constraint-checked execution envelope.
#[derive(Debug, Clone)]
pub struct Deed {
    pub valid:     bool,
    pub seal:      Option<String>,
    pub trust:     f64,
    pub entropy:   f64,
    pub resonance: f64,
    pub cycles:    usize,
    pub signals:   u32,
}

impl Deed {
    /// Evaluate a deed from a completed VM state.
    pub fn evaluate(vm: &Vm, program_hash: &str) -> Self {
        let (trust, entropy, resonance) = vm.state_vector();
        let valid = entropy < ENTROPY_THRESHOLD && vm.halted && vm.signal_count > 0;

        let seal = if valid {
            let payload = format!(
                "⟦Ω⟧|τ={:.6}|ε={:.6}|ρ={:.6}|cycles={}|signals={}|prog={}",
                trust, entropy, resonance, vm.cycle, vm.signal_count, program_hash
            );
            let hash = Sha256::digest(payload.as_bytes());
            Some(hex::encode(hash))
        } else {
            None
        };

        Self {
            valid,
            seal,
            trust,
            entropy,
            resonance,
            cycles: vm.cycle,
            signals: vm.signal_count,
        }
    }

    pub fn status_glyph(&self) -> &'static str {
        if self.valid { "✅ meta_block(valid)" } else { "⛔ meta_block(degraded)" }
    }
}
