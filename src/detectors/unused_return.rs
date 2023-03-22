use std::collections::HashSet;

use super::detector::{Confidence, Detector, Impact, Result};
use crate::core::core_unit::CoreUnit;
use cairo_lang_sierra::extensions::core::CoreConcreteLibfunc;
use cairo_lang_sierra::program::Statement as SierraStatement;

// TODO at the moment this only handle simple type (e.g. a single felt) but if the function returns a struct or a tuple it doesn't work

pub struct UnusedReturn {
    name: String,
    impact: Impact,
    confidence: Confidence,
}

impl UnusedReturn {
    pub fn new(name: String, impact: Impact, confidence: Confidence) -> Self {
        UnusedReturn {
            name,
            impact,
            confidence,
        }
    }
}

impl Detector for UnusedReturn {
    fn name(&self) -> &str {
        &self.name
    }

    fn confidence(&self) -> &Confidence {
        &self.confidence
    }

    fn impact(&self) -> &Impact {
        &self.impact
    }

    fn run(&self, core: &CoreUnit) -> Vec<Result> {
        let mut results = Vec::new();
        for f in core.get_compilation_unit().functions_user_defined() {
            for (i, s) in f.get_statements().iter().enumerate() {
                if let SierraStatement::Invocation(invoc) = s {
                    let function = core
                        .get_compilation_unit()
                        .registry()
                        .get_libfunc(&invoc.libfunc_id)
                        .expect("Library function not found in the registry");
                    if let CoreConcreteLibfunc::FunctionCall(f_called) = function {
                        let mut return_ids = HashSet::new();
                        for r in f_called.function.signature.ret_types.iter() {
                            return_ids.insert(r.id);
                        }
                        for next_statement in f.get_statements_at(i + 1) {
                            if let SierraStatement::Invocation(invoc) = next_statement {
                                let function = core
                                    .get_compilation_unit()
                                    .registry()
                                    .get_libfunc(&invoc.libfunc_id)
                                    .expect("Library function not found in the registry");
                                if let CoreConcreteLibfunc::Drop(id) = function {
                                    for param in id.signature.param_signatures.iter() {
                                        if return_ids.contains(&param.ty.id) {
                                            // Unused value
                                            let message = format!("Return value unused for the function call {} in {}", s, f.name());
                                            results.push(Result {
                                                name: self.name().to_string(),
                                                impact: self.impact,
                                                confidence: self.confidence,
                                                message,
                                            })
                                        }
                                    }
                                    break; // Not a Drop statement we exit
                                }
                                break; // Return statement we exit
                            }
                        }
                    }
                }
            }
        }

        results
    }
}
