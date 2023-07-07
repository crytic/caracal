use std::collections::{HashMap, VecDeque};

use crate::core::function::Function;
use crate::core::{basic_block::BasicBlock, cfg::Cfg, instruction::Instruction};
use cairo_lang_sierra::extensions::core::{CoreLibfunc, CoreType};
use cairo_lang_sierra::program_registry::ProgramRegistry;

use super::traversal;

pub trait Direction {
    const IS_FORWARD: bool;

    /// Apply the transfer function for the current basic block and propagate
    #[allow(clippy::too_many_arguments)]
    fn apply_transfer_function<A: Analysis + Clone>(
        analysis: &A,
        current_state: &mut AnalysisState<A>,
        basic_block: BasicBlock,
        worklist: &mut VecDeque<BasicBlock>,
        global_state: &mut HashMap<usize, AnalysisState<A>>,
        cfg: &dyn Cfg,
        functions: &[Function],
        registry: &ProgramRegistry<CoreType, CoreLibfunc>,
    );
}

pub struct Forward;

impl Direction for Forward {
    const IS_FORWARD: bool = true;

    fn apply_transfer_function<A: Analysis + Clone>(
        analysis: &A,
        current_state: &mut AnalysisState<A>,
        basic_block: BasicBlock,
        worklist: &mut VecDeque<BasicBlock>,
        global_state: &mut HashMap<usize, AnalysisState<A>>,
        cfg: &dyn Cfg,
        functions: &[Function],
        registry: &ProgramRegistry<CoreType, CoreLibfunc>,
    ) {
        for instruction in basic_block.get_instructions().iter() {
            analysis.transfer_function(
                &basic_block,
                &mut current_state.pre,
                instruction,
                functions,
                registry,
            );
        }

        // Set the post state of the current block
        global_state.get_mut(&basic_block.get_id()).unwrap().post = current_state.pre.clone();

        // Propagate
        for bb in basic_block.get_outgoing_basic_blocks() {
            let changed = global_state
                .get_mut(bb)
                .unwrap()
                .pre
                .join(&current_state.pre);
            if changed {
                let basic_block = cfg.get_basic_block(*bb).unwrap().clone();
                if !worklist.contains(&basic_block) {
                    worklist.push_back(basic_block);
                }
            }
        }
    }
}

pub struct Backward;

impl Direction for Backward {
    const IS_FORWARD: bool = false;

    fn apply_transfer_function<A: Analysis + Clone>(
        analysis: &A,
        current_state: &mut AnalysisState<A>,
        basic_block: BasicBlock,
        worklist: &mut VecDeque<BasicBlock>,
        global_state: &mut HashMap<usize, AnalysisState<A>>,
        cfg: &dyn Cfg,
        functions: &[Function],
        registry: &ProgramRegistry<CoreType, CoreLibfunc>,
    ) {
        for instruction in basic_block.get_instructions().iter().rev() {
            analysis.transfer_function(
                &basic_block,
                &mut current_state.post,
                instruction,
                functions,
                registry,
            );
        }

        // Set the post state of the current block
        global_state.get_mut(&basic_block.get_id()).unwrap().post = current_state.pre.clone();

        // Propagate
        for bb in basic_block.get_incoming_basic_blocks() {
            let changed = global_state
                .get_mut(bb)
                .unwrap()
                .pre
                .join(&current_state.post);
            if changed {
                let basic_block = cfg.get_basic_block(*bb).unwrap().clone();
                if !worklist.contains(&basic_block) {
                    worklist.push_back(basic_block);
                }
            }
        }
    }
}

pub trait Domain {
    /// The top element of the domain
    fn top() -> Self;
    /// The bottom element of the domain
    fn bottom() -> Self;
    /// Computes the least upper bound of two elements and store the result in self
    /// Return true if self changed
    fn join(&mut self, other: &Self) -> bool;
}

pub trait Analysis {
    /// The type that holds the state of the dataflow analysis
    type Domain: Domain + Clone + std::fmt::Debug;
    /// The direction of the analysis
    type Direction: Direction;

    /// The function applied to each instruction
    fn transfer_function(
        &self,
        basic_block: &BasicBlock,
        state: &mut Self::Domain,
        instruction: &Instruction,
        functions: &[Function],
        registry: &ProgramRegistry<CoreType, CoreLibfunc>,
    );
    /// The initial state when entering a basic block
    fn bottom_value(&self) -> Self::Domain;
}

#[derive(Clone, Debug)]
pub struct AnalysisState<A: Analysis + Clone> {
    /// State at the start of the basic block
    pub pre: A::Domain,
    /// State at the end of the basic block
    pub post: A::Domain,
}

/// Engine to solve data flow problems
pub struct Engine<'a, A: Analysis + Clone> {
    cfg: &'a dyn Cfg,
    analysis: A,
    state: HashMap<usize, AnalysisState<A>>,
}

impl<'a, A> Engine<'a, A>
where
    A: Analysis + Clone,
{
    /// Create a new Engine to solve the analysis on the cfg
    pub fn new(cfg: &'a dyn Cfg, analysis: A) -> Self {
        let basic_blocks = cfg.get_basic_blocks().len();
        let mut state = HashMap::with_capacity(basic_blocks);

        // Initialize the state for each basic block with the bottom value
        for i in 0..basic_blocks {
            state.insert(
                i,
                AnalysisState {
                    pre: analysis.bottom_value(),
                    post: analysis.bottom_value(),
                },
            );
        }

        Engine {
            cfg,
            analysis,
            state,
        }
    }

    /// Run the analysis to a fix point
    pub fn run_analysis(
        &mut self,
        functions: &[Function],
        registry: &ProgramRegistry<CoreType, CoreLibfunc>,
    ) {
        let basic_blocks = self.cfg.get_basic_blocks();
        let mut worklist: VecDeque<BasicBlock> = VecDeque::with_capacity(basic_blocks.len());

        if A::Direction::IS_FORWARD {
            for bb in traversal::ReversePostorder::new(self.cfg).result().clone() {
                worklist.push_back(bb);
            }
        } else {
            for bb in traversal::Postorder::new(self.cfg).result().clone() {
                worklist.push_back(bb);
            }
        }

        while let Some(bb) = worklist.pop_front() {
            let mut state = self
                .state
                .get_mut(&bb.get_id())
                .expect("Basic block state not found during data flow analysis")
                .clone();

            A::Direction::apply_transfer_function(
                &self.analysis,
                &mut state,
                bb,
                &mut worklist,
                &mut self.state,
                self.cfg,
                functions,
                registry,
            );
        }
    }

    /// Return the result of the analysis after run_analysis was called
    pub fn result(&self) -> &HashMap<usize, AnalysisState<A>> {
        &self.state
    }
}
