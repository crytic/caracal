use self::{cfg::CFGPrinter, cfg_optimized::CFGOptimizedPrinter, printer::Printer};

pub mod cfg;
pub mod cfg_optimized;
pub mod printer;

pub fn get_printers() -> Vec<Box<dyn Printer>> {
    vec![
        Box::<CFGPrinter>::default(),
        Box::<CFGOptimizedPrinter>::default(),
    ]
}
