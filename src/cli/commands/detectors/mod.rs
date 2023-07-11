use super::Cmd;
use caracal::detectors::get_detectors;
use clap::Args;

#[derive(Args, Debug)]
pub struct DetectorsArgs {}

impl Cmd for DetectorsArgs {
    fn run(&self) -> anyhow::Result<()> {
        get_detectors().iter().for_each(|d| println!("{}", d));
        Ok(())
    }
}
