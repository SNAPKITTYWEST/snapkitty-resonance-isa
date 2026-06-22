/// Abjad symbolic opcode set — A through H map to the 8 resonance instructions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbjadOp {
    Load,    // A — load a register from state
    Store,   // B — write register into state
    Compare, // C — evaluate a condition
    Branch,  // D — conditional jump on last compare
    Enter,   // E — enter resonance scope / deed boundary
    Freeze,  // F — seal current state (WORM boundary)
    Signal,  // G — emit coherence pulse to UMO
    Halt,    // H — end of program
}

impl AbjadOp {
    pub fn mnemonic(&self) -> &'static str {
        match self {
            AbjadOp::Load    => "LOAD",
            AbjadOp::Store   => "STORE",
            AbjadOp::Compare => "COMPARE",
            AbjadOp::Branch  => "BRANCH",
            AbjadOp::Enter   => "ENTER",
            AbjadOp::Freeze  => "FREEZE",
            AbjadOp::Signal  => "SIGNAL",
            AbjadOp::Halt    => "HALT",
        }
    }
}

/// Parse a single token from a .rasm source file.
pub fn parse_token(token: &str) -> Option<AbjadOp> {
    match token.trim().to_uppercase().as_str() {
        "A" | "LOAD"    => Some(AbjadOp::Load),
        "B" | "STORE"   => Some(AbjadOp::Store),
        "C" | "COMPARE" => Some(AbjadOp::Compare),
        "D" | "BRANCH"  => Some(AbjadOp::Branch),
        "E" | "ENTER"   => Some(AbjadOp::Enter),
        "F" | "FREEZE"  => Some(AbjadOp::Freeze),
        "G" | "SIGNAL"  => Some(AbjadOp::Signal),
        "H" | "HALT"    => Some(AbjadOp::Halt),
        _ => None,
    }
}

/// Parse a full .rasm source string into an instruction stream.
/// Lines starting with ';' are comments. Empty lines are skipped.
pub fn parse_program(src: &str) -> Vec<(AbjadOp, Option<String>)> {
    src.lines()
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with(';'))
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let token = parts.next()?;
            let operand = parts.next().map(|s| s.to_string());
            let op = parse_token(token)?;
            Some((op, operand))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tokens() {
        assert_eq!(parse_token("E"), Some(AbjadOp::Enter));
        assert_eq!(parse_token("H"), Some(AbjadOp::Halt));
        assert_eq!(parse_token("LOAD"), Some(AbjadOp::Load));
        assert_eq!(parse_token("X"), None);
    }

    #[test]
    fn test_parse_program() {
        let src = "; entry\nE field_core\nA trust\nH";
        let prog = parse_program(src);
        assert_eq!(prog.len(), 3);
        assert_eq!(prog[0].0, AbjadOp::Enter);
        assert_eq!(prog[0].1, Some("field_core".to_string()));
        assert_eq!(prog[2].0, AbjadOp::Halt);
    }
}
