# Resonance ISA Specification v0.1

## Abjad Opcode Set

| Symbol | Mnemonic | 4-bit Opcode | Description |
|--------|----------|-------------|-------------|
| A | LOAD    | 0001 | Load a named register into the active trust vector |
| B | STORE   | 0010 | Write current trust value into a named register |
| C | COMPARE | 0011 | Evaluate a condition; result stored in `last_compare` |
| D | BRANCH  | 0100 | If `last_compare` false: raise entropy, decay trust |
| E | ENTER   | 0101 | Push a resonance scope (deed boundary) |
| F | FREEZE  | 0110 | Seal current state — WORM boundary, no further register writes |
| G | SIGNAL  | 0111 | Emit a coherence pulse; strengthens resonance field |
| H | HALT    | 1111 | End of program |

## Entropy Gate

Every instruction is gated by:

```
allowed(State) :- entropy(State, E), E < 0.21.
```

If `entropy >= 0.21` when any instruction executes, the VM raises `EntropyViolation` and halts. Execution is blocked. The Deed evaluates as `meta_block(degraded)`.

## Program Structure

A valid `.rasm` program must:
1. Begin with `E` (ENTER)
2. End with `H` (HALT)
3. Emit at least one `G` (SIGNAL) to produce a non-degraded Deed seal

## Unicode Mantra Oscillator (UMO)

After VM execution, the state vector (τ, ε, ρ) drives the UMO:

```
Φ(t) = sin(τ·t) × cos(ρ·t) × (1 − ε)
```

| Φ range | Glyph | Meaning |
|---------|-------|---------|
| > 0.9 | ☉ | Source coherent |
| > 0.7 | ◉ | Consensus active |
| > 0.5 | ◇ | Knowledge flowing |
| > 0.3 | ▣ | Constraint holding |
| > 0.1 | ▒ | Low entropy zone |
| ≤ 0.1 | ⛔ | Resonance failure |

## Deed Seal

On successful execution (halted + signaled + entropy < 0.21):

```
WORM SEAL = SHA-256(⟦Ω⟧|τ|ε|ρ|cycles|signals|program_hash)
```

The seal is the canonical proof that the program executed within sovereign constraint bounds.

## Full Pipeline

```
.rasm source
    ↓ abjad::parse_program()
Abjad token stream
    ↓ ir::lower() + ir::validate()
IR instruction list
    ↓ assembler::assemble()
ByteWord bytecode
    ↓ vm::Vm::run()
VM execution trace
    ↓ deed::Deed::evaluate()
Trust Deed (valid / degraded) + WORM seal
    ↓ umo::Umo::stream()
Unicode Mantra Oscillator waveform
```
