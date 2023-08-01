use super::Cmd;
use caracal::printers::get_printers;
use clap::Args;

#[derive(Args, Debug)]
pub struct PrintersArgs {}

impl Cmd for PrintersArgs {
    fn run(&self) -> anyhow::Result<()> {
        get_printers().iter().for_each(|d| println!("{}", d));
        Ok(())
    }
}
