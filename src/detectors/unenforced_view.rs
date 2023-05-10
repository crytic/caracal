use super::detector::{Confidence, Detector, Impact, Result};
use crate::core::core_unit::CoreUnit;
use crate::core::function::Type;
use std::collections::{HashMap,HashSet};
use cairo_lang_sierra::program::Statement as SierraStatement;
use cairo_lang_sierra::extensions::core::CoreConcreteLibfunc;


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
        for func in compilation_unit.functions() {
            let b =func.storage_vars_written().peekable().peek().is_none();
            println!("function type: {:?} function name: {}, and storage vars: {}", *func.ty(), func.name(), b);
        }
        let unenforced_view_funcs: HashSet<String> = compilation_unit.functions().filter(|f| *f.ty() == Type::View && !f.storage_vars_written().peekable().peek().is_none()).map(|f| f.name()).collect();

        let view_funcs:Vec<_> = compilation_unit.functions().filter(|f| *f.ty() == Type::View).collect();

        for func in view_funcs {
            let func_name = func.name();
            let (declaration,name) = func_name.rsplit_once("::").unwrap();
            if func.storage_vars_written().peekable().peek().is_none() {
                results.push(Result { name: self.name().to_string(), impact: self.impact(), confidence: self.confidence(), message: format!("{} defined in {} is declared as view but writes to storage vars",name,declaration) });
            }
            //chain together all internal/external/core/lib calls
            //iterate over each of them, turning them back into a function and investigating their external calls and so on
            // if there is a storage variable read, we push the original function name to the result
        }

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
