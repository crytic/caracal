use std::collections::HashSet;

use crate::core::basic_block::BasicBlock;
use crate::core::cfg::Cfg;

pub struct Postorder<'a> {
    cfg: &'a dyn Cfg,
    result: Vec<BasicBlock>,
    visited: HashSet<usize>,
}

impl<'a> Postorder<'a> {
    pub fn new(cfg: &'a dyn Cfg) -> Self {
        let mut postorder = Postorder {
            cfg,
            result: Vec::new(),
            visited: HashSet::new(),
        };
        let basic_blocks = cfg.get_basic_blocks();

        if basic_blocks.is_empty() {
            return postorder;
        }

        postorder.compute_postorder(&basic_blocks[0]);

        postorder
    }

    fn compute_postorder(&mut self, basic_block: &BasicBlock) {
        if !self.visited.insert(basic_block.get_id()) {
            return;
        }

        for outgoing_bb in basic_block.get_outgoing_basic_blocks().iter() {
            let bb = self
                .cfg
                .get_basic_block(*outgoing_bb)
                .expect("Basic block not found when computing postorder");
            self.compute_postorder(bb);
        }

        self.result.push(basic_block.clone());
    }

    pub fn result(&self) -> &Vec<BasicBlock> {
        &self.result
    }
}

pub struct ReversePostorder {
    result: Vec<BasicBlock>,
}

impl ReversePostorder {
    pub fn new(cfg: &dyn Cfg) -> Self {
        let mut reverse_postorder = ReversePostorder { result: Vec::new() };

        reverse_postorder.result = Postorder::new(cfg).result().clone();
        reverse_postorder.result.reverse();

        reverse_postorder
    }

    pub fn result(&self) -> &Vec<BasicBlock> {
        &self.result
    }
}
