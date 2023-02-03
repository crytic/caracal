use crate::core::{basic_block::BasicBlock, cfg::Cfg, instruction::Instruction};
use std::collections::{HashMap, HashSet, VecDeque};

use super::traversal;

pub trait Direction {
    const IS_FORWARD: bool;

    /// Apply the transfer function for the current basic block and propagate
    fn apply_transfer_function<A: Analysis>(
        analysis: &A,
        current_state: &mut A::Domain,
        basic_block: BasicBlock,
        worklist: &mut VecDeque<BasicBlock>,
        state: &mut HashMap<usize, A::Domain>,
        cfg: &dyn Cfg,
    );
}

pub struct Forward;

impl Direction for Forward {
    const IS_FORWARD: bool = true;

    fn apply_transfer_function<A: Analysis>(
        analysis: &A,
        current_state: &mut A::Domain,
        basic_block: BasicBlock,
        worklist: &mut VecDeque<BasicBlock>,
        state: &mut HashMap<usize, A::Domain>,
        cfg: &dyn Cfg,
    ) {
        for instruction in basic_block.get_instructions().iter() {
            analysis.transfer_function(current_state, instruction);
        }

        // Propagate
        for bb in basic_block.get_outgoing_basic_blocks() {
            let changed = state.get_mut(bb).unwrap().join(current_state);
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

    fn apply_transfer_function<A: Analysis>(
        analysis: &A,
        current_state: &mut A::Domain,
        basic_block: BasicBlock,
        worklist: &mut VecDeque<BasicBlock>,
        state: &mut HashMap<usize, A::Domain>,
        cfg: &dyn Cfg,
    ) {
        for instruction in basic_block.get_instructions().iter().rev() {
            analysis.transfer_function(current_state, instruction);
        }

        // Propagate
        for bb in basic_block.get_incoming_basic_blocks() {
            let changed = state.get_mut(bb).unwrap().join(current_state);
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
    fn transfer_function(&self, state: &mut Self::Domain, instruction: &Instruction);
    /// The initial state when entering a basic block
    fn bottom_value(&self) -> Self::Domain;
}

pub struct Engine<'a, A: Analysis> {
    cfg: &'a dyn Cfg,
    analysis: A,
    state: HashMap<usize, A::Domain>,
}

impl<'a, A> Engine<'a, A>
where
    A: Analysis,
{
    pub fn new(cfg: &'a dyn Cfg, analysis: A) -> Self {
        let basic_blocks = cfg.get_basic_blocks().len();
        let mut state = HashMap::with_capacity(basic_blocks);

        // Initialize the state for each basic block with the bottom value
        for i in 0..basic_blocks {
            state.insert(i, analysis.bottom_value());
        }

        Engine {
            cfg,
            analysis,
            state,
        }
    }

    pub fn run_analysis(&mut self) {
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
                .get(&bb.get_id())
                .expect("Basic block state not found during data flow analysis")
                .clone();

            A::Direction::apply_transfer_function(
                &self.analysis,
                &mut state,
                bb,
                &mut worklist,
                &mut self.state,
                self.cfg,
            );
        }
    }

    pub fn result(&self) -> &HashMap<usize, A::Domain> {
        &self.state
    }
}
