use clap::Parser;
use commands::Commands;

pub mod commands;

/// Starknet smart contract static analysis tool
#[derive(Parser, Debug)]
pub struct CliArgs {
    /// command to run
    #[clap(subcommand)]
    pub command: Commands,
}
