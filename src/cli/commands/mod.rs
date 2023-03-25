use self::{
    detect::DetectArgs, detectors::DetectorsArgs, print::PrintArgs, printers::PrintersArgs,
};
use clap::Subcommand;

mod detect;
mod detectors;
mod print;
mod printers;

/// The supported commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List of the detectors available
    Detectors(DetectorsArgs),
    /// Run the detectors
    Detect(DetectArgs),
    /// List of the printers available
    Printers(PrintersArgs),
    /// Run the printers
    Print(PrintArgs),
}

pub trait Cmd {
    fn run(&self) -> anyhow::Result<()>;
}

impl Cmd for Commands {
    fn run(&self) -> anyhow::Result<()> {
        match self {
            Commands::Detect(cmd) => cmd.run(),
            Commands::Detectors(cmd) => cmd.run(),
            Commands::Print(cmd) => cmd.run(),
            Commands::Printers(cmd) => cmd.run(),
        }
    }
}

/* impl From<Args> for CoreOpts {
    fn from(args: Args) -> Self {
        CoreOpts {
            file: args.file,
            filter: args.filter,
            print: args.print,
            corelib: args.corelib,
        }
    }
}
 */
