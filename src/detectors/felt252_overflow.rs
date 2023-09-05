use super::detector::{Confidence, Detector, Impact, Result};
use crate::core::compilation_unit::CompilationUnit;
use crate::core::core_unit::CoreUnit;
use cairo_lang_sierra::extensions::felt252::Felt252BinaryOperationConcrete;
use cairo_lang_sierra::extensions::felt252::Felt252BinaryOperator;
use cairo_lang_sierra::extensions::{core::CoreConcreteLibfunc, felt252::Felt252Concrete};
use cairo_lang_sierra::ids::VarId;
use cairo_lang_sierra::program::GenInvocation;
use cairo_lang_sierra::program::Statement as SierraStatement;
use cairo_lang_sierra::program::StatementIdx;
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
                let statements: &Vec<SierraStatement> = f.get_statements();
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
                            match op {
                                Felt252BinaryOperationConcrete::WithConst(var) => {
                                    let operation = var.operator;

                                    self.handle_binops(
                                        &mut results,
                                        compilation_unit,
                                        invoc,
                                        statements,
                                        index,
                                        stmt,
                                        &operation,
                                        &name,
                                    )
                                }
                                Felt252BinaryOperationConcrete::WithVar(var) => {
                                    let operation = var.operator;

                                    self.handle_binops(
                                        &mut results,
                                        compilation_unit,
                                        invoc,
                                        statements,
                                        index,
                                        stmt,
                                        &operation,
                                        &name,
                                    )
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
        libfunc: &SierraStatement,
        args: &[VarId],
        name: &String,
    ) {
        let mut tainted_by: HashSet<&VarId> = HashSet::new();
        let mut taints = String::new();
        for param in args.iter() {
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
                    "The function {} uses the felt252 operation {} with the user-controlled parameters: {}, which is not overflow safe",
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
    #[allow(clippy::too_many_arguments)]
    fn handle_binops(
        &self,
        results: &mut Vec<Result>,
        compilation_unit: &CompilationUnit,
        invoc: &GenInvocation<StatementIdx>,
        statements: &[SierraStatement],
        idx: usize,
        libfunc: &SierraStatement,
        operation: &Felt252BinaryOperator,
        name: &String,
    ) {
        // Get the return value of the sub statement
        match operation {
            Felt252BinaryOperator::Sub => {
                let ret_value = &invoc.branches[0].results[0];

                // Look two instructions in advance

                if let Some(SierraStatement::Invocation(sub_statement)) = statements.get(idx + 2) {
                    // Check if felt252_is_zero uses return param of sub instruction

                    let libfunc_sub = compilation_unit
                        .registry()
                        .get_libfunc(&sub_statement.libfunc_id)
                        .expect("Library function not found in the registry");
                    if let CoreConcreteLibfunc::Felt252(Felt252Concrete::IsZero(_)) = libfunc_sub {
                        let user_params = &sub_statement.args;
                        if !user_params.contains(ret_value) {
                            self.check_felt252_tainted(
                                results,
                                compilation_unit,
                                libfunc,
                                &invoc.args,
                                name,
                            );
                        }
                    } else {
                        self.check_felt252_tainted(
                            results,
                            compilation_unit,
                            libfunc,
                            &invoc.args,
                            name,
                        )
                    }
                }
            }
            _ => {
                self.check_felt252_tainted(results, compilation_unit, libfunc, &invoc.args, name);
            }
        }
    }
}
