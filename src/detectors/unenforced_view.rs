use super::detector::{Confidence, Detector, Impact, Result};
use crate::core::compilation_unit::CompilationUnit;
use crate::core::core_unit::CoreUnit;
use crate::core::function::Type;
use cairo_lang_sierra::extensions::core::CoreConcreteLibfunc;
use cairo_lang_sierra::program::Statement as SierraStatement;

#[derive(Default)]
pub struct UnenforcedView {}

impl Detector for UnenforcedView {
    fn name(&self) -> &str {
        "unenforced-view"
    }
    fn description(&self) -> &str {
        "function has view decorator but modifies state"
    }
    fn confidence(&self) -> Confidence {
        Confidence::Medium
    }
    fn impact(&self) -> Impact {
        Impact::Medium
    }
    fn run(&self, core: &CoreUnit) -> Vec<Result> {
        let mut results = Vec::new();
        let compilation_units = core.get_compilation_units();

        for compilation_unit in compilation_units {
            let view_funcs: Vec<_> = compilation_unit
                .functions()
                .filter(|f| *f.ty() == Type::View)
                .collect();

            for func in view_funcs {
                let func_name = func.name();
                let (declaration, name) = func_name.rsplit_once("::").unwrap();
                if func.storage_vars_written().count() > 0 || func.events_emitted().count() > 0 {
                    results.push(Result {
                        name: self.name().to_string(),
                        impact: self.impact(),
                        confidence: self.confidence(),
                        message: format!(
                            "{} defined in {} is declared as view but changes state",
                            name, declaration
                        ),
                    });
                }
                let subcalls = func.private_functions_calls().collect();
                self.check_view_subcalls(
                    compilation_unit,
                    declaration,
                    name,
                    &mut results,
                    subcalls,
                )
            }
        }
        results
    }
}
impl UnenforcedView {
    fn check_view_subcalls(
        &self,
        compilation_unit: &CompilationUnit,
        declaration: &str,
        name: &str,
        results: &mut Vec<Result>,
        subcalls: Vec<&SierraStatement>,
    ) {
        if subcalls.is_empty() {
            return;
        }

        for call in subcalls {
            // do lookup
            if let SierraStatement::Invocation(invoc) = call {
                let libfunc = compilation_unit
                    .registry()
                    .get_libfunc(&invoc.libfunc_id)
                    .expect("Library not found in core registry");
                if let CoreConcreteLibfunc::FunctionCall(f_called) = libfunc {
                    let func_name = &f_called
                        .function
                        .id
                        .debug_name
                        .as_ref()
                        .unwrap()
                        .to_string();
                    let called_fn = compilation_unit
                        .functions_user_defined()
                        .find(|f| &f.name() == func_name)
                        .unwrap();
                    // if lookup writes to storage push result using original view function + declaration
                    if called_fn.storage_vars_written().count() > 0
                        || called_fn.events_emitted().count() > 0
                    {
                        results.push(Result {
                            name: self.name().to_string(),
                            impact: self.impact(),
                            confidence: self.confidence(),
                            message: format!(
                                "{} defined in {} is declared as view but changes state",
                                name, declaration
                            ),
                        });
                    }
                    // for now we just check over the private calls, even though the compiler doesn't validate that an external call can be called from within the contract
                    let subcalls_to_check = called_fn.private_functions_calls().collect();
                    self.check_view_subcalls(
                        compilation_unit,
                        declaration,
                        name,
                        results,
                        subcalls_to_check,
                    );
                }
            }
        }
    }
}
