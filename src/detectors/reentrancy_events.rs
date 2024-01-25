use std::collections::HashSet;

use super::detector::{Confidence, Detector, Impact, Result};
use crate::analysis::dataflow::AnalysisState;
use crate::analysis::reentrancy::ReentrancyDomain;
use crate::core::core_unit::CoreUnit;

#[derive(Default)]
pub struct ReentrancyEvents;

impl Detector for ReentrancyEvents {
    fn name(&self) -> &str {
        "reentrancy-events"
    }

    fn description(&self) -> &str {
        "Detect when an event is emitted after an external call leading to out-of-order events"
    }

    fn confidence(&self) -> Confidence {
        Confidence::Medium
    }

    fn impact(&self) -> Impact {
        Impact::Low
    }

    fn run(&self, core: &CoreUnit) -> HashSet<Result> {
        let mut results: HashSet<Result> = HashSet::new();
        let compilation_units = core.get_compilation_units();

        for compilation_unit in compilation_units {
            for f in compilation_unit.functions_user_defined() {
                for bb_info in f.analyses().reentrancy.iter() {
                    if let AnalysisState {
                        post: ReentrancyDomain::State(reentrancy_info),
                        ..
                    } = bb_info.1
                    {
                        for event in reentrancy_info.events.iter() {
                            for call in reentrancy_info.external_calls.iter() {
                                let external_function_call = format!(
                                    "{}",
                                    call.get_external_call().as_ref().unwrap().get_statement()
                                );

                                if let Some(safe_external_calls) = core.get_safe_external_calls() {
                                    if safe_external_calls
                                        .iter()
                                        .any(|f_name| external_function_call.contains(f_name))
                                    {
                                        continue;
                                    }
                                }

                                results.insert(Result {
                                    name: self.name().to_string(),
                                    impact: self.impact(),
                                    confidence: self.confidence(),
                                    message: format!(
                                        "Reentrancy in {}\n\tExternal call {} done in {}\n\tEvent emitted after {} in {}.",
                                        f.name(),
                                        external_function_call,
                                        call.get_function(),
                                        event.get_event_emitted().as_ref().unwrap().get_statement(),
                                        event.get_function()
                                    ),
                                });
                            }
                        }
                    }
                }
            }
        }

        results
    }
}
