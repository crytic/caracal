use anyhow::Result;
use std::path::PathBuf;

use cairo_lang_sierra::extensions::core::{CoreLibfunc, CoreType};
use cairo_lang_sierra::program_registry::ProgramRegistry;

use crate::compilation::compile;
use crate::core::compilation_unit::CompilationUnit;

pub struct CoreOpts {
    pub target: PathBuf,
    pub corelib: Option<PathBuf>,
}

pub struct CoreUnit {
    compilation_units: Vec<CompilationUnit>,
}

impl CoreUnit {
    pub fn new(opts: CoreOpts) -> Result<Self> {
        let program_compiled = compile(opts)?;
        let compilation_units = program_compiled
            .iter()
            .map(|p| {
                let mut compilation_unit = CompilationUnit::new(
                    p.sierra.clone(),
                    p.abi.clone(),
                    ProgramRegistry::<CoreType, CoreLibfunc>::new(&p.sierra).unwrap(),
                );
                compilation_unit.analyze();
                compilation_unit
            })
            .collect();

        Ok(CoreUnit { compilation_units })
    }

    pub fn get_compilation_units(&self) -> &Vec<CompilationUnit> {
        &self.compilation_units
    }
}
