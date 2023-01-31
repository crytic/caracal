use crate::detectors::detector::Result;
use crate::detectors::get_detectors;
use crate::{Args, CompilationUnit};
use crate::{Filter, Print};

pub struct CoreUnit<'a> {
    compilation_unit: CompilationUnit<'a>,
    args: Args,
}

impl<'a> CoreUnit<'a> {
    pub fn new(compilation_unit: CompilationUnit<'a>, args: Args) -> Self {
        CoreUnit {
            compilation_unit,
            args,
        }
    }

    pub fn get_compilation_unit(&self) -> &CompilationUnit {
        &self.compilation_unit
    }

    pub fn run(&mut self) {
        self.compilation_unit.analyze();
        let mut results: Vec<Result> = Vec::new();
        let detectors_to_run = get_detectors();
        for d in detectors_to_run {
            results.extend(d.run(self));
        }

        for r in results {
            println!(
                "{} impact: {} confidence: {}\n {}",
                r.name, r.impact, r.confidence, r.message
            );
        }

        self.run_printer();
    }

    fn run_printer(&self) {
        match self.args.filter {
            Filter::All => match self.args.print {
                Some(Print::Cfg) => {
                    for f in self.compilation_unit.functions() {
                        f.cfg_to_dot(f.get_cfg());
                    }
                }
                Some(Print::CfgOptimized) => {
                    for f in self.compilation_unit.functions() {
                        f.cfg_to_dot(f.get_cfg_optimized());
                    }
                }
                _ => (),
            },
            Filter::UserFunctions => match self.args.print {
                Some(Print::Cfg) => {
                    for f in self.compilation_unit.functions_user_defined() {
                        f.cfg_to_dot(f.get_cfg());
                    }
                }
                Some(Print::CfgOptimized) => {
                    for f in self.compilation_unit.functions_user_defined() {
                        f.cfg_to_dot(f.get_cfg_optimized());
                    }
                }
                _ => (),
            },
        }
    }
}
