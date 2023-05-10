use super::detector::{Confidence, Detector, Impact, Result};
use crate::core::core_unit::CoreUnit;
use crate::core::function::Type;
use std::collections::{HashMap,HashSet};
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
    fn run(&self, core:&CoreUnit) -> Vec<Result> {
        let mut results = Vec::new();
        let compilation_unit = core.get_compilation_unit();
        //let mut storage_vars  = HashMap::new();
        let unenforced_view_funcs: HashSet<String> = compilation_unit.functions().filter(|f| *f.ty() == Type::View && !f.storage_vars_written().peekable().peek().is_none()).map(|f| f.name()).collect();

        unenforced_view_funcs.iter().map(|f| f.rsplit_once("::").unwrap()).for_each(|(function_declaration, function_name)| {
            results.push(Result {
                name: self.name().to_string(),
                impact: self.impact(),
                confidence: self.confidence(),
                message: format!("{} defined in {} is declared as view but writes to the storage variables",function_name, function_declaration)
            })
        });
        results


    }

}
