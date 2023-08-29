use std::collections::HashSet;

use super::detector::{Confidence, Detector, Impact, Result};
use crate::analysis::taint::WrapperVariable;
use crate::core::compilation_unit::CompilationUnit;
use crate::core::core_unit::CoreUnit;
use crate::core::function::{Function, Type};
use cairo_lang_sierra::extensions::array::ArrayConcreteLibfunc;
use cairo_lang_sierra::extensions::core::{CoreConcreteLibfunc, CoreTypeConcrete};
use cairo_lang_sierra::program::{GenStatement, Statement as SierraStatement};

#[derive(Default)]
pub struct ArrayUseAfterPopFront {}

impl Detector for ArrayUseAfterPopFront {
    fn name(&self) -> &str {
        "array-use-after-pop-front"
    }

    fn description(&self) -> &str {
        "Detect use of an array after removing element(s)"
    }

    fn confidence(&self) -> Confidence {
        Confidence::Medium
    }

    fn impact(&self) -> Impact {
        Impact::Low
    }

    fn run(&self, core: &CoreUnit) -> Vec<Result> {
        let mut results = Vec::new();
        let compilation_units = core.get_compilation_units();

        for compilation_unit in compilation_units.iter() {
            for function in compilation_unit.functions_user_defined() {
                let pop_fronts: Vec<(usize, WrapperVariable)> = function
                    .get_statements()
                    .iter()
                    .enumerate()
                    .filter_map(|(index, stmt)| match stmt {
                        SierraStatement::Invocation(invoc) => {
                            let libfunc = compilation_unit
                                .registry()
                                .get_libfunc(&invoc.libfunc_id)
                                .expect("Library function not found in the registry");

                            match libfunc {
                                CoreConcreteLibfunc::Array(ArrayConcreteLibfunc::PopFront(_)) => {
                                    Some((
                                        index,
                                        WrapperVariable::new(
                                            function.name(),
                                            invoc.args[0].clone(),
                                        ),
                                    ))
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    })
                    .collect();

                let bad_array_used = pop_fronts.iter().any(|(index, bad_array)| {
                    self.is_array_used_after_pop_front(
                        compilation_unit,
                        function,
                        bad_array,
                        *index,
                    )
                });

                if bad_array_used {
                    let message = format!(
                        "An array is used after removing elements from it in the function {}",
                        &function.name()
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

        results
    }
}

impl ArrayUseAfterPopFront {
    fn is_array_used_after_pop_front(
        &self,
        compilation_unit: &CompilationUnit,
        function: &Function,
        bad_array: &WrapperVariable,
        pop_stmt_index: usize,
    ) -> bool {
        // Check the remaining statements of the function
        let bad_array_used_in_function =
            self.check_statements(compilation_unit, function, bad_array, pop_stmt_index);

        // Check if the bad array is sent to any function being called from this function
        let bad_array_used_in_calls = bad_array_used_in_function
            || self.check_calls(
                compilation_unit,
                function,
                bad_array,
                &mut function.private_functions_calls(),
            )
            || self.check_calls(
                compilation_unit,
                function,
                bad_array,
                &mut function.library_functions_calls(),
            )
            || self.check_calls(
                compilation_unit,
                function,
                bad_array,
                &mut function.external_functions_calls(),
            )
            || self.check_calls(
                compilation_unit,
                function,
                bad_array,
                &mut function.events_emitted(),
            );

        // Check the caller of the current function
        bad_array_used_in_calls || self.check_returns(compilation_unit, function, bad_array)
    }

    // Analyse the statements of the function after the pop_front statement
    // to see if any other element is added to the array.
    fn check_statements(
        &self,
        compilation_unit: &CompilationUnit,
        function: &Function,
        bad_array: &WrapperVariable,
        stmt_index: usize,
    ) -> bool {
        let taint = compilation_unit.get_taint(&function.name()).unwrap();

        // Analyse the statements of the function after the pop_front statement
        // to see if any other element is added to the array.
        let bad_array_used = function
            .get_statements_at(stmt_index)
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
                    CoreConcreteLibfunc::Array(ArrayConcreteLibfunc::Append(_)) => {
                        let mut sinks = HashSet::new();
                        sinks.insert(WrapperVariable::new(function.name(), invoc.args[0].clone()));

                        taint.taints_any_sinks(bad_array, &sinks)
                    }
                    _ => false,
                }
            });

        bad_array_used
    }

    fn check_calls<'a>(
        &self,
        compilation_unit: &CompilationUnit,
        function: &Function,
        bad_array: &WrapperVariable,
        calls: &mut impl Iterator<Item = &'a SierraStatement>,
    ) -> bool {
        let taint = compilation_unit.get_taint(&function.name()).unwrap();

        calls.any(|s| {
            if let GenStatement::Invocation(invoc) = s {
                let lib_func = compilation_unit
                    .registry()
                    .get_libfunc(&invoc.libfunc_id)
                    .expect("Library function not found in the registry");

                if let CoreConcreteLibfunc::FunctionCall(_) = lib_func {
                    let sinks: HashSet<WrapperVariable> = invoc
                        .args
                        .iter()
                        .map(|v| WrapperVariable::new(function.name(), v.clone()))
                        .collect();

                    return taint.taints_any_sinks(bad_array, &sinks);
                }
            }
            false
        })
    }

    // check if the bad array is returned by the function
    // if yes then check if its a loop function
    // if not then its clear usage of a bad array
    // if yes then we need to check its caller to see if it uses the bad array
    fn check_returns(
        &self,
        compilation_unit: &CompilationUnit,
        function: &Function,
        bad_array: &WrapperVariable,
    ) -> bool {
        let taint = compilation_unit.get_taint(&function.name()).unwrap();

        let returns_array = function.returns().any(|r| {
            let return_type = compilation_unit
                .registry()
                .get_type(r)
                .expect("Type not found in the registry");

            if let CoreTypeConcrete::Array(_) = return_type {
                return true;
            }
            false
        });

        let returns_bad_array = returns_array
            && function.get_statements().iter().any(|s| {
                if let GenStatement::Return(return_vars) = s {
                    let sinks: HashSet<WrapperVariable> = return_vars
                        .iter()
                        .map(|v| WrapperVariable::new(function.name(), v.clone()))
                        .collect();

                    return taint.taints_any_sinks(bad_array, &sinks);
                }
                false
            });

        // No need to check remaining statements of the caller function
        // as returning a bad array is already a use of the bad array
        // if the current function is not a loop function
        if returns_bad_array && !matches!(function.ty(), Type::Loop) {
            return true;
        }

        // In case the functon is a loop function, we need to check
        // the remaining statements of the caller function to see if they used the bad array
        compilation_unit.functions_user_defined().any(|maybe_caller| {
            let is_caller = maybe_caller.loop_functions_calls().any(|f| {
                if let GenStatement::Invocation(invoc) = f {
                    let lib_func = compilation_unit
                        .registry()
                        .get_libfunc(&invoc.libfunc_id)
                        .expect("Library function not found in the registry");

                    if let CoreConcreteLibfunc::FunctionCall(f_called) = lib_func {
                        if function.name().as_str() == f_called.function.id.debug_name.as_ref().unwrap() {
                            //invoc.branches[0].results
                            
                        }
                    }
                }
                false
            });

            if is_caller {
                return self.check_statements(compilation_unit, maybe_caller, bad_array, 0);
            }

            false
        })
    }
}
