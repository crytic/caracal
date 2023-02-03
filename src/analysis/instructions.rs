use super::dataflow::{Analysis, Domain, Forward};
use crate::core::instruction;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstructionDomain {
    Bottom,
    Top,
    /// Track the instruction's PC
    Instructions(HashSet<usize>),
}

impl Domain for InstructionDomain {
    fn bottom() -> Self {
        Self::Bottom
    }

    fn top() -> Self {
        Self::Top
    }

    fn join(&mut self, other: &Self) -> bool {
        let res = match (&self, other) {
            // If self is Top or other is Bottom we don't need to do anything
            (Self::Top, _) | (_, Self::Bottom) => return false,
            // The two instructions set are the same
            (Self::Instructions(a), Self::Instructions(b)) if a == b => return false,
            // We union the different instructions set
            (Self::Instructions(a), Self::Instructions(b)) => {
                let mut aa = a.clone();
                aa.extend(b);
                Self::Instructions(aa)
            }
            // If self is bottom and other is not, clone other in self
            (Self::Bottom, Self::Instructions(a)) => Self::Instructions(a.clone()),
            _ => Self::Top,
        };

        *self = res;
        true
    }
}

pub struct InstructionAnalysis;

impl Analysis for InstructionAnalysis {
    type Direction = Forward;
    type Domain = InstructionDomain;

    fn bottom_value(&self) -> Self::Domain {
        Self::Domain::Bottom
    }

    fn transfer_function(&self, state: &mut Self::Domain, instruction: &instruction::Instruction) {
        let pc = instruction.get_pc();

        let new_state = match state {
            InstructionDomain::Bottom => {
                let mut new_set = HashSet::new();
                new_set.insert(pc);
                InstructionDomain::Instructions(new_set)
            }
            InstructionDomain::Instructions(instructions) => {
                instructions.insert(pc);
                InstructionDomain::Instructions(instructions.clone())
            }
            InstructionDomain::Top => InstructionDomain::Top,
        };

        *state = new_state;
    }
}
