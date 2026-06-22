use abjad::AbjadOp;

/// Intermediate representation instruction — op + optional operand label.
#[derive(Debug, Clone)]
pub struct Instruction {
    pub op:      AbjadOp,
    pub operand: Option<String>,
    pub index:   usize,
}

/// Lower a raw abjad token stream into an IR program.
pub fn lower(tokens: Vec<(AbjadOp, Option<String>)>) -> Vec<Instruction> {
    tokens
        .into_iter()
        .enumerate()
        .map(|(i, (op, operand))| Instruction { op, operand, index: i })
        .collect()
}

/// Validate the IR — must start with Enter and end with Halt.
pub fn validate(program: &[Instruction]) -> Result<(), String> {
    if program.is_empty() {
        return Err("empty program".into());
    }
    if program[0].op != AbjadOp::Enter {
        return Err(format!("program must begin with ENTER, got {:?}", program[0].op));
    }
    if program.last().map(|i| &i.op) != Some(&AbjadOp::Halt) {
        return Err("program must end with HALT".into());
    }
    Ok(())
}
