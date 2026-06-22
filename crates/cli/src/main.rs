use std::fs;
use sha2::{Sha256, Digest};
use clap::{Parser, Subcommand};
use abjad::parse_program;
use ir::{lower, validate};
use assembler::{assemble, disassemble};
use vm::Vm;
use deed::Deed;
use umo::Umo;

#[derive(Parser)]
#[command(
    name  = "rasm",
    about = "⟦ Ω ⟧ Resonance Assembly VM — Abjad → IR → Bytecode → VM → UMO",
    version,
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run a .rasm program through the full pipeline
    Run {
        /// Path to the .rasm source file
        file: String,
        /// Show Unicode Mantra Oscillator waveform
        #[arg(long, default_value_t = true)]
        umo: bool,
    },
    /// Compile a .rasm file and print the binary bytecode
    Compile {
        file: String,
    },
    /// Validate a .rasm program (IR pass only)
    Check {
        file: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Run { file, umo } => run(&file, umo),
        Cmd::Compile { file }  => compile(&file),
        Cmd::Check { file }    => check(&file),
    }
}

fn load(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| { eprintln!("error: {e}"); std::process::exit(1) })
}

fn program_hash(src: &str) -> String {
    hex::encode(Sha256::digest(src.as_bytes()))
}

fn run(path: &str, show_umo: bool) {
    let src = load(path);
    println!("⟦ Ω ⟧ RESONANCE VM — {path}");
    println!("{}", "─".repeat(50));

    // Abjad → IR
    let tokens = parse_program(&src);
    let ir = lower(tokens);
    validate(&ir).unwrap_or_else(|e| { eprintln!("IR validation failed: {e}"); std::process::exit(1) });

    // IR → bytecode
    let bytecode = assemble(&ir);
    println!("bytecode : {}", disassemble(&bytecode));
    println!("instructions: {}", bytecode.len());
    println!("{}", "─".repeat(50));

    // Execute VM
    let mut vm = Vm::new();
    for (i, result) in vm.run(&bytecode) {
        let op = &ir[i];
        println!("  [{i:02}] {:?} {:5} → {:?}", op.op, op.operand.as_deref().unwrap_or(""), result);
        if result == vm::StepResult::EntropyViolation {
            println!("  ⛔ entropy={:.4} exceeds threshold 0.21 — execution blocked", vm.entropy);
            break;
        }
    }

    // Deed evaluation
    let hash = program_hash(&src);
    let deed = Deed::evaluate(&vm, &hash);
    println!("{}", "─".repeat(50));
    println!("{}", deed.status_glyph());
    println!("  τ trust     : {:.4}", deed.trust);
    println!("  ε entropy   : {:.4}", deed.entropy);
    println!("  ρ resonance : {:.4}", deed.resonance);
    println!("  cycles      : {}", deed.cycles);
    println!("  signals     : {}", deed.signals);
    if let Some(seal) = &deed.seal {
        println!("  WORM seal   : {seal}");
    }

    // UMO waveform
    if show_umo {
        println!("{}", "─".repeat(50));
        println!("⍟ UNICODE MANTRA OSCILLATOR  Φ(t) = sin(τ·t) × cos(ρ·t) × (1−ε)");
        let umo = Umo::new(deed.trust, deed.entropy, deed.resonance);
        for line in umo.stream(6, 12, 0.4) {
            println!("  {line}");
        }
        println!("  {}", umo.sealed_output());
    }
}

fn compile(path: &str) {
    let src = load(path);
    let tokens = parse_program(&src);
    let ir = lower(tokens);
    validate(&ir).unwrap_or_else(|e| { eprintln!("{e}"); std::process::exit(1) });
    let bytecode = assemble(&ir);
    println!("{}", disassemble(&bytecode));
}

fn check(path: &str) {
    let src = load(path);
    let tokens = parse_program(&src);
    let ir = lower(tokens);
    match validate(&ir) {
        Ok(())   => println!("✓ valid — {} instructions", ir.len()),
        Err(msg) => { eprintln!("✗ {msg}"); std::process::exit(1) }
    }
}
