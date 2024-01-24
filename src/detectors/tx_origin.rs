use super::detector::{Confidence, Detector, Impact, Result};
use crate::analysis::taint::WrapperVariable;
use crate::core::compilation_unit::CompilationUnit;
use crate::core::core_unit::CoreUnit;
use crate::core::function::Function;
use cairo_lang_sierra::extensions::core::CoreConcreteLibfunc;
use cairo_lang_sierra::extensions::felt252::Felt252Concrete;
use cairo_lang_sierra::extensions::structure::StructConcreteLibfunc;
use cairo_lang_sierra::ids::VarId;
use cairo_lang_sierra::program::{GenStatement, Statement as SierraStatement};
use fxhash::FxHashSet;
use std::collections::HashSet;

#[derive(Default)]
pub struct TxOrigin {}

impl Detector for TxOrigin {
    fn name(&self) -> &str {
        "tx-origin"
    }

    fn description(&self) -> &str {
        "Detect usage of the transaction origin address as access control"
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

        for compilation_unit in compilation_units.iter() {
            for function in compilation_unit.functions_user_defined() {
                let tx_origins: FxHashSet<WrapperVariable> = function
                    .get_statements()
                    .iter()
                    .filter_map(|stmt| match stmt {
                        SierraStatement::Invocation(invoc) => {
                            let libfunc = compilation_unit
                                .registry()
                                .get_libfunc(&invoc.libfunc_id)
                                .expect("Library function not found in the registry");

                            match libfunc {
                                CoreConcreteLibfunc::Struct(
                                    StructConcreteLibfunc::Deconstruct(struct_type),
                                ) => {
                                    let struct_params: Vec<String> = struct_type
                                        .signature
                                        .param_signatures
                                        .iter()
                                        .map(|s| s.ty.to_string())
                                        .collect();
                                    match &struct_params[..] {
                                        [maybe_tx_info, ..]
                                            if [
                                                "core::starknet::info::TxInfo",
                                                "core::starknet::info::v2::TxInfo",
                                            ]
                                            .contains(&maybe_tx_info.as_str()) =>
                                        {
                                            Some(WrapperVariable::new(
                                                function.name(),
                                                invoc.branches[0].results[1].id,
                                            ))
                                        }
                                        _ => None,
                                    }
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    })
                    .collect();

                let mut checked_private_functions = HashSet::new();
                let tx_origin_used = self.is_tx_origin_used_in_conditionals(
                    compilation_unit,
                    function,
                    &tx_origins,
                    &mut checked_private_functions,
                );

                if tx_origin_used {
                    let message = format!(
                        "The transaction origin contract address is used in an access control check in the function {}",
                        &function.name()
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

impl TxOrigin {
    fn is_tx_origin_used_in_conditionals(
        &self,
        compilation_unit: &CompilationUnit,
        function: &Function,
        tx_origin_tainted_args: &FxHashSet<WrapperVariable>,
        checked_private_functions: &mut HashSet<String>,
    ) -> bool {
        let tx_origin_checked = function
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
                        .is_felt252_is_zero_arg_taintaed_by_tx_origin(
                            compilation_unit,
                            tx_origin_tainted_args,
                            invoc.args.clone(),
                            &function.name(),
                        ),
                    _ => false,
                }
            });

        let tx_origin_checked_in_private_functions = tx_origin_checked
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

                        let sinks: FxHashSet<WrapperVariable> = invoc
                            .args
                            .iter()
                            .map(|v| WrapperVariable::new(function.name(), v.id))
                            .collect();

                        let tx_origin_tainted_args: FxHashSet<WrapperVariable> =
                            tx_origin_tainted_args
                                .iter()
                                .flat_map(|source| taint.taints_any_sinks_variable(source, &sinks))
                                .map(|sink| {
                                    let mut var_id = None;
                                    for (i, var) in invoc.args.iter().enumerate() {
                                        if var.id == sink.variable() {
                                            var_id = Some(i);
                                            break;
                                        }
                                    }
                                    if let Some(id) = var_id {
                                        WrapperVariable::new(
                                            private_function.name(),
                                            id.try_into().unwrap(),
                                        )
                                    } else {
                                        println!(
                                            "tx_origin: Did not find sink id, id could be wrong."
                                        );
                                        // This is very likely to use a wrong var id
                                        // but we should never enter this branch
                                        WrapperVariable::new(
                                            private_function.name(),
                                            sink.variable() - invoc.args[0].id,
                                        )
                                    }
                                })
                                .collect();

                        checked_private_functions.insert(private_function.name());
                        return self.is_tx_origin_used_in_conditionals(
                            compilation_unit,
                            private_function,
                            &tx_origin_tainted_args,
                            checked_private_functions,
                        );
                    }
                }
                false
            });

        tx_origin_checked_in_private_functions
    }

    fn is_felt252_is_zero_arg_taintaed_by_tx_origin(
        &self,
        compilation_unit: &CompilationUnit,
        tx_origin_tainted_args: &FxHashSet<WrapperVariable>,
        felt252_is_zero_args: Vec<VarId>,
        function_name: &str,
    ) -> bool {
        let taint = compilation_unit.get_taint(function_name).unwrap();
        let sink = WrapperVariable::new(function_name.to_string(), felt252_is_zero_args[0].id);
        taint.taints_any_sources(tx_origin_tainted_args, &sink)
    }
}
