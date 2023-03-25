use super::printer::{Filter, PrintOpts, Printer, Result};
use crate::core::{core_unit::CoreUnit, function::Function};

#[derive(Default)]
pub struct CFGOptimizedPrinter {}

impl Printer for CFGOptimizedPrinter {
    fn name(&self) -> &str {
        "cfg-optimized"
    }

    fn description(&self) -> &str {
        "Export the CFG optimized of each function in a .dot file"
    }

    fn run(&self, core: &CoreUnit, opts: PrintOpts) -> Vec<Result> {
        let mut results = Vec::new();
        let compilation_unit = core.get_compilation_unit();

        match opts.filter {
            Filter::All => compilation_unit
                .functions()
                .for_each(|f| self.print_cfg_optimized(f, &mut results)),
            Filter::UserFunctions => compilation_unit
                .functions_user_defined()
                .for_each(|f| self.print_cfg_optimized(f, &mut results)),
        }

        results
    }
}

impl CFGOptimizedPrinter {
    fn print_cfg_optimized(&self, function: &Function, results: &mut Vec<Result>) {
        let message = format!(
            "CFG optimized for the function {} in {}",
            function.name(),
            function.cfg_to_dot(function.get_cfg_optimized())
        );
        results.push(Result {
            name: self.name().to_string(),
            message,
        });
    }
}
