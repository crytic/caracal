use crate::core::core_unit::CoreUnit;
use anyhow::{Context, Ok};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

use crate::core::compilation_unit::CompilationUnit;
use cairo_lang_compiler::project::setup_project;
use cairo_lang_sierra::extensions::core::{CoreLibfunc, CoreType};
use cairo_lang_sierra::program_registry::ProgramRegistry;
use cairo_lang_sierra_generator::db::SierraGenGroup;
use cairo_lang_sierra_generator::replace_ids::replace_sierra_ids_in_program;
use cairo_lang_starknet::db::get_starknet_database;

mod core;
mod detectors;

/// Starknet smart contract static analysis tool
#[derive(Parser, Debug)]
pub struct Args {
    /// File to analyze
    file: PathBuf,

    /// Which functions to analyze    
    #[arg(short, long, value_enum, default_value_t = Filter::UserFunctions)]
    filter: Filter,

    /// Print something
    #[arg(short, long, value_enum, value_name = "WHAT")]
    print: Option<Print>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Filter {
    /// All the functions in the program (core library functions, wrapper functions...)
    All,
    /// Only user defined functions
    UserFunctions,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Print {
    /// A CFG of the SIERRA represenation
    Cfg,
    /// An optimized CFG of the SIERRA representation
    CfgOptimized,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut db_val = get_starknet_database();
    let db = &mut db_val;

    let main_crate_ids = setup_project(db, &args.file)?;

    let sierra = db
        .get_sierra_program(main_crate_ids)
        .ok()
        .context("Compilation failed without any diagnostics.")?;
    let sierra = replace_sierra_ids_in_program(db, &sierra);
    let registry = ProgramRegistry::<CoreType, CoreLibfunc>::new(&sierra)?;
    let compilation_unit = CompilationUnit::new(&sierra);

    let mut core = CoreUnit::new(compilation_unit, args, registry);
    core.run();

    Ok(())
}
