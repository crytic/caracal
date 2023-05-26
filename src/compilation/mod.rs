use anyhow::{anyhow, Result};
use std::fs;

use cairo_lang_sierra::program::Program;
use cairo_lang_starknet::abi::Contract;

use crate::core::core_unit::CoreOpts;

mod scarb;
mod standard;

pub struct ProgramCompiled {
    pub sierra: Program,
    pub abi: Contract,
}

pub fn compile(opts: CoreOpts) -> Result<Vec<ProgramCompiled>> {
    if opts.target.is_dir() {
        if let Ok(entries) = fs::read_dir(opts.target.as_path()) {
            for entry in entries.flatten() {
                if entry.file_name() == "Scarb.toml" {
                    return scarb::compile(opts);
                }
            }
        }
        Err(anyhow!("Compilation framework not found."))
    } else {
        standard::compile(opts)
    }
}
