use self::printer::Printer;

pub mod cfg;
pub mod printer;
pub mod callgraph;

pub fn get_printers() -> Vec<Box<dyn Printer>> {
    vec![Box::<cfg::CFGPrinter>::default(),Box::<callgraph::CallgraphPrinter>::default()]
}
