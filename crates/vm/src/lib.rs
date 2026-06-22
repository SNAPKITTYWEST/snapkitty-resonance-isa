use std::collections::HashMap;
use assembler::ByteWord;

pub const ENTROPY_THRESHOLD: f64 = 0.21;

/// VM execution result after a single step.
#[derive(Debug, Clone, PartialEq)]
pub enum StepResult {
    Continue,
    SignalEmitted,
    Halted,
    EntropyViolation,
}

/// The Resonance VM — executes compiled bytecode with entropy gating.
pub struct Vm {
    pub registers:    HashMap<String, f64>,
    pub entropy:      f64,
    pub trust:        f64,
    pub resonance:    f64,
    pub halted:       bool,
    pub scope_depth:  u32,
    pub signal_count: u32,
    pub frozen:       bool,
    pub last_compare: bool,
    pub cycle:        usize,
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

impl Vm {
    pub fn new() -> Self {
        let mut registers = HashMap::new();
        registers.insert("trust_vector".into(),    1.0);
        registers.insert("entropy_register".into(), 0.05);
        registers.insert("resonance_field".into(),  1.0);
        Self {
            registers,
            entropy:      0.05,
            trust:        1.0,
            resonance:    1.0,
            halted:       false,
            scope_depth:  0,
            signal_count: 0,
            frozen:       false,
            last_compare: false,
            cycle:        0,
        }
    }

    /// Gate check — entropy must be below threshold to execute.
    pub fn allowed(&self) -> bool {
        self.entropy < ENTROPY_THRESHOLD
    }

    /// Execute a single bytecode word.
    pub fn step(&mut self, word: &ByteWord) -> StepResult {
        if self.halted { return StepResult::Halted; }
        if !self.allowed() { return StepResult::EntropyViolation; }

        self.cycle += 1;
        match word.opcode {
            0b0001 => {
                // LOAD — bring a register into the active trust vector
                self.trust = self.registers.get("trust_vector").copied().unwrap_or(1.0);
                StepResult::Continue
            }
            0b0010 => {
                // STORE — write current trust back to register
                self.registers.insert("trust_vector".into(), self.trust);
                StepResult::Continue
            }
            0b0011 => {
                // COMPARE — evaluate entropy < threshold
                self.last_compare = self.entropy < ENTROPY_THRESHOLD;
                StepResult::Continue
            }
            0b0100 => {
                // BRANCH — if last compare failed, raise entropy (signal degradation)
                if !self.last_compare {
                    self.entropy += 0.1;
                    self.trust   *= 0.9;
                }
                StepResult::Continue
            }
            0b0101 => {
                // ENTER — push scope
                self.scope_depth += 1;
                StepResult::Continue
            }
            0b0110 => {
                // FREEZE — seal state (mark frozen, no further register writes)
                self.frozen = true;
                StepResult::Continue
            }
            0b0111 => {
                // SIGNAL — emit coherence pulse; resonance strengthens with each signal
                self.signal_count += 1;
                self.resonance = (self.resonance + 0.05).min(1.0);
                StepResult::SignalEmitted
            }
            0b1111 => {
                // HALT
                self.halted = true;
                StepResult::Halted
            }
            _ => StepResult::Continue,
        }
    }

    /// Run a full bytecode program to completion. Returns a trace of each step.
    pub fn run(&mut self, bytecode: &[ByteWord]) -> Vec<(usize, StepResult)> {
        bytecode
            .iter()
            .enumerate()
            .map(|(i, word)| (i, self.step(word)))
            .collect()
    }

    /// Current state vector: (trust τ, entropy ε, resonance ρ).
    pub fn state_vector(&self) -> (f64, f64, f64) {
        (self.trust, self.entropy, self.resonance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assembler::{ByteWord};

    fn word(opcode: u8) -> ByteWord { ByteWord { opcode, operand: 0 } }

    #[test]
    fn test_entropy_gate() {
        let mut vm = Vm::new();
        vm.entropy = 0.99;
        assert!(!vm.allowed());
    }

    #[test]
    fn test_halt() {
        let mut vm = Vm::new();
        let result = vm.step(&word(0b1111));
        assert_eq!(result, StepResult::Halted);
        assert!(vm.halted);
    }

    #[test]
    fn test_signal_strengthens_resonance() {
        let mut vm = Vm::new();
        vm.step(&word(0b0111));
        assert!(vm.resonance > 1.0 - 1e-9);
        assert_eq!(vm.signal_count, 1);
    }
}
