use self::printer::Printer;

pub mod cfg;
pub mod cfg_optimized;
pub mod printer;

pub fn get_printers() -> Vec<Box<dyn Printer>> {
    vec![
        Box::<cfg::CFGPrinter>::default(),
        Box::<cfg_optimized::CFGOptimizedPrinter>::default(),
    ]
}
