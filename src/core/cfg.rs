use std::collections::{HashMap, HashSet};

use super::basic_block::BasicBlock;
use super::function::Function;
use super::instruction::Instruction;
use cairo_lang_sierra::extensions::core::{CoreLibfunc, CoreType};
use cairo_lang_sierra::program::{BranchTarget, Statement as SierraStatement};
use cairo_lang_sierra::program_registry::ProgramRegistry;

pub trait Cfg {
    fn get_basic_blocks(&self) -> &[BasicBlock];
    fn get_basic_block(&self, id: usize) -> Option<&BasicBlock>;
}

#[derive(Debug, Clone, Default)]
pub struct CfgRegular {
    basic_blocks: Vec<BasicBlock>,
}

impl CfgRegular {
    pub fn new() -> Self {
        CfgRegular {
            basic_blocks: Vec::new(),
        }
    }

    pub fn analyze(
        &mut self,
        statements: &[SierraStatement],
        base_pc: usize,
        functions: &[Function],
        registry: &ProgramRegistry<CoreType, CoreLibfunc>,
        function_name: String,
    ) {
        self.compute_basic_blocks(statements, base_pc, functions, registry, function_name);
        self.compute_cfg();
    }

    fn compute_basic_blocks(
        &mut self,
        statements: &[SierraStatement],
        base_pc: usize,
        functions: &[Function],
        registry: &ProgramRegistry<CoreType, CoreLibfunc>,
        function_name: String,
    ) {
        // Track basic block ids
        let mut basic_block_counter = 0;
        let mut basic_blocks = Vec::new();

        // The instructions of the current block
        let mut instructions_current_block = Vec::new();

        // Tracks PCs of jump's target
        let mut target_pcs: HashSet<usize> = HashSet::new();

        for (i, statement) in statements.iter().enumerate() {
            let current_pc = base_pc + i;
            match statement {
                // Statement with a single branch. It's a statemnt that doesn't change the flow (Fallthrough) or an unconditional jump
                SierraStatement::Invocation(function) if function.branches.len() == 1 => {
                    match function.branches[0].target {
                        BranchTarget::Fallthrough => {
                            self.handle_statement_basic_block(
                                statement.clone(),
                                &target_pcs,
                                current_pc,
                                &mut basic_blocks,
                                &mut instructions_current_block,
                                &mut basic_block_counter,
                                functions,
                                registry,
                                function_name.clone(),
                            );
                        }
                        BranchTarget::Statement(pc) => {
                            // Unconditional jump
                            target_pcs.insert(pc.0);
                            self.handle_statement_basic_block(
                                statement.clone(),
                                &target_pcs,
                                current_pc,
                                &mut basic_blocks,
                                &mut instructions_current_block,
                                &mut basic_block_counter,
                                functions,
                                registry,
                                function_name.clone(),
                            );
                        }
                    }
                }
                // Statement with multiple branches
                SierraStatement::Invocation(function) => {
                    for branch in function.branches.iter() {
                        match branch.target {
                            BranchTarget::Statement(pc) => {
                                target_pcs.insert(pc.0);
                            }
                            BranchTarget::Fallthrough => {
                                target_pcs.insert(current_pc + 1);
                            }
                        }
                    }

                    self.handle_statement_basic_block(
                        statement.clone(),
                        &target_pcs,
                        current_pc,
                        &mut basic_blocks,
                        &mut instructions_current_block,
                        &mut basic_block_counter,
                        functions,
                        registry,
                        function_name.clone(),
                    );
                }
                SierraStatement::Return(_) => {
                    // Always terminate the current block
                    // Add the instruction in the current block
                    instructions_current_block
                        .push(Instruction::new(current_pc, statement.clone()));
                    // Create a new basic block
                    basic_blocks.push(BasicBlock::new(
                        function_name.clone(),
                        basic_block_counter,
                        instructions_current_block.clone(),
                        functions,
                        registry,
                    ));
                    // Clear the current instructions for the next basic block
                    instructions_current_block.clear();
                    basic_block_counter += 1;
                }
            }
        }

        self.basic_blocks = basic_blocks;
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_statement_basic_block(
        &self,
        statement: SierraStatement,
        target_pcs: &HashSet<usize>,
        current_pc: usize,
        basic_blocks: &mut Vec<BasicBlock>,
        instructions_current_block: &mut Vec<Instruction>,
        basic_block_counter: &mut usize,
        functions: &[Function],
        registry: &ProgramRegistry<CoreType, CoreLibfunc>,
        function_name: String,
    ) {
        // Add the instruction in the current block
        instructions_current_block.push(Instruction::new(current_pc, statement));

        // If the current statement is a target of a jump complete the current basic block
        if target_pcs.get(&(current_pc + 1)).is_some() {
            // Create a new basic block
            basic_blocks.push(BasicBlock::new(
                function_name,
                *basic_block_counter,
                instructions_current_block.clone(),
                functions,
                registry,
            ));
            // Clear the current instructions for the next basic block
            instructions_current_block.clear();
            *basic_block_counter += 1;
        }
    }

    fn compute_cfg(&mut self) {
        // Track the incoming edges - basic block id -> incoming basic blocks id
        let mut incoming: HashMap<usize, Vec<usize>> = HashMap::new();
        // Track the outgoing edges - basic block id -> outgoing basic blocks id
        let mut outgoing: HashMap<usize, Vec<usize>> = HashMap::new();

        for source_bb in self.basic_blocks.iter() {
            let last_instruction = source_bb
                .last_instruction()
                .expect("Basic Block with 0 instruction");
            if let SierraStatement::Invocation(function) = &last_instruction.get_statement() {
                for branch in function.branches.iter() {
                    match branch.target {
                        BranchTarget::Statement(destination) => {
                            self.handle_statement_cfg(
                                destination.0,
                                &mut incoming,
                                &mut outgoing,
                                source_bb.get_id(),
                            );
                        }
                        // Fallthrough, the target_pc is the subsequent PC
                        BranchTarget::Fallthrough => {
                            self.handle_statement_cfg(
                                last_instruction.get_pc() + 1,
                                &mut incoming,
                                &mut outgoing,
                                source_bb.get_id(),
                            );
                        }
                    }
                }
            }
        }

        for bb in self.basic_blocks.iter_mut() {
            // Add the outgoing basic block for the bb if any
            if let Some(destinations) = outgoing.get(&bb.get_id()) {
                destinations.iter().for_each(|d| bb.add_outgoing_bb(*d));
            }
            // Add the incoming basic block for the bb if any
            if let Some(sources) = incoming.get(&bb.get_id()) {
                sources.iter().for_each(|s| bb.add_incoming_bb(*s));
            }
        }
    }

    // Statement that can be a target of a jump
    fn handle_statement_cfg(
        &self,
        target_pc: usize,
        incoming: &mut HashMap<usize, Vec<usize>>,
        outgoing: &mut HashMap<usize, Vec<usize>>,
        source_bb_id: usize,
    ) {
        for bb in self.basic_blocks.iter() {
            if bb
                .first_instruction()
                .expect("Basic Block with 0 instruction")
                .get_pc()
                == target_pc
            {
                let destination_bb_id = bb.get_id();

                let sources = incoming.entry(destination_bb_id).or_default();
                sources.push(source_bb_id);
                let destinations = outgoing.entry(source_bb_id).or_default();
                destinations.push(destination_bb_id);
                break;
            }
        }
    }
}

impl Cfg for CfgRegular {
    fn get_basic_blocks(&self) -> &[BasicBlock] {
        &self.basic_blocks
    }

    fn get_basic_block(&self, id: usize) -> Option<&BasicBlock> {
        self.basic_blocks.get(id)
    }
}
