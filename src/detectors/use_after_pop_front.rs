use std::collections::HashSet;

use super::detector::{Confidence, Detector, Impact, Result};
use crate::analysis::taint::WrapperVariable;
use crate::core::compilation_unit::CompilationUnit;
use crate::core::core_unit::CoreUnit;
use crate::core::function::{Function, Type};
use cairo_lang_sierra::extensions::array::ArrayConcreteLibfunc;
use cairo_lang_sierra::extensions::core::{CoreConcreteLibfunc, CoreTypeConcrete};
use cairo_lang_sierra::program::{GenStatement, Statement as SierraStatement};
use fxhash::FxHashSet;

#[derive(Default)]
pub struct UseAfterPopFront {}

enum CollectionType {
    Array,
    Span,
}

impl CollectionType {
    fn is_array(&self) -> bool {
        match self {
            CollectionType::Array => true,
            CollectionType::Span => false,
        }
    }
}

impl Detector for UseAfterPopFront {
    fn name(&self) -> &str {
        "use-after-pop-front"
    }

    fn description(&self) -> &str {
        "Detect use of an array or a span after removing element(s)"
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

        for compilation_unit in compilation_units.iter() {
            for function in compilation_unit.functions_user_defined() {
                let pop_fronts: Vec<(usize, WrapperVariable, CollectionType)> = function
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
                                        WrapperVariable::new(function.name(), invoc.args[0].id),
                                        CollectionType::Array,
                                    ))
                                }
                                CoreConcreteLibfunc::Array(
                                    ArrayConcreteLibfunc::SnapshotPopFront(_),
                                ) => Some((
                                    index,
                                    WrapperVariable::new(function.name(), invoc.args[0].id),
                                    CollectionType::Span,
                                )),
                                CoreConcreteLibfunc::Array(
                                    ArrayConcreteLibfunc::SnapshotPopBack(_),
                                ) => Some((
                                    index,
                                    WrapperVariable::new(function.name(), invoc.args[0].id),
                                    CollectionType::Span,
                                )),
                                _ => None,
                            }
                        }
                        _ => None,
                    })
                    .collect();

                // Required to silence clippy too-complex-type warning
                type BadCollectionType<'a, 'b> = Vec<(&'a WrapperVariable, &'b CollectionType)>;

                let (bad_array_used, bad_span_used): (BadCollectionType, BadCollectionType) =
                    pop_fronts
                        .iter()
                        .filter_map(|(index, bad_array, collection_type)| {
                            let is_used = self.is_used_after_pop_front(
                                compilation_unit,
                                function,
                                bad_array,
                                *index,
                            );
                            if is_used {
                                Some((bad_array, collection_type))
                            } else {
                                None
                            }
                        })
                        .partition(|(_, collection_type)| collection_type.is_array());

                if !bad_array_used.is_empty() {
                    let array_ids = bad_array_used
                        .iter()
                        .map(|f| f.0.variable())
                        .collect::<Vec<u64>>();
                    let message = match array_ids.len() {
                        1 => format!(
                            "The array {:?} is used after removing elements from it in the function {}",
                            array_ids,
                            &function.name()
                        ),
                        _ => format!(
                            "The arrays {:?} are used after removing elements from them in the function {}",
                            array_ids,
                            &function.name()
                        )
                    };
                    results.insert(Result {
                        name: self.name().to_string(),
                        impact: self.impact(),
                        confidence: self.confidence(),
                        message,
                    });
                }

                if !bad_span_used.is_empty() {
                    let span_ids = bad_span_used
                        .iter()
                        .map(|f| f.0.variable())
                        .collect::<Vec<u64>>();
                    let message = match span_ids.len() {
                        1 => format!(
                            "The span {:?} is used after removing elements from it in the function {}",
                            span_ids,
                            &function.name()
                        ),
                        _ => format!(
                            "The spans {:?} are used after removing elements from them in the function {}",
                            span_ids,
                            &function.name()
                        )
                    };
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

impl UseAfterPopFront {
    fn is_used_after_pop_front(
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
                &mut function
                    .private_functions_calls()
                    .chain(function.library_functions_calls())
                    .chain(function.external_functions_calls())
                    .chain(function.events_emitted()),
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
                        let mut sinks = FxHashSet::default();
                        sinks.insert(WrapperVariable::new(function.name(), invoc.args[0].id));

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
                    let sinks: FxHashSet<WrapperVariable> = invoc
                        .args
                        .iter()
                        .map(|v| WrapperVariable::new(function.name(), v.id))
                        .collect();

                    return taint.taints_any_sinks(bad_array, &sinks);
                }
            }
            false
        })
    }

    // check if the bad array is returned by the function
    // if yes then check if its a loop function
    // if yes then we need to check its caller to see if it uses the bad array
    // if not then its clear usage of a bad array
    fn check_returns(
        &self,
        compilation_unit: &CompilationUnit,
        function: &Function,
        bad_array: &WrapperVariable,
    ) -> bool {
        // No need to check remaining statements of the caller function
        // as returning a bad array is already a use of the bad array
        // if the current function is not a loop function
        match function.ty() {
            Type::Loop => self.check_loop_returns(compilation_unit, function, bad_array),
            _ => self.check_non_loop_returns(compilation_unit, function, bad_array),
        }
    }

    fn check_loop_returns(
        &self,
        compilation_unit: &CompilationUnit,
        function: &Function,
        bad_array: &WrapperVariable,
    ) -> bool {
        // We can not find the array from the return types of the function
        // We assume that the array is returned at the same index as it was taken on
        let return_array_indices: Vec<usize> = function
            .params_all()
            .enumerate()
            .filter_map(|(i, param)| {
                let param_type = compilation_unit
                    .registry()
                    .get_type(&param.ty)
                    .expect("Type not found in the registry");

                match param_type {
                    CoreTypeConcrete::Array(_) => Some(i),
                    span if self.is_core_type_concrete_span(compilation_unit, span) => Some(i),
                    _ => None,
                }
            })
            .collect();

        // In case the functon is a loop function, we need to check
        // the remaining statements of the caller function to see if they used the bad array
        compilation_unit
            .functions_user_defined()
            .any(|maybe_caller| {
                maybe_caller
                    .loop_functions_calls()
                    .flat_map(|f| {
                        if let GenStatement::Invocation(invoc) = f {
                            let lib_func = compilation_unit
                                .registry()
                                .get_libfunc(&invoc.libfunc_id)
                                .expect("Library function not found in the registry");

                            if let CoreConcreteLibfunc::FunctionCall(f_called) = lib_func {
                                if function.name().as_str()
                                    == f_called.function.id.debug_name.as_ref().unwrap()
                                {
                                    return return_array_indices
                                        .iter()
                                        .map(|i| {
                                            WrapperVariable::new(
                                                maybe_caller.name(),
                                                invoc.branches[0].results[*i].id,
                                            )
                                        })
                                        .collect();
                                }
                            }
                        }
                        Vec::new()
                    })
                    .any(|caller_bad_array| {
                        self.check_statements(compilation_unit, maybe_caller, &caller_bad_array, 0)
                            || self.check_calls(
                                compilation_unit,
                                function,
                                bad_array,
                                &mut maybe_caller
                                    .private_functions_calls()
                                    .chain(maybe_caller.library_functions_calls())
                                    .chain(maybe_caller.external_functions_calls())
                                    .chain(maybe_caller.events_emitted()),
                            )
                    })
            })
    }

    fn check_non_loop_returns(
        &self,
        compilation_unit: &CompilationUnit,
        function: &Function,
        bad_array: &WrapperVariable,
    ) -> bool {
        let taint = compilation_unit.get_taint(&function.name()).unwrap();

        let return_array_indices: Vec<usize> = function
            .returns_all()
            .enumerate()
            .flat_map(|(i, r)| {
                let return_type = compilation_unit
                    .registry()
                    .get_type(r)
                    .expect("Type not found in the registry");

                match return_type {
                    CoreTypeConcrete::Array(_) => Some(i),
                    span if self.is_core_type_concrete_span(compilation_unit, span) => Some(i),
                    _ => None,
                }
            })
            .collect();

        // Not returning any array
        if return_array_indices.is_empty() {
            return false;
        }

        // It is not required because taint analysis adds all the arugments as
        // tainters of the all the return values. Added it in case the taint
        // analysis is improved later on to be more granular.
        let returned_bad_arrays: Vec<WrapperVariable> = function
            .get_statements()
            .iter()
            .flat_map(|s| {
                if let GenStatement::Return(return_vars) = s {
                    let sinks: FxHashSet<WrapperVariable> = return_vars
                        .iter()
                        .map(|v| WrapperVariable::new(function.name(), v.id))
                        .collect();

                    return taint.taints_any_sinks_variable(bad_array, &sinks);
                }
                Vec::new()
            })
            .collect();

        !returned_bad_arrays.is_empty()
    }

    // The Span is not a Core Sierra type, it is defined in the corelib as a Struct
    // Therefore we can not match it against CoreTypeConcrete::Span directly
    fn is_core_type_concrete_span(
        &self,
        compilation_unit: &CompilationUnit,
        maybe_span: &CoreTypeConcrete,
    ) -> bool {
        match maybe_span {
            CoreTypeConcrete::Struct(struct_type) => match &struct_type.members[..] {
                [maybe_snapshot, ..] => {
                    let maybe_snapshot_type = compilation_unit
                        .registry()
                        .get_type(maybe_snapshot)
                        .expect("Type not found in the registry");

                    match maybe_snapshot_type {
                        CoreTypeConcrete::Snapshot(maybe_array) => {
                            let maybe_array_type = compilation_unit
                                .registry()
                                .get_type(&maybe_array.ty)
                                .expect("Type not found in the registry");

                            matches!(maybe_array_type, CoreTypeConcrete::Array(_))
                        }
                        _ => false,
                    }
                }
                _ => false,
            },
            _ => false,
        }
    }
}
