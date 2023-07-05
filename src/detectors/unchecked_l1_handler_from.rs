use std::collections::HashSet;

use super::detector::{Confidence, Detector, Impact, Result};
use crate::analysis::taint::WrapperVariable;
use crate::core::compilation_unit::CompilationUnit;
use crate::core::core_unit::CoreUnit;
use crate::core::function::Type;
use cairo_lang_sierra::extensions::{core::CoreConcreteLibfunc, felt252::Felt252Concrete};
use cairo_lang_sierra::ids::VarId;
use cairo_lang_sierra::program::Statement as SierraStatement;

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

    fn run(&self, core: &CoreUnit) -> Vec<Result> {
        let mut results = Vec::new();
        let compilation_units = core.get_compilation_units();

        for compilation_unit in compilation_units {
            let l1_handler_funcs: Vec<_> = compilation_unit
                .functions()
                .filter(|f| *f.ty() == Type::L1Handler)
                .collect();

            for f in l1_handler_funcs {
                let from_address: VarId =
                    f.params().map(|p| p.id.clone()).collect::<Vec<VarId>>()[0].clone();

                // Check if any call to felt252_is_zero uses from_address argument
                let from_checked = f
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
                                    from_address.clone(),
                                    invoc.args.clone(),
                                    compilation_unit,
                                    &f.name(),
                                ),
                            _ => false,
                        }
                    });

                if !from_checked {
                    let message = format!(
                        "The L1 handler function {} does not check the L1 from address",
                        &f.name()
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

impl UncheckedL1HandlerFrom {
    fn is_felt252_is_zero_arg_taintaed_by_from_address(
        &self,
        from_address: VarId,
        felt252_is_zero_args: Vec<VarId>,
        compilation_unit: &CompilationUnit,
        function_name: &str,
    ) -> bool {
        let source = WrapperVariable::new(function_name.to_string(), from_address);

        let sinks: HashSet<WrapperVariable> = felt252_is_zero_args
            .iter()
            .map(|v| WrapperVariable::new(function_name.to_string(), v.clone()))
            .collect();

        let taint = compilation_unit.get_taint(function_name).unwrap();

        // returns true If the felt252_is_zero arguments are tainted by the from_address
        taint.taints_any_sinks(&source, &sinks)
    }
}
