use anyhow::{bail, Context, Result};
use std::env;
use std::sync::Arc;

use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_compiler::project::{setup_project, ProjectConfig, ProjectConfigContent};
use cairo_lang_compiler::CompilerConfig;
use cairo_lang_filesystem::ids::Directory;
use cairo_lang_sierra_generator::db::SierraGenGroup;
use cairo_lang_sierra_generator::replace_ids::replace_sierra_ids_in_program;
use cairo_lang_starknet::abi::{AbiBuilder, Contract};
use cairo_lang_starknet::contract::find_contracts;
use cairo_lang_starknet::plugin::StarkNetPlugin;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;

use super::ProgramCompiled;
use crate::core::core_unit::CoreOpts;

pub fn compile(opts: CoreOpts) -> Result<Vec<ProgramCompiled>> {
    // NOTE: compiler_version module is not public so we need to update it as we update the Cairo version we use
    println!("Compiling with starknet-compile 2.1.0.");

    // corelib cli option has priority over the environment variable
    let corelib = match opts.corelib {
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
            crate_roots: OrderedHashMap::default(),
        },
    };

    let mut db = RootDatabase::builder()
        .with_project_config(project_config)
        .with_semantic_plugin(Arc::new(StarkNetPlugin::default()))
        .build()?;

    let mut compiler_config = CompilerConfig::default();

    let main_crate_ids = setup_project(&mut db, &opts.target)?;
    compiler_config.diagnostics_reporter.ensure(&db)?;

    let contracts = find_contracts(&db, &main_crate_ids);
    if contracts.is_empty() {
        bail!("Contract not found.");
    }

    let mut abi: Contract = Default::default();
    contracts.iter().for_each(|c| {
        abi.items.extend(
            AbiBuilder::submodule_as_contract_abi(&db, c.submodule_id)
                .expect("Error when getting the ABI.")
                .items,
        )
    });

    let sierra = db
        .get_sierra_program(main_crate_ids)
        .ok()
        .context("Compilation failed without any diagnostics.")?;
    let sierra = replace_sierra_ids_in_program(&db, &sierra);

    Ok(vec![ProgramCompiled { sierra, abi }])
}
