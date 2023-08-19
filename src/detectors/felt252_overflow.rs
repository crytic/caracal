use std::collections::HashSet;

use super::detector::{Confidence, Detector, Impact, Result};
use crate::analysis::taint::WrapperVariable;
use crate::core::compilation_unit::CompilationUnit;
use crate::utils::filter_builtins_from_arguments;
use cairo_lang_sierra::extensions::felt252::Felt252Libfunc::BinaryOperation;
use cairo_lang_sierra::extensions::lib_func::ParamSignature;
use crate::core::core_unit::CoreUnit;
use crate::core::function::{Function, Type};
use cairo_lang_sierra::extensions::{core::CoreConcreteLibfunc, felt252::Felt252Concrete};
use cairo_lang_sierra::ids::VarId;
use cairo_lang_sierra::extensions::ConcreteLibfunc;
use cairo_lang_sierra::program::{GenStatement, Statement as SierraStatement};

#[derive(Default)]
pub struct Felt252Overflow {}

impl Detector for Felt252Overflow {
    fn name(&self) -> &str {
        "felt252-overflow"
    }

    fn description(&self) -> &str {
        "Detect arithmetic overflow with felt252"
    }

    fn confidence(&self) -> Confidence {
        Confidence::Medium
    }

    fn impact(&self) -> Impact {
        Impact::Medium
    }

    fn run(&self, core: &CoreUnit) -> Vec<Result> {
        println!("test");
        let mut results = Vec::new();
        let compilation_units = core.get_compilation_units();
        for compilation_unit in compilation_units {
            let functions = compilation_unit.functions_user_defined();
            for f in functions {
                for stmt in f.get_statements().iter() {
                    if let SierraStatement::Invocation(invoc) = stmt {
                        // Get the concrete libfunc called
                        let libfunc = compilation_unit
                            .registry()
                            .get_libfunc(&invoc.libfunc_id)
                            .expect("Library function not found in the registry");

                        if let CoreConcreteLibfunc::Felt252(Felt252Concrete::BinaryOperation(op)) = libfunc {
                            self.check_felt252_tainted(&mut results, &compilation_unit,op.param_signatures(),invoc.args.clone(),&f.name());

                        }


                    }
                }
            }
        }
        return results;
    }
}
impl Felt252Overflow {
    fn check_felt252_tainted(&self,results:&mut Vec<Result>, compilation_unit: &CompilationUnit, params:&[ParamSignature],args:Vec<VarId>,name:&String) {
        let user_params = filter_builtins_from_arguments(params, args);
        //do the taint
        for param in user_params {
            if compilation_unit.is_tainted(name.to_string(), param) {
                let msg = format!("Function {} uses the felt252 type which is not overflow safe",&name);
                            results.push(Result {
                                name: self.name().to_string(),
                                impact: self.impact(),
                                confidence: self.confidence(),
                                message: msg,
                            });
            }
        }
    }
}
