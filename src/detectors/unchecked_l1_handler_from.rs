use std::collections::HashSet;

use super::detector::{Confidence, Detector, Impact, Result};
use crate::analysis::taint::WrapperVariable;
use crate::core::compilation_unit::CompilationUnit;
use crate::core::core_unit::CoreUnit;
use crate::core::function::{Function, Type};
use cairo_lang_sierra::extensions::{core::CoreConcreteLibfunc, felt252::Felt252Concrete};
use cairo_lang_sierra::ids::VarId;
use cairo_lang_sierra::program::{GenStatement, Statement as SierraStatement};

#[derive(Default)]
pub struct UncheckedL1HandlerFrom {}

impl Detector for UncheckedL1HandlerFrom {
    fn name(&self) -> &str {
        "unchecked-l1-handler-from"
    }

    fn description(&self) -> &str {
        "Detect L1 handlers without from address check"
    }

    fn confidence(&self) -> Confidence {
        Confidence::Medium
    }

    fn impact(&self) -> Impact {
        Impact::High
    }

    fn run(&self, core: &CoreUnit) -> HashSet<Result> {
        let mut results: HashSet<Result> = HashSet::new();
        let compilation_units = core.get_compilation_units();

        for compilation_unit in compilation_units {
            let l1_handler_funcs: Vec<_> = compilation_unit
                .functions()
                .filter(|f| *f.ty() == Type::L1Handler)
                .collect();

            for f in l1_handler_funcs {
                let from_address =
                    f.params().map(|p| p.id.clone()).collect::<Vec<VarId>>()[1].clone();
                let mut sources = HashSet::new();
                sources.insert(WrapperVariable::new(f.name(), from_address));

                // Used to avoid infinite recursion in case of recursive private function calls
                let mut checked_private_functions = HashSet::new();

                // Check if any call to felt252_is_zero uses from_address argument
                let from_checked = self.is_from_checked_in_function(
                    &sources,
                    compilation_unit,
                    f,
                    &mut checked_private_functions,
                );

                if !from_checked {
                    let message = format!(
                        "The L1 handler function {} does not check the L1 from address",
                        &f.name()
                    );
                    results.insert(Result {
                        name: self.name().to_string(),
                        impact: self.impact(),
                        confidence: self.confidence(),
                        message,
                    });
                }
            }
        }

        results
    }
}

impl UncheckedL1HandlerFrom {
    fn is_from_checked_in_function(
        &self,
        from_tainted_args: &HashSet<WrapperVariable>,
        compilation_unit: &CompilationUnit,
        function: &Function,
        checked_private_functions: &mut HashSet<String>,
    ) -> bool {
        let from_checked = function
            .get_statements()
            .iter()
            .filter_map(|stmt| match stmt {
                SierraStatement::Invocation(invoc) => Some(invoc),
                _ => None,
            })
            .any(|invoc| {
                let libfunc = compilation_unit
                    .registry()
                    .get_libfunc(&invoc.libfunc_id)
                    .expect("Library function not found in the registry");

                match libfunc {
                    CoreConcreteLibfunc::Felt252(Felt252Concrete::IsZero(_)) => self
                        .is_felt252_is_zero_arg_taintaed_by_from_address(
                            from_tainted_args,
                            invoc.args.clone(),
                            compilation_unit,
                            &function.name(),
                        ),
                    _ => false,
                }
            });

        let from_checked_in_private_functions = from_checked
            || function.private_functions_calls().any(|s| {
                if let GenStatement::Invocation(invoc) = s {
                    let lib_func = compilation_unit
                        .registry()
                        .get_libfunc(&invoc.libfunc_id)
                        .expect("Library function not found in the registry");

                    if let CoreConcreteLibfunc::FunctionCall(f_called) = lib_func {
                        let private_function = compilation_unit
                            .function_by_name(f_called.function.id.debug_name.as_ref().unwrap())
                            .unwrap();
                        if checked_private_functions.contains(&private_function.name()) {
                            return false;
                        }

                        let taint = compilation_unit.get_taint(&function.name()).unwrap();

                        let sinks: HashSet<WrapperVariable> = invoc
                            .args
                            .iter()
                            .map(|v| WrapperVariable::new(function.name(), v.clone()))
                            .collect();

                        let from_tainted_args: HashSet<WrapperVariable> = from_tainted_args
                            .iter()
                            .flat_map(|source| taint.taints_any_sinks_variable(source, &sinks))
                            .map(|sink| {
                                WrapperVariable::new(
                                    private_function.name(),
                                    VarId::new(sink.variable().id - invoc.args[0].id),
                                )
                            })
                            .collect();

                        checked_private_functions.insert(private_function.name());
                        return self.is_from_checked_in_function(
                            &from_tainted_args,
                            compilation_unit,
                            private_function,
                            checked_private_functions,
                        );
                    }
                }
                false
            });

        from_checked_in_private_functions
    }

    fn is_felt252_is_zero_arg_taintaed_by_from_address(
        &self,
        sources: &HashSet<WrapperVariable>,
        felt252_is_zero_args: Vec<VarId>,
        compilation_unit: &CompilationUnit,
        function_name: &str,
    ) -> bool {
        let sink = WrapperVariable::new(function_name.to_string(), felt252_is_zero_args[0].clone());
        let taint = compilation_unit.get_taint(function_name).unwrap();
        // returns true If the felt252_is_zero arguments are tainted by the from_address
        taint.taints_any_sources(sources, &sink)
    }
}
