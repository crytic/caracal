use cairo_lang_sierra::program::Statement as SierraStatement;

#[derive(Debug, Clone)]
pub struct Instruction {
    pc: usize,
    statement: SierraStatement,
}

impl Instruction {
    pub fn new(pc: usize, statement: SierraStatement) -> Self {
        Instruction { pc, statement }
    }

    pub fn get_pc(&self) -> usize {
        self.pc
    }

    pub fn get_statement(&self) -> &SierraStatement {
        &self.statement
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.pc, self.statement)
    }
}
