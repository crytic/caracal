use super::detector::{Confidence, Detector, Impact, Result};
use crate::core::compilation_unit::CompilationUnit;
use crate::core::core_unit::CoreUnit;
use cairo_lang_sierra::extensions::core::CoreConcreteLibfunc;
use cairo_lang_sierra::extensions::enm::EnumConcreteLibfunc;
use cairo_lang_sierra::extensions::structure::StructConcreteLibfunc;
use cairo_lang_sierra::extensions::ConcreteType;
use cairo_lang_sierra::program::{GenStatement, Statement as SierraStatement, StatementIdx};

#[derive(Default)]
pub struct UnusedReturn {}

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
        let compilation_unit = core.get_compilation_unit();

        for f in compilation_unit.functions_user_defined() {
            for (i, stmt) in f.get_statements().iter().enumerate() {
                if let SierraStatement::Invocation(invoc) = stmt {
                    // Get the concrete libfunc called
                    let libfunc = compilation_unit
                        .registry()
                        .get_libfunc(&invoc.libfunc_id)
                        .expect("Library function not found in the registry");

                    if let CoreConcreteLibfunc::FunctionCall(_) = libfunc {
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
                                    );
                                }
                            } else if let CoreConcreteLibfunc::Enum(EnumConcreteLibfunc::Match(_)) =
                                libfunc
                            {
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

impl<'a> UnusedReturn {
    fn iterate_struct_deconstruct(
        &self,
        compilation_unit: &'a CompilationUnit,
        results: &mut Vec<Result>,
        mut libfunc: &'a CoreConcreteLibfunc,
        mut stmt_to_check: &[GenStatement<StatementIdx>],
        stmt: &GenStatement<StatementIdx>,
        function_name: &str,
    ) {
        while let CoreConcreteLibfunc::Struct(StructConcreteLibfunc::Deconstruct(_)) = libfunc {
            if let SierraStatement::Invocation(invoc) = &stmt_to_check[0] {
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
