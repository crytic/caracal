use anyhow::{anyhow, bail, Context, Result};
use std::env;
use std::process;
use std::sync::Arc;

use super::ProgramCompiled;
use crate::compilation::utils::felt252_serde::sierra_from_felt252s;
use crate::compilation::utils::replacer::SierraProgramDebugReplacer;
use crate::core::core_unit::CoreOpts;
use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_compiler::project::{setup_project, ProjectConfig, ProjectConfigContent};
use cairo_lang_compiler::CompilerConfig;
use cairo_lang_filesystem::ids::Directory;
use cairo_lang_sierra_generator::db::SierraGenGroup;
use cairo_lang_sierra_generator::replace_ids::{replace_sierra_ids_in_program, SierraIdReplacer};
use cairo_lang_starknet::abi::{AbiBuilder, Contract};
use cairo_lang_starknet::compiler_version::current_compiler_version_id;
use cairo_lang_starknet::contract::find_contracts;
use cairo_lang_starknet::contract_class::ContractClass;
use cairo_lang_starknet::inline_macros::selector::SelectorMacro;
use cairo_lang_starknet::plugin::StarkNetPlugin;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;

pub fn compile(opts: CoreOpts) -> Result<Vec<ProgramCompiled>> {
    let output = process::Command::new("starknet-compile")
        .arg("--version")
        .output();

    if let Ok(result) = output {
        if result.status.success() {
            println!(
                "Found local cairo installation {}",
                String::from_utf8(result.stdout)?
            );

            return local_compiler(opts);
        }
    }

    println!(
        "Local cairo installation not found. Compiling with starknet-compile {}",
        current_compiler_version_id()
    );

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
        corelib: Some(Directory::Real(corelib)),
        base_path: "".into(),
        content: ProjectConfigContent {
            crate_roots: OrderedHashMap::default(),
        },
    };

    let mut db = RootDatabase::builder()
        .with_project_config(project_config)
        .with_macro_plugin(Arc::new(StarkNetPlugin::default()))
        .with_inline_macro_plugin(SelectorMacro::NAME, Arc::new(SelectorMacro))
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

fn local_compiler(opts: CoreOpts) -> Result<Vec<ProgramCompiled>> {
    let output = if let Some(contract_path) = opts.contract_path {
        process::Command::new("starknet-compile")
            .arg(opts.target)
            .arg("--contract-path")
            .arg(contract_path)
            .arg("--replace-ids")
            .output()?
    } else {
        process::Command::new("starknet-compile")
            .arg(opts.target)
            .arg("--replace-ids")
            .output()?
    };

    if !output.status.success() {
        bail!(anyhow!(
            "starknet-compile failed to compile.\n Status {}\n {}",
            output.status,
            String::from_utf8(output.stderr)?
        ));
    }

    let contract_class: ContractClass =
        serde_json::from_str(&String::from_utf8(output.stdout)?).unwrap();

    // We don't have to check the existence because we ran the compiler with --replace-ids
    let debug_info = contract_class.sierra_program_debug_info.unwrap();

    let sierra = sierra_from_felt252s(&contract_class.sierra_program)
        .unwrap()
        .2;
    let sierra = SierraProgramDebugReplacer { debug_info }.apply(&sierra);

    Ok(vec![ProgramCompiled {
        sierra,
        abi: contract_class.abi.unwrap(),
    }])
}
