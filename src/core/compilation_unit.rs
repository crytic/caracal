use super::function::Function;

use cairo_lang_sierra::program::{
    Function as SierraFunction, Program, Statement as SierraStatement,
};

#[derive(Debug)]
pub struct CompilationUnit<'a> {
    sierra_program: &'a Program,
    functions: Vec<Function<'a>>,
}

impl<'a> CompilationUnit<'a> {
    pub fn new(sierra_program: &'a Program) -> Self {
        CompilationUnit {
            sierra_program,
            functions: Vec::new(),
        }
    }

    /// Returns the functions that are not part of the core library
    pub fn functions(&self) -> impl Iterator<Item = &Function> {
        self.functions.iter()
    }

    /// Returns the functions that are not part of the core library
    pub fn functions_without_core(&self) -> impl Iterator<Item = &Function> {
        self.functions.iter().filter(|f| !f.is_core())
    }

    fn append_function(&mut self, data: &'a SierraFunction, statements: Vec<SierraStatement>) {
        self.functions.push(Function::new(data, statements));
    }

    pub fn analyze(&mut self) {
        // Add the functions in the sierra program
        let mut funcs_chunks = self.sierra_program.funcs.windows(2).peekable();

        // There is only 1 function
        if funcs_chunks.peek().is_none() {
            let function = &self.sierra_program.funcs[0];

            self.append_function(
                function,
                self.sierra_program.statements
                    [function.entry_point.0..self.sierra_program.statements.len()]
                    .to_vec(),
            );
        } else {
            while let Some(funcs) = funcs_chunks.next() {
                if funcs_chunks.peek().is_some() {
                    self.append_function(
                        &funcs[0],
                        self.sierra_program.statements
                            [funcs[0].entry_point.0..funcs[1].entry_point.0]
                            .to_vec(),
                    );
                } else {
                    // Last pair
                    self.append_function(
                        &funcs[0],
                        self.sierra_program.statements
                            [funcs[0].entry_point.0..funcs[1].entry_point.0]
                            .to_vec(),
                    );
                    self.append_function(
                        &funcs[1],
                        self.sierra_program.statements
                            [funcs[1].entry_point.0..self.sierra_program.statements.len()]
                            .to_vec(),
                    );
                }
            }
        }

        // Analyze each function
        self.functions.iter_mut().for_each(|f| f.analyze());
    }
}
