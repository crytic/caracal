use anyhow::{bail, Context};
use cairo_lang_filesystem::ids::Directory;
use clap::{Parser, ValueEnum};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_compiler::project::{setup_project, ProjectConfig, ProjectConfigContent};
use cairo_lang_compiler::CompilerConfig;
use cairo_lang_sierra::extensions::core::{CoreLibfunc, CoreType};
use cairo_lang_sierra::program_registry::ProgramRegistry;
use cairo_lang_sierra_generator::db::SierraGenGroup;
use cairo_lang_sierra_generator::replace_ids::replace_sierra_ids_in_program;
use cairo_lang_starknet::abi::AbiBuilder;
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

    /// Corelib path (e.g. mypath/corelib/src)
    #[arg(long)]
    corelib: Option<PathBuf>,
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

    // corelib cli option has priority over the environment variable
    let corelib = match args.corelib {
        Some(ref p) => p.clone(),
        None => {
            match env::var("CORELIB_PATH") {
                Ok(p) => p.into(),
                Err(e) => bail!("{e}. The Corelib path must be specified either with the CORELIB_PATH environment variable or the --corelib cli option"),
            }
        }
    };

    // Needed to pass the correct corelib path
    let project_config = ProjectConfig {
        corelib: Some(Directory(corelib)),
        base_path: "".into(),
        content: ProjectConfigContent {
            crate_roots: HashMap::new(),
        },
    };
    let mut db = RootDatabase::builder()
        .with_project_config(project_config)
        .with_starknet()
        .build()?;
    let mut compiler_config = CompilerConfig {
        replace_ids: true,
        ..CompilerConfig::default()
    };
    let main_crate_ids = setup_project(&mut db, &args.file)?;
    compiler_config.diagnostics_reporter.ensure(&mut db)?;

    let contracts = find_contracts(&db, &main_crate_ids);
    let contract = match &contracts[..] {
        [contract] => contract,
        [] => bail!("Contract not found."),
        _ => {
            bail!("Compilation unit must include only one contract.",)
        }
    };

    let abi = AbiBuilder::from_trait(&db, get_abi(&db, contract)?).with_context(|| "ABI error")?;
    let sierra = db
        .get_sierra_program(main_crate_ids)
        .ok()
        .context("Compilation failed without any diagnostics.")?;
    let sierra = replace_sierra_ids_in_program(&mut db, &sierra);
    let registry = ProgramRegistry::<CoreType, CoreLibfunc>::new(&sierra)?;
    let compilation_unit = CompilationUnit::new(&sierra, abi, registry);

    let mut core = CoreUnit::new(compilation_unit, args);
    core.run();

    anyhow::Ok(())
}
