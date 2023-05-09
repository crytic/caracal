use super::detector::{Confidence, Detector, Impact, Result};
use crate::core::core_unit::CoreUnit;
use crate::core::function::Type;
use cairo_lang_sierra::extensions::core::CoreConcreteLibfunc;
use cairo_lang_sierra::program::Statement as SierraStatement;
use std::collections::HashSet;

// Note: Inlined functions are reported as dead code

#[derive(Default)]
pub struct DeadCode {}

impl Detector for DeadCode {
    fn name(&self) -> &str {
        "dead-code"
    }

    fn description(&self) -> &str {
        "Detect private functions never used"
    }

    fn confidence(&self) -> Confidence {
        Confidence::Medium
    }

    fn impact(&self) -> Impact {
        Impact::Low
    }

    fn run(&self, core: &CoreUnit) -> Vec<Result> {
        let mut results = Vec::new();
        let compilation_unit = core.get_compilation_unit();
        let mut private_functions: HashSet<String> = compilation_unit
            .functions()
            .filter(|f| *f.ty() == Type::Private)
            .map(|f| f.name())
            .collect();

        // We must iterate over all functions because some user implemented functions
        // such as when implementing Serde/StorageAccess trait are called by non user-defined functions
        for f in compilation_unit.functions() {
            for private_call_stmt in f.private_functions_calls() {
                if let SierraStatement::Invocation(invoc) = private_call_stmt {
                    // Get the concrete libfunc called
                    let libfunc = compilation_unit
                        .registry()
                        .get_libfunc(&invoc.libfunc_id)
                        .expect("Library function not found in the registry");

                    if let CoreConcreteLibfunc::FunctionCall(f_called) = libfunc {
                        // We remove the function called from private_functions
                        private_functions.remove(
                            &f_called
                                .function
                                .id
                                .debug_name
                                .as_ref()
                                .unwrap()
                                .to_string(),
                        );
                    }
                }
            }
        }

        // We rsplit the private function to get the function name and the first part is the module where the function is defined
        private_functions
            .iter()
            .map(|private_function| private_function.rsplit_once("::").unwrap())
            .for_each(|(function_declaration, function_name)| {
                results.push(Result {
                    name: self.name().to_string(),
                    impact: self.impact(),
                    confidence: self.confidence(),
                    message: format!(
                        "Function {} defined in {} is never used",
                        function_name, function_declaration
                    ),
                })
            });

        results
    }
}
