use super::detector::{Confidence, Detector, Impact, Result};
use crate::core::compilation_unit::CompilationUnit;
use crate::core::core_unit::CoreUnit;
use crate::core::function::Type;
use crate::utils::filter_builtins_from_returns;
use cairo_lang_sierra::extensions::core::CoreConcreteLibfunc;
use cairo_lang_sierra::extensions::enm::EnumConcreteLibfunc;
use cairo_lang_sierra::extensions::structure::StructConcreteLibfunc;
use cairo_lang_sierra::extensions::ConcreteType;
use cairo_lang_sierra::program::{GenStatement, Statement as SierraStatement, StatementIdx};

#[derive(Default)]
pub struct UnusedReturn;

impl Detector for UnusedReturn {
    fn name(&self) -> &str {
        "unused-return"
    }

    fn description(&self) -> &str {
        "Detect unused return values"
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
            for f in compilation_unit.functions_user_defined() {
                for (i, stmt) in f.get_statements().iter().enumerate() {
                    if let SierraStatement::Invocation(invoc) = stmt {
                        // Get the concrete libfunc called
                        let libfunc = compilation_unit
                            .registry()
                            .get_libfunc(&invoc.libfunc_id)
                            .expect("Library function not found in the registry");

                        if let CoreConcreteLibfunc::FunctionCall(f_called) = libfunc {
                            // Get the statements after the function call
                            // if it's a drop it means there is an unused argument
                            // if it's a struct_deconstruct we need to look at the next statement until it's different from struct_deconstruct
                            // and check if it's a drop
                            // if it's a enum_match we will have something like that
                            //    function_call<user@unused_result::unused_result::UnusedResult::add_1>([6], [7], [8]) -> ([3], [4], [5]);
                            //    enum_match<core::PanicResult::<(core::felt252,)>>([5]) { fallthrough([9]) 63([10]) };
                            //    branch_align() -> ();
                            //    struct_deconstruct<Tuple<felt252>>([9]) -> ([11]);
                            // followed possibly by others struct_deconstruct and eventually a drop
                            // Note: we should avoid report when a Unit () is dropped

                            if let Some(f) = compilation_unit.functions().find(|f| {
                                f.name() == f_called.function.id.debug_name.clone().unwrap()
                            }) {
                                // We don't check for unused return in case of Storage functions
                                // When a loop function is called in sierra and in that function
                                // an array is emptied with pop_front this array is dropped
                                // when returning from the function call and it would be incorrectly
                                // reported as unused-return
                                if matches!(f.ty(), &Type::Storage | &Type::Loop) {
                                    continue;
                                }
                            } else {
                                // Should never happen
                                println!(
                                    "Unused-return: function not found {}",
                                    f_called.function.id.debug_name.clone().unwrap()
                                );
                                continue;
                            }

                            let following_stmts = f.get_statements_at(i + 1);
                            if let SierraStatement::Invocation(invoc) = &following_stmts[0] {
                                let mut libfunc = compilation_unit
                                    .registry()
                                    .get_libfunc(&invoc.libfunc_id)
                                    .expect("Library function not found in the registry");

                                // Immediate Drop instruction
                                if let CoreConcreteLibfunc::Drop(drop_libfunc) = libfunc {
                                    let ty_dropped = compilation_unit
                                        .registry()
                                        .get_type(&drop_libfunc.signature.param_signatures[0].ty)
                                        .expect("Type not found in registry");
                                    let info = ty_dropped.info();
                                    // If size is 0 it's the Unit type
                                    if info.size != 0 {
                                        results.push(Result {
                                            name: self.name().to_string(),
                                            impact: self.impact(),
                                            confidence: self.confidence(),
                                            message: format!(
                                            "Return value unused for the function call {} in {}",
                                            stmt,
                                            f.name()
                                        ),
                                        });
                                    }
                                } else if let CoreConcreteLibfunc::Struct(
                                    StructConcreteLibfunc::Deconstruct(_),
                                ) = libfunc
                                {
                                    let return_variables = filter_builtins_from_returns(
                                        &f_called.signature.branch_signatures[0].vars,
                                        invoc.branches[0].results.clone(),
                                    )
                                    .len();
                                    // Go to the next statement and update the libfunc
                                    let stmt_to_check = &following_stmts[1..];
                                    if let SierraStatement::Invocation(invoc) = &stmt_to_check[0] {
                                        libfunc = compilation_unit
                                            .registry()
                                            .get_libfunc(&invoc.libfunc_id)
                                            .expect("Library function not found in the registry");

                                        self.iterate_struct_deconstruct(
                                            compilation_unit,
                                            &mut results,
                                            libfunc,
                                            stmt_to_check,
                                            stmt,
                                            &f.name(),
                                            return_variables,
                                        );
                                    }
                                } else if let CoreConcreteLibfunc::Enum(
                                    EnumConcreteLibfunc::Match(_),
                                ) = libfunc
                                {
                                    let return_variables = filter_builtins_from_returns(
                                        &f_called.signature.branch_signatures[0].vars,
                                        invoc.branches[0].results.clone(),
                                    )
                                    .len();
                                    // Jump one statement which is a branch_align and the next one will be a struct_deconstruct
                                    let stmt_to_check = &following_stmts[2..];
                                    if let SierraStatement::Invocation(invoc) = &stmt_to_check[0] {
                                        libfunc = compilation_unit
                                            .registry()
                                            .get_libfunc(&invoc.libfunc_id)
                                            .expect("Library function not found in the registry");

                                        self.iterate_struct_deconstruct(
                                            compilation_unit,
                                            &mut results,
                                            libfunc,
                                            stmt_to_check,
                                            stmt,
                                            &f.name(),
                                            return_variables,
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

impl<'a> UnusedReturn {
    #[allow(clippy::too_many_arguments)]
    fn iterate_struct_deconstruct(
        &self,
        compilation_unit: &'a CompilationUnit,
        results: &mut Vec<Result>,
        mut libfunc: &'a CoreConcreteLibfunc,
        mut stmt_to_check: &[GenStatement<StatementIdx>],
        stmt: &GenStatement<StatementIdx>,
        function_name: &str,
        return_variables: usize,
    ) {
        let mut return_variables_counter = 0;
        while let CoreConcreteLibfunc::Struct(StructConcreteLibfunc::Deconstruct(_)) = libfunc {
            if let SierraStatement::Invocation(invoc) = &stmt_to_check[0] {
                // If there are other struct deconstruction are not related to the returned variables
                if return_variables_counter == return_variables {
                    break;
                }

                return_variables_counter += 1;
                libfunc = compilation_unit
                    .registry()
                    .get_libfunc(&invoc.libfunc_id)
                    .expect("Library function not found in the registry");

                stmt_to_check = &stmt_to_check[1..];
            } else {
                break;
            }
        }

        // If the instruction after all the struct_deconstruct is a drop report unused return value
        if let CoreConcreteLibfunc::Drop(drop_libfunc) = libfunc {
            let ty_dropped = compilation_unit
                .registry()
                .get_type(&drop_libfunc.signature.param_signatures[0].ty)
                .expect("Type not found in registry");
            let info = ty_dropped.info();
            // If size is 0 it's the Unit type
            if info.size != 0 {
                results.push(Result {
                    name: self.name().to_string(),
                    impact: self.impact(),
                    confidence: self.confidence(),
                    message: format!(
                        "Return value unused for the function call {} in {}",
                        stmt, function_name
                    ),
                });
            }
        }
    }
}
