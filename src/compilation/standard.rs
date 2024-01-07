use anyhow::{anyhow, bail, Result};
use std::env;
use std::process;

use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_compiler::project::{setup_project, ProjectConfig, ProjectConfigContent};
use cairo_lang_compiler::CompilerConfig;
use cairo_lang_filesystem::ids::Directory;
use cairo_lang_sierra_generator::replace_ids::SierraIdReplacer;
use cairo_lang_starknet::compiler_version::current_compiler_version_id;
use cairo_lang_starknet::contract::find_contracts;
use cairo_lang_starknet::contract_class::{compile_prepared_db, ContractClass};
use cairo_lang_starknet::starknet_plugin_suite;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;

use super::ProgramCompiled;
use crate::compilation::utils::felt252_serde::sierra_from_felt252s;
use crate::compilation::utils::replacer::SierraProgramDebugReplacer;
use crate::core::core_unit::CoreOpts;

pub fn compile(opts: CoreOpts) -> Result<Vec<ProgramCompiled>> {
    let output = process::Command::new("starknet-compile")
        .arg("--version")
        .output();

    if let Ok(result) = output {
        if result.status.success() {
            let version = String::from_utf8(result.stdout)?;
            println!("Found local cairo installation {}", version);
            // We have to check the version because if it's >= 2.1.0 the compiler needs --single-file flag
            let version: Vec<&str> = version
                .split_whitespace()
                .nth(1)
                .unwrap()
                .split('.')
                .collect();
            if version[0] >= "2" && version[1] >= "1" {
                return local_compiler(opts, true);
            }

            return local_compiler(opts, false);
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
            crates_config: Default::default(),
        },
    };

    let mut db = RootDatabase::builder()
        .with_project_config(project_config)
        .with_plugin_suite(starknet_plugin_suite())
        .build()?;

    let compiler_config = CompilerConfig {
        replace_ids: true,
        ..Default::default()
    };

    let main_crate_ids = setup_project(&mut db, &opts.target)?;

    let contracts = find_contracts(&db, &main_crate_ids);
    if contracts.is_empty() {
        bail!("Contract not found.");
    }

    let mut contracts_arg = vec![];
    contracts.iter().for_each(|c| contracts_arg.push(c));

    let contract_classes = compile_prepared_db(&db, &contracts_arg, compiler_config)
        .expect("Error when compiling contracts.");

    let mut programs_compiled: Vec<ProgramCompiled> = vec![];

    for contract_class in contract_classes {
        let debug_info = contract_class.sierra_program_debug_info.unwrap();
        let program = sierra_from_felt252s(&contract_class.sierra_program)
            .unwrap()
            .2;
        let program = SierraProgramDebugReplacer { debug_info }.apply(&program);

        programs_compiled.push(ProgramCompiled {
            sierra: program,
            abi: contract_class.abi.unwrap(),
        });
    }

    Ok(programs_compiled)
}

fn local_compiler(opts: CoreOpts, single_file_flag: bool) -> Result<Vec<ProgramCompiled>> {
    let output = if single_file_flag {
        process::Command::new("starknet-compile")
            .arg("--single-file")
            .arg(opts.target)
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
