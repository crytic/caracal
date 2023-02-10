use cairo_lang_sierra::{
    ids::VarId,
    program::{GenStatement, Statement as SierraStatement},
};

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

    pub fn variables_read(&self) -> &Vec<VarId> {
        match &self.statement {
            GenStatement::Invocation(inv) => &inv.args,
            GenStatement::Return(vars) => vars,
        }
    }

    pub fn variables_written(&self) -> Vec<VarId> {
        match &self.statement {
            GenStatement::Invocation(inv) => {
                let mut vars_written = Vec::new();
                for branch in inv.branches.iter() {
                    vars_written.extend(branch.results.clone());
                }
                vars_written
            }
            GenStatement::Return(_) => {
                vec![]
            }
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.pc, self.statement)
    }
}
