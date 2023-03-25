use super::Cmd;
use clap::{Args, ValueHint};
use starknet_static_analysis::core::core_unit::{CoreOpts, CoreUnit};
use starknet_static_analysis::printers::{get_printers, printer::Filter, printer::PrintOpts};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct PrintArgs {
    /// File to analyze
    #[arg(value_hint = ValueHint::FilePath)]
    file: PathBuf,

    /// Corelib path (e.g. mypath/corelib/src)
    #[arg(long)]
    corelib: Option<PathBuf>,

    /// Which functions to run the printer (all, user-functions)   
    #[arg(short, long, default_value_t = Filter::UserFunctions)]
    filter: Filter,

    /// Which printer to use
    #[arg(short, long)]
    what: String,
}

impl From<&PrintArgs> for CoreOpts {
    fn from(args: &PrintArgs) -> Self {
        CoreOpts {
            file: args.file.clone(),
            corelib: args.corelib.clone(),
        }
    }
}

impl From<&PrintArgs> for PrintOpts {
    fn from(args: &PrintArgs) -> Self {
        PrintOpts {
            filter: args.filter,
        }
    }
}

impl Cmd for PrintArgs {
    fn run(&self) -> anyhow::Result<()> {
        let printers = get_printers();
        let printer = printers
            .iter()
            .find(|printer| printer.name() == self.what)
            .expect("Invalid printer provided");

        let core = CoreUnit::new(self.into())?;

        printer
            .run(&core, self.into())
            .iter()
            .for_each(|r| println!("{r}"));

        Ok(())
    }
}
