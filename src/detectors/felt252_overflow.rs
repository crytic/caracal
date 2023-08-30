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
use std::collections::HashSet;

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
                let name = f.name();
                // Vector for looking up future instructions
                let statements: Vec<SierraStatement> = f.get_statements().to_owned();
                for (index, stmt) in statements.iter().enumerate() {
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

                                        // Look two instructions in advance, since pattern will always be sub -> store_temp -> is_zero

                                        if let Some(SierraStatement::Invocation(sub_statement)) =
                                            statements.get(index + 2)
                                        {
                                            // Check if felt252_is_zero uses return param of sub instruction

                                            let libfunc_sub = compilation_unit
                                                .registry()
                                                .get_libfunc(&sub_statement.libfunc_id)
                                                .expect(
                                                    "Library function not found in the registry",
                                                );
                                            if let CoreConcreteLibfunc::Felt252(
                                                Felt252Concrete::IsZero(_z),
                                            ) = libfunc_sub
                                            {
                                                let user_params = &sub_statement.args;
                                                if !user_params.contains(ret_value) {
                                                    // This is a geniuine sub instruction since it isn't used by felt252_is_zero
                                                    // Maybe we can just continue here since is_zero is only for checking branches?
                                                    self.check_felt252_tainted(
                                                        &mut results,
                                                        compilation_unit,
                                                        op.param_signatures(),
                                                        stmt,
                                                        invoc.args.clone(),
                                                        &name,
                                                    );
                                                }
                                            } else {
                                                self.check_felt252_tainted(
                                                    &mut results,
                                                    compilation_unit,
                                                    op.param_signatures(),
                                                    stmt,
                                                    invoc.args.clone(),
                                                    &name,
                                                )
                                            }
                                        }
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
                            if let Felt252BinaryOperationConcrete::WithConst(var) = op {
                                match var.operator {
                                    // Do the same as above but for constant case
                                    Felt252BinaryOperator::Sub => {
                                        // Get the return value of the sub statement
                                        let ret_value = &invoc.branches[0].results[0];

                                        // Look two instructions in advance

                                        if let Some(SierraStatement::Invocation(sub_statement)) =
                                            statements.get(index + 2)
                                        {
                                            // Check if felt252_is_zero uses return param of sub instruction

                                            let libfunc_sub = compilation_unit
                                                .registry()
                                                .get_libfunc(&sub_statement.libfunc_id)
                                                .expect(
                                                    "Library function not found in the registry",
                                                );
                                            if let CoreConcreteLibfunc::Felt252(
                                                Felt252Concrete::IsZero(_z),
                                            ) = libfunc_sub
                                            {
                                                let user_params = &sub_statement.args;
                                                if !user_params.contains(ret_value) {
                                                    self.check_felt252_tainted(
                                                        &mut results,
                                                        compilation_unit,
                                                        op.param_signatures(),
                                                        stmt,
                                                        invoc.args.clone(),
                                                        &name,
                                                    );
                                                }
                                            } else {
                                                self.check_felt252_tainted(
                                                    &mut results,
                                                    compilation_unit,
                                                    op.param_signatures(),
                                                    stmt,
                                                    invoc.args.clone(),
                                                    &name,
                                                )
                                            }
                                        }
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
                    "The function {} uses the felt252 operation {} with the user-controlled parameters: {}",
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
