use abjad::AbjadOp;
use ir::Instruction;

/// Encode an AbjadOp into its 4-bit binary opcode (stored in a u8).
pub fn encode(op: &AbjadOp) -> u8 {
    match op {
        AbjadOp::Load    => 0b0001,
        AbjadOp::Store   => 0b0010,
        AbjadOp::Compare => 0b0011,
        AbjadOp::Branch  => 0b0100,
        AbjadOp::Enter   => 0b0101,
        AbjadOp::Freeze  => 0b0110,
        AbjadOp::Signal  => 0b0111,
        AbjadOp::Halt    => 0b1111,
    }
}

/// A compiled bytecode word — opcode + operand hash (0 if none).
#[derive(Debug, Clone)]
pub struct ByteWord {
    pub opcode:  u8,
    pub operand: u8, // simplified operand hash for demo VM
}

/// Assemble an IR program into a binary bytecode stream.
pub fn assemble(program: &[Instruction]) -> Vec<ByteWord> {
    program
        .iter()
        .map(|instr| ByteWord {
            opcode:  encode(&instr.op),
            operand: instr.operand
                .as_deref()
                .map(|s| s.bytes().fold(0u8, |acc, b| acc.wrapping_add(b)))
                .unwrap_or(0),
        })
        .collect()
}

/// Display the bytecode stream as a space-separated binary string.
pub fn disassemble(bytecode: &[ByteWord]) -> String {
    bytecode
        .iter()
        .map(|w| format!("{:04b}", w.opcode))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_halt() {
        assert_eq!(encode(&AbjadOp::Halt), 0b1111);
    }

    #[test]
    fn test_encode_enter() {
        assert_eq!(encode(&AbjadOp::Enter), 0b0101);
    }
}
