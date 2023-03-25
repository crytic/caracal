use super::Cmd;
use clap::{Args, ValueHint};
use starknet_static_analysis::{
    core::core_unit::{CoreOpts, CoreUnit},
    detectors::get_detectors,
};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct DetectArgs {
    /// File to analyze
    #[arg(value_hint = ValueHint::FilePath)]
    file: PathBuf,

    /// Corelib path (e.g. mypath/corelib/src)
    #[arg(long)]
    corelib: Option<PathBuf>,
}

impl From<&DetectArgs> for CoreOpts {
    fn from(args: &DetectArgs) -> Self {
        CoreOpts {
            file: args.file.clone(),
            corelib: args.corelib.clone(),
        }
    }
}

impl Cmd for DetectArgs {
    fn run(&self) -> anyhow::Result<()> {
        let core = CoreUnit::new(self.into())?;
        get_detectors()
            .iter()
            .map(|d| d.run(&core))
            .for_each(|results| results.iter().for_each(|r| println!("{r}")));

        Ok(())
    }
}
