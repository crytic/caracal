use clap::Parser;
use commands::Commands;

pub mod commands;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct CliArgs {
    /// command to run
    #[clap(subcommand)]
    pub command: Commands,
}
