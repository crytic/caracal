use anyhow::{bail, Context, Ok};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_compiler::diagnostics::check_diagnostics;
use cairo_lang_compiler::project::setup_project;
use cairo_lang_compiler::CompilerConfig;
use cairo_lang_sierra::extensions::core::{CoreLibfunc, CoreType};
use cairo_lang_sierra::program_registry::ProgramRegistry;
use cairo_lang_sierra_generator::db::SierraGenGroup;
use cairo_lang_sierra_generator::replace_ids::replace_sierra_ids_in_program;
use cairo_lang_starknet::abi::Contract;
use cairo_lang_starknet::contract::{find_contracts, get_abi};
use cairo_lang_starknet::db::StarknetRootDatabaseBuilderEx;

use crate::core::compilation_unit::CompilationUnit;
use crate::core::core_unit::CoreUnit;

mod analysis;
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

    // This part is adapted from cairo_lang_starknet::contract_class::compile_path
    let mut db_val = {
        let mut b = RootDatabase::builder();
        b.with_dev_corelib().unwrap();
        b.with_starknet();
        b.build()
    };
    let db = &mut db_val;

    let main_crate_ids = setup_project(db, &args.file)?;

    if check_diagnostics(db, CompilerConfig::default().on_diagnostic) {
        bail!("Compilation failed.");
    }

    let contracts = find_contracts(db, &main_crate_ids);
    let contract = match &contracts[..] {
        [contract] => contract,
        [] => bail!("Contract not found."),
        _ => {
            bail!("Compilation unit must include only one contract.",)
        }
    };

    let abi = Contract::from_trait(db, get_abi(db, contract)?).with_context(|| "ABI error")?;
    let sierra = db
        .get_sierra_program(main_crate_ids)
        .ok()
        .context("Compilation failed without any diagnostics.")?;
    let sierra = replace_sierra_ids_in_program(db, &sierra);
    let registry = ProgramRegistry::<CoreType, CoreLibfunc>::new(&sierra)?;
    let compilation_unit = CompilationUnit::new(&sierra, abi, registry);

    let mut core = CoreUnit::new(compilation_unit, args);
    core.run();

    Ok(())
}
