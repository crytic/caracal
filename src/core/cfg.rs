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

#[derive(Debug, Clone, Default)]
pub struct CfgOptimized {
    basic_blocks: Vec<BasicBlock>,
}

impl CfgOptimized {
    pub fn new() -> Self {
        CfgOptimized {
            basic_blocks: Vec::new(),
        }
    }

    pub fn analyze(&mut self, basic_blocks: Vec<BasicBlock>) {
        self.basic_blocks = basic_blocks;
        self.remove_redundant_bbs();
        self.rename_bbs();
    }

    fn get_mut_basic_block(&mut self, id: usize) -> Option<&mut BasicBlock> {
        self.basic_blocks.iter_mut().find(|bb| bb.get_id() == id)
    }

    // Should be used during optimization passes because the basic blocks may not be in the correct sequence
    // which means when we remove basic_blocks from [1,2,3,4] becomes [1,3,4]
    // so we can't just get the basic block at id
    fn get_basic_block_not_renamed(&self, id: usize) -> Option<&BasicBlock> {
        self.basic_blocks.iter().find(|bb| bb.get_id() == id)
    }

    /// Merge a basic block with 1 outgoing basic block to the following basic block if it has a single incoming basic block
    fn remove_redundant_bbs(&mut self) {
        // source basic block id | destination basic block id | destination basic block's outgoing edges to add in the source basic block | instructions to add in the source basic block
        let mut bb_to_merge: Vec<(usize, usize, Vec<usize>, Vec<Instruction>)> = Vec::new();

        for bb in self.basic_blocks.iter() {
            if bb.get_outgoing_basic_blocks().len() == 1 {
                let outgoing_bb = self
                    .get_basic_block_not_renamed(bb.get_outgoing_basic_blocks()[0])
                    .expect("Outgoing basic block not found");
                if outgoing_bb.get_incoming_basic_blocks().len() == 1 {
                    bb_to_merge.push((
                        bb.get_id(),
                        outgoing_bb.get_id(),
                        outgoing_bb.get_outgoing_basic_blocks().clone(),
                        outgoing_bb.get_instructions().clone(),
                    ));
                }
            }
        }

        for (bb_source_id, bb_destination_id, new_outgoing_bbs, new_instructions) in
            bb_to_merge.iter()
        {
            let bb_to_mutate = self
                .get_mut_basic_block(*bb_source_id)
                .expect("Source basic block not found");
            bb_to_mutate.remove_outgoing_bb(*bb_destination_id);
            for bb in new_outgoing_bbs {
                bb_to_mutate.add_outgoing_bb(*bb);
            }
            // Remove the last instruction
            // We assume it's a jump, maybe we could check it
            bb_to_mutate.get_mut_instructions().pop();
            // Add the destination basic block instructions
            bb_to_mutate
                .get_mut_instructions()
                .extend(new_instructions.clone());
        }

        // Update incoming edges for the outgoing basic block of the removed basic block
        for (bb_source_id, bb_destination_id, new_outgoing_bbs, _) in bb_to_merge.iter() {
            let bb_index_to_remove = self
                .basic_blocks
                .iter()
                .position(|bb| bb.get_id() == *bb_destination_id)
                .expect("Didn't find basic block to remove");
            self.basic_blocks.remove(bb_index_to_remove);
            for bb in new_outgoing_bbs {
                // Delete the incoming edge from the removed basic block and add bb_source_id
                let bb_to_mutate = self
                    .get_mut_basic_block(*bb)
                    .expect("New outgoing basic block not found");
                bb_to_mutate.remove_incoming_bb(*bb_destination_id);
                bb_to_mutate.add_incoming_bb(*bb_source_id);
            }
        }
    }

    fn rename_bbs(&mut self) {
        // Track how many basic block were removed and then subtract that value from the next basic block id
        let mut bb_counter_to_subtract = 0;
        // Track bb id => new id
        let mut bb_to_new_id = HashMap::new();

        for (i, bb) in self.basic_blocks.iter().enumerate() {
            let bb_id = bb.get_id();
            // Check if the bb_id is in the correct sequence
            if i != bb_id {
                // A basic block was removed between this block and the previous block
                // we need to increment the bb_counter_to_subtract
                // [0,1,2,3]
                // [0,2,3]
                //    ^
                // 2 - 1 > 0 -> bb_counter_to_subtract += 1
                // new id = 2 - 1 = 1
                // [0,1,3]
                //      ^
                // 3 - 2 > 1 -> No
                // new id = 3 - 1 = 2
                // [0,1,2]
                if bb_id - i > bb_counter_to_subtract {
                    bb_counter_to_subtract += 1;
                }
                let bb_new_id = bb_id - bb_counter_to_subtract;
                bb_to_new_id.insert(bb_id, bb_new_id);
            }
        }

        // Update the id, incoming and outgoing basic blocks
        for bb in self.basic_blocks.iter_mut() {
            if let Some(new_id) = bb_to_new_id.get(&bb.get_id()) {
                let bb_id = bb.get_mut_id();
                *bb_id = *new_id;
            }

            for incoming_bb in bb.get_incoming_basic_blocks().clone() {
                if let Some(new_id) = bb_to_new_id.get(&incoming_bb) {
                    bb.remove_incoming_bb(incoming_bb);
                    bb.add_incoming_bb(*new_id);
                }
            }
            for outgoing_bb in bb.get_outgoing_basic_blocks().clone() {
                if let Some(new_id) = bb_to_new_id.get(&outgoing_bb) {
                    bb.remove_outgoing_bb(outgoing_bb);
                    bb.add_outgoing_bb(*new_id);
                }
            }
        }
    }
}

impl Cfg for CfgOptimized {
    fn get_basic_blocks(&self) -> &[BasicBlock] {
        &self.basic_blocks
    }

    fn get_basic_block(&self, id: usize) -> Option<&BasicBlock> {
        self.basic_blocks.get(id)
    }
}
