use super::detector::{Confidence, Detector, Impact, Result};
use crate::core::compilation_unit::CompilationUnit;
use crate::core::core_unit::CoreUnit;
use crate::utils::filter_builtins_from_arguments;
use cairo_lang_sierra::extensions::felt252::Felt252BinaryOperationConcrete;
use cairo_lang_sierra::extensions::felt252::Felt252BinaryOperator;
use cairo_lang_sierra::extensions::lib_func::ParamSignature;
use cairo_lang_sierra::extensions::ConcreteLibfunc;
use cairo_lang_sierra::extensions::{core::CoreConcreteLibfunc, felt252::Felt252Concrete};
use cairo_lang_sierra::ids::VarId;
use cairo_lang_sierra::program::Statement as SierraStatement;
use std::collections::{HashMap, HashSet};

pub struct SubVarInfo<'a> {
    /// The libfunc used
    libfunc: &'a SierraStatement,
    /// The parameters used
    params: &'a [ParamSignature],
    /// Arguments to the sub
    args: &'a Vec<VarId>,
}
#[derive(Default)]
pub struct Felt252Overflow {}

impl Detector for Felt252Overflow {
    fn name(&self) -> &str {
        "felt252-overflow"
    }

    fn description(&self) -> &str {
        "Detect felt252 arithmetic overflow with user-controlled params"
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
            let functions = compilation_unit.functions_user_defined();
            // Iterate through the functions and find binary operations
            for f in functions {
                let mut sub_vars = HashMap::new();
                let name = f.name();
                for stmt in f.get_statements().iter() {
                    if let SierraStatement::Invocation(invoc) = stmt {
                        // Get the concrete libfunc called
                        let libfunc = compilation_unit
                            .registry()
                            .get_libfunc(&invoc.libfunc_id)
                            .expect("Library function not found in the registry");

                        if let CoreConcreteLibfunc::Felt252(Felt252Concrete::BinaryOperation(op)) =
                            libfunc
                        {
                            if let Felt252BinaryOperationConcrete::WithVar(var) = op {
                                match var.operator {
                                    // We need to see if this is a geniune sub or an if/assert
                                    Felt252BinaryOperator::Sub => {
                                        // Get the return value of the sub statement
                                        let ret_value = &invoc.branches[0].results[0];
                                        let sub_struct = SubVarInfo {
                                            libfunc: stmt,
                                            params: op.param_signatures(),
                                            args: &invoc.args,
                                        };
                                        // Add to HashMap to track return values for later
                                        sub_vars.insert(ret_value, sub_struct);
                                        // Continue the loop, we'll analyze this after we check felt252_is_zero
                                        continue;
                                    }
                                    _ => {
                                        self.check_felt252_tainted(
                                            &mut results,
                                            compilation_unit,
                                            op.param_signatures(),
                                            stmt,
                                            invoc.args.clone(),
                                            &name,
                                        );
                                    }
                                }
                            }
                        }
                        // Check if felt252_is_zero uses return param of sub instruction
                        if let CoreConcreteLibfunc::Felt252(Felt252Concrete::IsZero(op)) = libfunc {
                            let user_params = filter_builtins_from_arguments(
                                op.param_signatures(),
                                invoc.args.clone(),
                            );
                            for (k, v) in &sub_vars {
                                if !user_params.contains(k) {
                                    // This is a geniuine sub instruction since it isn't used by felt252_is_zero
                                    self.check_felt252_tainted(
                                        &mut results,
                                        compilation_unit,
                                        v.params,
                                        v.libfunc,
                                        v.args.clone(),
                                        &name,
                                    );
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
impl Felt252Overflow {
    fn check_felt252_tainted(
        &self,
        results: &mut Vec<Result>,
        compilation_unit: &CompilationUnit,
        params: &[ParamSignature],
        libfunc: &SierraStatement,
        args: Vec<VarId>,
        name: &String,
    ) {
        let user_params = filter_builtins_from_arguments(params, args);
        let mut tainted_by: HashSet<&VarId> = HashSet::new();
        let mut taints = String::new();
        for param in user_params.iter() {
            // TODO: improve when we have source mapping,can add parameter's name instead of ID
            if compilation_unit.is_tainted(name.to_string(), param.clone())
                && !tainted_by.contains(&param)
            {
                let msg = format!("{},", &param);
                taints.push_str(&msg);
                tainted_by.insert(param);
            }
        }
        // Get rid of trailing comma from formatting
        taints.pop();
        // Not tainted by any parameter, but still uses felt252 type
        if tainted_by.is_empty() {
            let msg = format!(
                "The function {} uses the felt252 operation {}, which is not overflow safe",
                &name, libfunc
            );
            results.push(Result {
                name: self.name().to_string(),
                impact: self.impact(),
                confidence: self.confidence(),
                message: msg,
            });
        } else {
            let msg = format!(
                    "The function {} uses the felt 252 operation {} with the user-controlled parameters: {}",
                    &name,
                    libfunc,
                    taints
                );
            results.push(Result {
                name: self.name().to_string(),
                impact: self.impact(),
                confidence: self.confidence(),
                message: msg,
            });
        }
    }
}
