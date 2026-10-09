use std::fmt;

/// The result of lowering an integer expression in the current function.
pub enum Operand {
    Constant(i32),
    Register(String),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant(value) => write!(f, "{value}"),
            Self::Register(name) => f.write_str(name),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ControlFlow {
    Continues,
    Terminated,
}

/// Owns instruction output and SSA temporary names for one LLVM function.
#[derive(Default)]
pub struct FunctionContext {
    instructions: String,
    next_register: usize,
}

impl FunctionContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fresh_register(&mut self) -> String {
        let register = format!("%t{}", self.next_register);
        self.next_register += 1;
        register
    }

    pub fn emit(&mut self, instruction: &str) {
        self.instructions.push_str("  ");
        self.instructions.push_str(instruction);
        self.instructions.push('\n');
    }

    pub fn into_instructions(self) -> String {
        self.instructions
    }
}
