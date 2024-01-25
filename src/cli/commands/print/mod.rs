use super::Cmd;
use caracal::core::core_unit::{CoreOpts, CoreUnit};
use caracal::printers::{get_printers, printer::Filter, printer::PrintOpts};
use clap::{Args, ValueHint};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct PrintArgs {
    /// Target to analyze
    #[arg(value_hint = ValueHint::FilePath)]
    target: PathBuf,

    /// Corelib path (e.g. mypath/corelib/src)
    #[arg(long)]
    corelib: Option<PathBuf>,

    /// Path to the contracts to compile when using a cairo project with multiple contracts
    #[arg(long, num_args(0..))]
    contract_path: Option<Vec<String>>,

    /// Functions name that are safe when called (e.g. they don't cause a reentrancy)
    #[arg(long, num_args(0..))]
    safe_external_calls: Option<Vec<String>>,

    /// Which functions to run the printer (all, user-functions)   
    #[arg(short, long, default_value_t = Filter::UserFunctions)]
    filter: Filter,

    /// Which printer to use
    #[arg(short, long)]
    printer: String,
}

impl From<&PrintArgs> for CoreOpts {
    fn from(args: &PrintArgs) -> Self {
        CoreOpts {
            target: args.target.clone(),
            corelib: args.corelib.clone(),
            contract_path: args.contract_path.clone(),
            safe_external_calls: args.safe_external_calls.clone(),
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
            .find(|printer| printer.name() == self.printer)
            .expect("Invalid printer provided");

        let core = CoreUnit::new(self.into())?;

        printer
            .run(&core, self.into())
            .iter()
            .for_each(|r| println!("{r}"));

        Ok(())
    }
}
