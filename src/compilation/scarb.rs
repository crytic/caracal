use anyhow::{anyhow, bail, Result};
use std::fs;
use std::path::Path;
use std::process;

use crate::compilation::utils::felt252_serde::sierra_from_felt252s;
use crate::compilation::ProgramCompiled;
use crate::core::core_unit::CoreOpts;
use cairo_lang_sierra::debug_info::DebugInfo;
use cairo_lang_sierra_generator::replace_ids::SierraIdReplacer;
use cairo_lang_starknet::contract_class::ContractClass;

pub fn compile(opts: CoreOpts) -> Result<Vec<ProgramCompiled>> {
    let output = process::Command::new("scarb")
        .current_dir(opts.target.as_path())
        .arg("build")
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
        for entry in entries.flatten() {
            if entry
                .path()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .ends_with(".sierra")
            {
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
        let contract_class: ContractClass = serde_json::from_str(&contents).unwrap();
        let debug_info;

        if contract_class.sierra_program_debug_info.is_none() {
            println!("Skipping analysing file {}. Debug info not found. Ensure in Scarb.toml you have \n[cairo]\nsierra-replace-ids = true", sierra_file.to_str().unwrap());
            continue;
        } else {
            debug_info = contract_class.sierra_program_debug_info.unwrap();
            if debug_info.libfunc_names.is_empty()
                && debug_info.type_names.is_empty()
                && debug_info.user_func_names.is_empty()
            {
                println!("Skipping analysing file {}. Debug info empty. Ensure in Scarb.toml you have \n[cairo]\nsierra-replace-ids = true", sierra_file.to_str().unwrap());
                continue;
            }
        }

        let program = sierra_from_felt252s(&contract_class.sierra_program)
            .unwrap()
            .2;
        let program = ScarbDebugReplacer { debug_info }.apply(&program);
        programs_compiled.push(ProgramCompiled {
            sierra: program,
            abi: contract_class.abi.unwrap(),
        });
    }

    Ok(programs_compiled)
}

struct ScarbDebugReplacer {
    debug_info: DebugInfo,
}

impl SierraIdReplacer for ScarbDebugReplacer {
    fn replace_libfunc_id(
        &self,
        id: &cairo_lang_sierra::ids::ConcreteLibfuncId,
    ) -> cairo_lang_sierra::ids::ConcreteLibfuncId {
        let func_name = self
            .debug_info
            .libfunc_names
            .get(id)
            .expect("No libfunc in debug info");
        cairo_lang_sierra::ids::ConcreteLibfuncId {
            id: id.id,
            debug_name: Some(func_name.clone()),
        }
    }

    fn replace_type_id(
        &self,
        id: &cairo_lang_sierra::ids::ConcreteTypeId,
    ) -> cairo_lang_sierra::ids::ConcreteTypeId {
        let type_name = self
            .debug_info
            .type_names
            .get(id)
            .expect("No typeid in debug info");
        cairo_lang_sierra::ids::ConcreteTypeId {
            id: id.id,
            debug_name: Some(type_name.clone()),
        }
    }

    fn replace_function_id(
        &self,
        sierra_id: &cairo_lang_sierra::ids::FunctionId,
    ) -> cairo_lang_sierra::ids::FunctionId {
        let function_name = self
            .debug_info
            .user_func_names
            .get(sierra_id)
            .expect("No funcid in debug info");
        cairo_lang_sierra::ids::FunctionId {
            id: sierra_id.id,
            debug_name: Some(function_name.clone()),
        }
    }
}
