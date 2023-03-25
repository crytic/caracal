use super::Cmd;
use clap::Args;
use starknet_static_analysis::detectors::get_detectors;

#[derive(Args, Debug)]
pub struct DetectorsArgs {}

impl Cmd for DetectorsArgs {
    fn run(&self) -> anyhow::Result<()> {
        get_detectors().iter().for_each(|d| println!("{}", d));
        Ok(())
    }
}
