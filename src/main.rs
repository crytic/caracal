use clap::Parser;

use crate::cli::commands::Cmd;

mod cli;

fn main() -> anyhow::Result<()> {
    let args = cli::CliArgs::parse();

    args.command.run()?;

    anyhow::Ok(())
}
