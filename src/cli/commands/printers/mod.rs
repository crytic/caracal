use super::Cmd;
use clap::Args;
use starknet_static_analysis::printers::get_printers;

#[derive(Args, Debug)]
pub struct PrintersArgs {}

impl Cmd for PrintersArgs {
    fn run(&self) -> anyhow::Result<()> {
        get_printers().iter().for_each(|d| println!("{}", d));
        Ok(())
    }
}
