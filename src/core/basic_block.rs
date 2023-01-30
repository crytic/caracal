use super::instruction::Instruction;

#[derive(Debug, Clone)]
pub struct BasicBlock {
    id: usize,
    instructions: Vec<Instruction>,
    incoming_basic_blocks: Vec<usize>,
    outgoing_basic_blocks: Vec<usize>,
}

impl BasicBlock {
    pub fn new(id: usize, instructions: Vec<Instruction>) -> Self {
        BasicBlock {
            id,
            instructions,
            incoming_basic_blocks: Vec::new(),
            outgoing_basic_blocks: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub(super) fn get_mut_id(&mut self) -> &mut usize {
        &mut self.id
    }

    pub fn get_instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }

    pub(super) fn get_mut_instructions(&mut self) -> &mut Vec<Instruction> {
        &mut self.instructions
    }

    pub fn get_incoming_basic_blocks(&self) -> &Vec<usize> {
        &self.incoming_basic_blocks
    }

    pub fn get_outgoing_basic_blocks(&self) -> &Vec<usize> {
        &self.outgoing_basic_blocks
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
}
