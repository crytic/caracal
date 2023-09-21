use super::detector::{Confidence, Detector, Impact, Result};
use crate::analysis::dataflow::AnalysisState;
use crate::analysis::reentrancy::ReentrancyDomain;
use crate::core::core_unit::CoreUnit;
use crate::core::function::Type;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Default)]
pub struct ReadOnlyReentrancy;

impl Detector for ReadOnlyReentrancy {
    fn name(&self) -> &str {
        "read-only-reentrancy"
    }

    fn description(&self) -> &str {
        "Detect when a view function read a storage variable written after an external call"
    }

    fn confidence(&self) -> Confidence {
        Confidence::Medium
    }

    fn impact(&self) -> Impact {
        Impact::Medium
    }

    fn run(&self, core: &CoreUnit) -> HashSet<Result> {
        let mut results: HashSet<Result> = HashSet::new();
        let compilation_units = core.get_compilation_units();

        for compilation_unit in compilation_units {
            // Key the storage variable read - Value the functions name where it's read
            let mut vars_read: HashMap<String, HashSet<String>> = HashMap::new();

            for f in compilation_unit
                .functions_user_defined()
                .filter(|f| f.ty() == &Type::View)
            {
                for storage_var_read in f.storage_vars_read() {
                    let var_read = storage_var_read
                        .to_string()
                        .rsplit_once("::")
                        .unwrap()
                        .0
                        .to_string();
                    let functions_name = vars_read.entry(var_read).or_insert(HashSet::new());
                    functions_name.insert(f.name());
                }
            }

            for f in compilation_unit.functions_user_defined() {
                for bb_info in f.analyses().reentrancy.iter() {
                    if let AnalysisState {
                        post: ReentrancyDomain::State(reentrancy_info),
                        ..
                    } = bb_info.1
                    {
                        for call in reentrancy_info.external_calls.iter() {
                            for written_variable in reentrancy_info.storage_variables_written.iter()
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

                                if vars_read.contains_key(&written_variable_name) {
                                    for view_function in
                                        vars_read.get(&written_variable_name).unwrap()
                                    {
                                        results.insert(Result {
                                            name: self.name().to_string(),
                                            impact: self.impact(),
                                            confidence: self.confidence(),
                                            message: format!(
                                                "Read only reentrancy in {}\n\tExternal call {} done in {}\n\tVariable written after {} in {}",
                                                view_function,
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
                                                written_variable.get_function(),
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

        results
    }
}
