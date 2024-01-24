use anyhow::{anyhow, bail, Result};
use std::fs;
use std::path::Path;
use std::process;

use crate::compilation::utils::felt252_serde::sierra_from_felt252s;
use crate::compilation::utils::replacer::SierraProgramDebugReplacer;
use crate::compilation::ProgramCompiled;
use crate::core::core_unit::CoreOpts;
use cairo_lang_sierra_generator::replace_ids::SierraIdReplacer;
use cairo_lang_starknet::contract_class::ContractClass;

pub fn compile(opts: CoreOpts) -> Result<Vec<ProgramCompiled>> {
    process::Command::new("scarb")
        .current_dir(opts.target.as_path())
        .arg("clean")
        .output()?;

    let output = process::Command::new("scarb")
        .current_dir(opts.target.as_path())
        .arg("build")
        .arg("--workspace")
        .output()?;

    if !output.status.success() {
        bail!(anyhow!(
            "Scarb failed to compile.\n Status {}\n {}",
            output.status,
            String::from_utf8(output.stdout)?
        ));
    }

    let mut sierra_files_path = vec![];

    if let Ok(entries) = fs::read_dir(opts.target.as_path().join(Path::new("target/dev"))) {
        let accepted_formats = [
            // For scarb <= 0.7.0
            ".sierra",
            ".contract_class",
        ];
        for entry in entries.flatten() {
            if accepted_formats.iter().any(|f| {
                entry
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .ends_with(*f)
            }) {
                sierra_files_path.push(entry.path());
            }
        }
    }

    if sierra_files_path.is_empty() {
        bail!(anyhow!("Compiled sierra files not found. Ensure in Scarb.toml you have\n[[target.starknet-contract]]\nsierra = true"));
    }

    let mut programs_compiled: Vec<ProgramCompiled> = vec![];

    for sierra_file in sierra_files_path {
        let contents =
            fs::read_to_string(sierra_file.as_path()).expect("Failed to read a sierra file");
        // In some cases a .sierra is made even for newer scarb version which does not have a contract class
        // and it is not needed for us so if we get an error we skip the file
        let contract_class: ContractClass = if let Ok(c) = serde_json::from_str(&contents) {
            c
        } else {
            continue;
        };
        let debug_info;

        if contract_class.sierra_program_debug_info.is_none() {
            println!("Skipping analysing file {}. Debug info not found. Ensure in Scarb.toml you have \n[cairo]\nsierra-replace-ids = true\n", sierra_file.to_str().unwrap());
            continue;
        } else {
            debug_info = contract_class.sierra_program_debug_info.unwrap();
            if debug_info.libfunc_names.is_empty()
                && debug_info.type_names.is_empty()
                && debug_info.user_func_names.is_empty()
            {
                println!("Skipping analysing file {}. Debug info not found. If the file has code ensure in Scarb.toml you have \n[cairo]\nsierra-replace-ids = true\n", sierra_file.to_str().unwrap());
                continue;
            }
        }

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
