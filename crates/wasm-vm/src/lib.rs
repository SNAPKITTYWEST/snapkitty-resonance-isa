use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

use abjad::parse_program;
use ir::{lower, validate};
use assembler::{assemble, disassemble};
use vm::Vm;
use deed::Deed;
use umo::Umo;

#[derive(Serialize, Deserialize)]
pub struct StepTrace {
    pub index:   usize,
    pub op:      String,
    pub operand: Option<String>,
    pub result:  String,
}

#[derive(Serialize, Deserialize)]
pub struct RunResult {
    pub ok:        bool,
    pub error:     Option<String>,
    pub bytecode:  String,
    pub trace:     Vec<StepTrace>,
    pub valid:     bool,
    pub trust:     f64,
    pub entropy:   f64,
    pub resonance: f64,
    pub cycles:    usize,
    pub signals:   u32,
    pub seal:      Option<String>,
    pub status:    String,
    pub umo_lines: Vec<String>,
    pub umo_sealed: String,
}

/// Run a .rasm source string through the full pipeline.
/// Returns a JSON-serialisable RunResult.
#[wasm_bindgen]
pub fn run_program(src: &str) -> JsValue {
    let result = execute(src);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// Validate only — returns "ok" or an error string.
#[wasm_bindgen]
pub fn check_program(src: &str) -> String {
    let tokens = parse_program(src);
    let ir = lower(tokens);
    match validate(&ir) {
        Ok(())   => format!("✓ valid — {} instructions", ir.len()),
        Err(msg) => format!("✗ {msg}"),
    }
}

/// Disassemble a .rasm source to binary bytecode string.
#[wasm_bindgen]
pub fn compile_program(src: &str) -> String {
    let tokens = parse_program(src);
    let ir = lower(tokens);
    let bytecode = assemble(&ir);
    disassemble(&bytecode)
}

fn execute(src: &str) -> RunResult {
    // Parse
    let tokens = parse_program(src);
    let ir = lower(tokens);

    if let Err(msg) = validate(&ir) {
        return RunResult {
            ok: false, error: Some(msg),
            bytecode: String::new(), trace: vec![],
            valid: false, trust: 0.0, entropy: 1.0, resonance: 0.0,
            cycles: 0, signals: 0, seal: None,
            status: "✗ IR validation failed".into(),
            umo_lines: vec![], umo_sealed: String::new(),
        };
    }

    let bytecode = assemble(&ir);
    let bytecode_str = disassemble(&bytecode);

    // Execute
    let mut vm_inst = Vm::new();
    let mut trace = Vec::new();

    for (i, word) in bytecode.iter().enumerate() {
        let step = vm_inst.step(word);
        let step_str = format!("{:?}", step);
        let is_violation = step_str.contains("Violation");
        trace.push(StepTrace {
            index:   i,
            op:      ir[i].op.mnemonic().to_string(),
            operand: ir[i].operand.clone(),
            result:  step_str,
        });
        if is_violation || vm_inst.halted { break; }
    }

    // Deed
    let prog_hash = hex::encode(Sha256::digest(src.as_bytes()));
    let deed = Deed::evaluate(&vm_inst, &prog_hash);

    // UMO
    let umo = Umo::new(deed.trust, deed.entropy, deed.resonance);
    let umo_lines = umo.stream(6, 12, 0.4);
    let umo_sealed = umo.sealed_output();

    RunResult {
        ok:        true,
        error:     None,
        bytecode:  bytecode_str,
        trace,
        valid:     deed.valid,
        trust:     deed.trust,
        entropy:   deed.entropy,
        resonance: deed.resonance,
        cycles:    deed.cycles,
        signals:   deed.signals,
        status:    deed.status_glyph().to_string(),
        seal:      deed.seal,
        umo_lines,
        umo_sealed,
    }
}
