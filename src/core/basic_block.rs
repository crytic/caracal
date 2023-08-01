use super::instruction::Instruction;
use crate::core::function::{Function, Type};
use std::hash::{Hash, Hasher};

use cairo_lang_sierra::extensions::core::{CoreConcreteLibfunc, CoreLibfunc, CoreType};
use cairo_lang_sierra::program::GenStatement;
use cairo_lang_sierra::program_registry::ProgramRegistry;

#[derive(Debug, Clone, Default)]
pub struct BasicBlock {
    function: String,
    id: usize,
    instructions: Vec<Instruction>,
    incoming_basic_blocks: Vec<usize>,
    outgoing_basic_blocks: Vec<usize>,
    private_call: Option<Instruction>,
    external_call: Option<Instruction>,
    library_call: Option<Instruction>,
    storage_variable_read: Option<Instruction>,
    storage_variable_written: Option<Instruction>,
    event_emitted: Option<Instruction>,
}

impl PartialEq for BasicBlock {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for BasicBlock {}

impl Hash for BasicBlock {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.function.hash(state);
        self.id.hash(state);
    }
}

impl BasicBlock {
    pub fn new(
        function: String,
        id: usize,
        instructions: Vec<Instruction>,
        functions: &[Function],
        registry: &ProgramRegistry<CoreType, CoreLibfunc>,
    ) -> Self {
        let mut bb = BasicBlock {
            function,
            id,
            instructions,
            ..Default::default()
        };

        bb.analyze(functions, registry);

        bb
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_function(&self) -> &str {
        &self.function
    }

    pub fn get_instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }

    pub fn get_incoming_basic_blocks(&self) -> &Vec<usize> {
        &self.incoming_basic_blocks
    }

    pub fn get_outgoing_basic_blocks(&self) -> &Vec<usize> {
        &self.outgoing_basic_blocks
    }

    /// Return the function call in this basic block
    pub fn get_function_call(&self) -> Option<&Instruction> {
        // Assume there can be only one function call in each basic block
        // when a function call is executed sierra always check for panics so it should be true

        if let Some(instruction) = &self.event_emitted {
            return Some(instruction);
        }
        if let Some(instruction) = &self.external_call {
            return Some(instruction);
        }
        if let Some(instruction) = &self.library_call {
            return Some(instruction);
        }
        if let Some(instruction) = &self.private_call {
            return Some(instruction);
        }
        if let Some(instruction) = &self.storage_variable_read {
            return Some(instruction);
        }
        if let Some(instruction) = &self.storage_variable_written {
            return Some(instruction);
        }

        None
    }

    pub fn add_incoming_bb(&mut self, basic_block: usize) {
        self.incoming_basic_blocks.push(basic_block);
    }

    pub fn add_outgoing_bb(&mut self, basic_block: usize) {
        self.outgoing_basic_blocks.push(basic_block);
    }

    pub fn remove_incoming_bb(&mut self, basic_block: usize) {
        let pos = self
            .incoming_basic_blocks
            .iter()
            .position(|id| *id == basic_block)
            .expect("Trying to remove an incoming edge that doesn't exist");
        self.incoming_basic_blocks.remove(pos);
    }

    pub fn remove_outgoing_bb(&mut self, basic_block: usize) {
        let pos = self
            .outgoing_basic_blocks
            .iter()
            .position(|id| *id == basic_block)
            .expect("Trying to remove an outgoing edge that doesn't exist");
        self.outgoing_basic_blocks.remove(pos);
    }

    pub fn last_instruction(&self) -> Option<&Instruction> {
        self.instructions.last()
    }

    pub fn first_instruction(&self) -> Option<&Instruction> {
        self.instructions.first()
    }

    pub fn get_private_call(&self) -> &Option<Instruction> {
        &self.private_call
    }

    pub fn get_external_call(&self) -> &Option<Instruction> {
        &self.external_call
    }

    pub fn get_library_call(&self) -> &Option<Instruction> {
        &self.library_call
    }

    pub fn get_storage_variable_read(&self) -> &Option<Instruction> {
        &self.storage_variable_read
    }

    pub fn get_storage_variable_written(&self) -> &Option<Instruction> {
        &self.storage_variable_written
    }

    pub fn get_event_emitted(&self) -> &Option<Instruction> {
        &self.event_emitted
    }

    pub fn analyze(
        &mut self,
        functions: &[Function],
        registry: &ProgramRegistry<CoreType, CoreLibfunc>,
    ) {
        for instruction in self.instructions.iter() {
            if let GenStatement::Invocation(invoc) = instruction.get_statement() {
                let lib_func = registry
                    .get_libfunc(&invoc.libfunc_id)
                    .expect("Library function not found in the registry");

                if let CoreConcreteLibfunc::FunctionCall(f_called) = lib_func {
                    // We search for the function called in our list of functions to know its type
                    for function in functions {
                        let function_name = function.name();
                        if function_name.as_str()
                            == f_called.function.id.debug_name.as_ref().unwrap()
                        {
                            match function.ty() {
                                Type::Storage => {
                                    if function_name.ends_with("::read") {
                                        self.storage_variable_read = Some(instruction.clone());
                                    } else if function_name.ends_with("::write") {
                                        self.storage_variable_written = Some(instruction.clone());
                                    }
                                }
                                Type::Event => self.event_emitted = Some(instruction.clone()),
                                Type::Private => self.private_call = Some(instruction.clone()),
                                Type::AbiCallContract => {
                                    self.external_call = Some(instruction.clone())
                                }
                                Type::AbiLibraryCall => {
                                    self.library_call = Some(instruction.clone())
                                }
                                _ => (),
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
}
