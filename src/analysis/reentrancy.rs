use super::dataflow::{Analysis, Domain, Forward};
use crate::core::cfg::Cfg;
use crate::core::function::Function;
use crate::core::{basic_block::BasicBlock, function::Type, instruction::Instruction};
use cairo_lang_sierra::extensions::core::CoreConcreteLibfunc;
use cairo_lang_sierra::extensions::core::{CoreLibfunc, CoreType};
use cairo_lang_sierra::program::GenStatement;
use cairo_lang_sierra::program_registry::ProgramRegistry;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ReentrancyInfo {
    pub external_calls: HashSet<BasicBlock>,
    pub storage_variables_read: HashSet<BasicBlock>,
    pub storage_variables_written: HashSet<BasicBlock>,
    /// Set of variables read before a function call. call -> variables
    pub variables_read_before_calls: HashMap<BasicBlock, HashSet<BasicBlock>>,
    pub events: HashSet<BasicBlock>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReentrancyDomain {
    Bottom,
    Top,
    State(Box<ReentrancyInfo>),
}

impl Domain for ReentrancyDomain {
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
            // The two reentrancy states are the same
            (Self::State(a), Self::State(b)) if a == b => return false,
            // We union the different reentrancy states set
            // Don't union storage_variables_written and events because it will be easier to write the detectors
            // if the values are kept only in the basic block where they happen (not propagated)
            (Self::State(a), Self::State(b)) => {
                let mut new_state = a.clone();
                new_state.external_calls.extend(b.external_calls.clone());
                new_state
                    .storage_variables_read
                    .extend(b.storage_variables_read.clone());
                new_state
                    .variables_read_before_calls
                    .extend(b.variables_read_before_calls.clone());
                Self::State(new_state)
            }
            // If self is bottom and other is not, clone other in self.
            // Don't clone storage_variables_written and events because it will be easier to write the detectors
            // if the values are kept only in the basic block where they happen (not propagated)
            (Self::Bottom, Self::State(a)) => Self::State(Box::new(ReentrancyInfo {
                external_calls: a.external_calls.clone(),
                storage_variables_read: a.storage_variables_read.clone(),
                storage_variables_written: HashSet::new(),
                variables_read_before_calls: a.variables_read_before_calls.clone(),
                events: HashSet::new(),
            })),
            _ => Self::Top,
        };

        *self = res;
        true
    }
}

#[derive(Clone, Debug)]
pub struct ReentrancyAnalysis;

impl Analysis for ReentrancyAnalysis {
    type Direction = Forward;
    type Domain = ReentrancyDomain;

    fn bottom_value(&self) -> Self::Domain {
        Self::Domain::Bottom
    }

    fn transfer_function(
        &self,
        basic_block: &BasicBlock,
        state: &mut Self::Domain,
        instruction: &Instruction,
        functions: &[Function],
        registry: &ProgramRegistry<CoreType, CoreLibfunc>,
    ) {
        ReentrancyAnalysis::transfer_function_helper(
            basic_block,
            state,
            instruction,
            functions,
            registry,
            &mut HashSet::new(),
        );
    }
}

impl ReentrancyAnalysis {
    fn transfer_function_helper(
        basic_block: &BasicBlock,
        state: &mut <ReentrancyAnalysis as Analysis>::Domain,
        instruction: &Instruction,
        functions: &[Function],
        registry: &ProgramRegistry<CoreType, CoreLibfunc>,
        private_functions_seen: &mut HashSet<String>,
    ) {
        match state {
            ReentrancyDomain::Bottom => {
                let new_info = ReentrancyInfo::default();
                *state = ReentrancyDomain::State(Box::new(new_info));
            }
            ReentrancyDomain::State(inner_state) => {
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
                                            inner_state
                                                .storage_variables_read
                                                .insert(basic_block.clone());
                                        } else if function_name.ends_with("::write") {
                                            inner_state
                                                .storage_variables_written
                                                .insert(basic_block.clone());
                                        }
                                    }
                                    Type::Event => {
                                        inner_state.events.insert(basic_block.clone());
                                    }
                                    // External and View are needed because it's possible to call self declared external functions within a private function
                                    Type::Private | Type::Loop | Type::External | Type::View => {
                                        if let GenStatement::Invocation(invoc) =
                                            instruction.get_statement()
                                        {
                                            let lib_func =
                                                registry.get_libfunc(&invoc.libfunc_id).expect(
                                                    "Library function not found in the registry",
                                                );
                                            if let CoreConcreteLibfunc::FunctionCall(f_called) =
                                                lib_func
                                            {
                                                for function in functions {
                                                    let function_name = function.name();
                                                    if function_name.as_str()
                                                        == f_called
                                                            .function
                                                            .id
                                                            .debug_name
                                                            .as_ref()
                                                            .unwrap()
                                                    {
                                                        if private_functions_seen
                                                            .contains(&function_name)
                                                        {
                                                            break;
                                                        }
                                                        private_functions_seen
                                                            .insert(function_name);

                                                        for bb in
                                                            function.get_cfg().get_basic_blocks()
                                                        {
                                                            if let Some(instruction) =
                                                                bb.get_function_call()
                                                            {
                                                                ReentrancyAnalysis::transfer_function_helper(
                                                                    bb,
                                                                    state,
                                                                    instruction,
                                                                    functions,
                                                                    registry,
                                                                    private_functions_seen,
                                                                );
                                                            }
                                                        }
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Type::AbiCallContract => {
                                        inner_state.external_calls.insert(basic_block.clone());
                                        inner_state.variables_read_before_calls.insert(
                                            basic_block.clone(),
                                            HashSet::from_iter(
                                                inner_state.storage_variables_read.clone(),
                                            ),
                                        );
                                    }

                                    _ => (),
                                }
                                break;
                            }
                        }
                    }
                }
            }

            ReentrancyDomain::Top => *state = ReentrancyDomain::Top,
        };
    }
}
