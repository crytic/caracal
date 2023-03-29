use super::detector::{Confidence, Detector, Impact, Result};
use crate::core::compilation_unit::CompilationUnit;
use crate::core::core_unit::CoreUnit;
use crate::utils::filter_builtins_from_arguments;
use cairo_lang_sierra::extensions::{
    core::CoreConcreteLibfunc, lib_func::ParamSignature, starknet::StarkNetConcreteLibfunc,
};
use cairo_lang_sierra::ids::VarId;
use cairo_lang_sierra::program::{GenStatement, Statement as SierraStatement, StatementIdx};

#[derive(Default)]
pub struct ControlledLibraryCall {}

impl Detector for ControlledLibraryCall {
    fn name(&self) -> &str {
        "controlled-library-call"
    }

    fn description(&self) -> &str {
        "Detect library calls with a user controlled class hash"
    }

    fn confidence(&self) -> Confidence {
        Confidence::Medium
    }

    fn impact(&self) -> Impact {
        Impact::High
    }

    fn run(&self, core: &CoreUnit) -> Vec<Result> {
        let mut results = Vec::new();
        let compilation_unit = core.get_compilation_unit();

        for f in compilation_unit.functions_user_defined() {
            // Check for library call made with the "interface" a trait with the ABI attribute
            for lib_call in f.library_functions_calls() {
                if let SierraStatement::Invocation(invoc) = lib_call {
                    // Get the concrete libfunc called
                    let libfunc = compilation_unit
                        .registry()
                        .get_libfunc(&invoc.libfunc_id)
                        .expect("Library function not found in the registry");

                    // We need this to get the signature of the function called to filter the builtins and get the class hash argument
                    if let CoreConcreteLibfunc::FunctionCall(abi_function) = libfunc {
                        self.check_user_controlled(
                            &mut results,
                            &abi_function.signature.param_signatures,
                            invoc.args.clone(),
                            compilation_unit,
                            &f.name(),
                            lib_call,
                        );
                    }
                }
            }

            // Check for library call made with the syscall
            for s in f.get_statements().iter() {
                if let SierraStatement::Invocation(invoc) = s {
                    // Get the concrete libfunc called
                    let libfunc = compilation_unit
                        .registry()
                        .get_libfunc(&invoc.libfunc_id)
                        .expect("Library function not found in the registry");

                    // We care only about a library call
                    if let CoreConcreteLibfunc::StarkNet(StarkNetConcreteLibfunc::LibraryCall(l)) =
                        libfunc
                    {
                        println!("in syscall checking...");
                        self.check_user_controlled(
                            &mut results,
                            &l.signature.param_signatures,
                            invoc.args.clone(),
                            compilation_unit,
                            &f.name(),
                            s,
                        );
                    }
                }
            }
        }

        results
    }
}

impl ControlledLibraryCall {
    fn check_user_controlled(
        &self,
        results: &mut Vec<Result>,
        formal_params: &[ParamSignature],
        actual_params: Vec<VarId>,
        compilation_unit: &CompilationUnit,
        function_name: &str,
        statement: &GenStatement<StatementIdx>,
    ) {
        // The first argument is the class hash
        let class_hash = filter_builtins_from_arguments(formal_params, actual_params)[0].clone();

        // If the class hash is tainted we add it to the report
        if compilation_unit.is_tainted(function_name.to_string(), class_hash) {
            let message = format!(
                "Library call to user controlled class hash in {}\n {}",
                function_name, statement
            );
            results.push(Result {
                name: self.name().to_string(),
                impact: self.impact(),
                confidence: self.confidence(),
                message,
            });
        }
    }
}
