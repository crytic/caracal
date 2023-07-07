use std::collections::HashSet;

use super::detector::{Confidence, Detector, Impact, Result};
use crate::analysis::dataflow::AnalysisState;
use crate::analysis::reentrancy::ReentrancyDomain;
use crate::core::core_unit::CoreUnit;

#[derive(Default)]
pub struct ReentrancyBenign;

impl Detector for ReentrancyBenign {
    fn name(&self) -> &str {
        "reentrancy-benign"
    }

    fn description(&self) -> &str {
        "Detect when a storage variable is written after an external call but not read before"
    }

    fn confidence(&self) -> Confidence {
        Confidence::Medium
    }

    fn impact(&self) -> Impact {
        Impact::Low
    }

    fn run(&self, core: &CoreUnit) -> Vec<Result> {
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
                        for call in reentrancy_info.external_calls.iter() {
                            if let Some(current_vars_read_before_call) = reentrancy_info
                                .variables_read_before_calls
                                .iter()
                                .find(|entry| entry.0.get_id() == call.get_id())
                            {
                                let vars_read: Vec<String> = current_vars_read_before_call
                                    .1
                                    .iter()
                                    .map(|var| {
                                        var.get_storage_variable_read()
                                            .as_ref()
                                            .unwrap()
                                            .get_statement()
                                            .to_string()
                                            .rsplit_once("::")
                                            .unwrap()
                                            .0
                                            .to_string()
                                    })
                                    .collect();
                                for written_variable in
                                    reentrancy_info.storage_variables_written.iter()
                                {
                                    let written_variable_name = written_variable
                                        .get_storage_variable_written()
                                        .as_ref()
                                        .unwrap()
                                        .get_statement()
                                        .to_string()
                                        .rsplit_once("::")
                                        .unwrap()
                                        .0
                                        .to_string();
                                    if !vars_read.contains(&written_variable_name) {
                                        results.insert(Result {
                                            name: self.name().to_string(),
                                            impact: self.impact(),
                                            confidence: self.confidence(),
                                            message: format!(
                                                "Reentrancy in {}\n\tExternal call {} done in {}\n\tVariable written after {} in {}.",
                                                f.name(),
                                                call.get_external_call()
                                                    .as_ref()
                                                    .unwrap()
                                                    .get_statement(),
                                                call.get_function(),
                                                written_variable
                                                    .get_storage_variable_written()
                                                    .as_ref()
                                                    .unwrap()
                                                    .get_statement(),
                                                written_variable.get_function()
                                            ),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Vec::from_iter(results)
    }
}
