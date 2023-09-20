use anyhow::{anyhow, Result};
use std::fs;

use cairo_lang_sierra::program::Program;
use cairo_lang_starknet::abi::Contract;

use crate::core::core_unit::CoreOpts;

mod cairo_project;
mod scarb;
mod standard;
mod utils;

pub struct ProgramCompiled {
    pub sierra: Program,
    pub abi: Contract,
}

pub fn compile(opts: CoreOpts) -> Result<Vec<ProgramCompiled>> {
    if opts.target.is_dir() {
        if let Ok(entries) = fs::read_dir(opts.target.as_path()) {
            for entry in entries.flatten() {
                if entry.file_name() == "Scarb.toml" {
                    println!("Compiling with Scarb. Found Scarb.toml.");
                    return scarb::compile(opts);
                } else if entry.file_name() == "cairo_project.toml" {
                    println!("Compiling with Cairo. Found cairo_project.toml.");
                    return cairo_project::compile(opts);
                }
            }
        }
        Err(anyhow!("Compilation framework not found."))
    } else {
        standard::compile(opts)
    }
}
