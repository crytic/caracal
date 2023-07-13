use super::printer::{Filter, PrintOpts, Printer, Result};
use crate::core::compilation_unit::CompilationUnit;
use crate::core::{core_unit::CoreUnit, function::Function};
use cairo_lang_sierra::extensions::core::CoreConcreteLibfunc;
use cairo_lang_sierra::program::Statement as SierraStatement;
use graphviz_rust::dot_generator::*;
use graphviz_rust::dot_structures::*;
use graphviz_rust::printer::{DotPrinter, PrinterContext};
use std::collections::{HashMap, HashSet};
use std::io::Write;

#[derive(Default)]
pub struct CallgraphPrinter {}

impl Printer for CallgraphPrinter {
    fn name(&self) -> &str {
        "callgraph"
    }
    fn description(&self) -> &str {
        "Export function call graph to a .dot file"
    }
    fn run(&self, core: &CoreUnit, opts: PrintOpts) -> Vec<Result> {
        let mut results = Vec::new();
        let compilation_units = core.get_compilation_units();
        for compilation_unit in compilation_units {
            let mut tracked_contracts = HashMap::new();
            let (module_name, _) =
                self.get_names(&compilation_unit.functions().next().unwrap().name());

            let mut graph = graph!( di id!(format!("\"{}\"",&module_name)));
            match opts.filter {
                Filter::All => compilation_unit.functions().for_each(|f| {
                    self.print_callgraph(&compilation_unit, f, &mut tracked_contracts, &mut graph)
                }),
                Filter::UserFunctions => compilation_unit.functions_user_defined().for_each(|f| {
                    self.print_callgraph(&compilation_unit, f, &mut tracked_contracts, &mut graph)
                }),
            }
            // Add generated subgraphs to original digraph
            for val in tracked_contracts.values() {
                graph.add_stmt(Stmt::Subgraph(val.clone()));
            }
            // Write callgraph to file
            let output = graph.print(&mut PrinterContext::default());

            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(format!("{}.dot", module_name))
                .expect("Error when creating file");

            f.write_all(&output.as_bytes()).unwrap();
            let message = format!("Call graph for module {}", &module_name);
            results.push(Result {
                name: self.name().to_string(),
                message,
            });
        }

        results
    }
}
impl CallgraphPrinter {
    fn print_callgraph(
        &self,
        compilation_unit: &CompilationUnit,
        f: &Function,
        tracked_contracts: &mut HashMap<String, Subgraph>,
        graph: &mut Graph,
    ) {
        // Add a node/module for the current function (if it doesn't exist yet, we add a module subgraph)
        let calling_fn_name = &f.name();
        let mut tracked_fns: HashSet<String> = HashSet::new();
        self.add_contract_subgraphs(calling_fn_name, tracked_contracts, &mut tracked_fns);

        // Iterate over function calls and create subgraphs + edges
        let private_functions_call_list = f.private_functions_calls();
        let external_call_list = f.external_functions_calls();
        let library_functions_call_list = f.library_functions_calls();
        self.create_subgraphs_and_edges(
            calling_fn_name,
            private_functions_call_list,
            tracked_contracts,
            &mut tracked_fns,
            compilation_unit,
            graph,
        );

        self.create_subgraphs_and_edges(
            calling_fn_name,
            external_call_list,
            tracked_contracts,
            &mut tracked_fns,
            compilation_unit,
            graph,
        );
        self.create_subgraphs_and_edges(
            calling_fn_name,
            library_functions_call_list,
            tracked_contracts,
            &mut tracked_fns,
            compilation_unit,
            graph,
        );
    }
    fn create_subgraphs_and_edges<'a>(
        &self,
        calling_fn_name: &str,
        call_list: impl Iterator<Item = &'a SierraStatement>,
        tracked_contracts: &mut HashMap<String, Subgraph>,
        tracked_fns: &mut HashSet<String>,
        compilation_unit: &CompilationUnit,
        graph: &mut Graph,
    ) {
        // Track edges in order to avoid duplicates
        let mut tracked_edges: HashSet<String> = HashSet::new();
        for call in call_list {
            if let SierraStatement::Invocation(invoc) = call {
                let libfunc = compilation_unit
                    .registry()
                    .get_libfunc(&invoc.libfunc_id)
                    .expect("Library not found in core registry");
                if let CoreConcreteLibfunc::FunctionCall(f_called) = libfunc {
                    let func_name = f_called
                        .function
                        .id
                        .debug_name
                        .as_ref()
                        .unwrap()
                        .to_string();
                    if tracked_edges.contains(&func_name) {
                        continue;
                    }

                    self.add_contract_subgraphs(&func_name, tracked_contracts, tracked_fns);
                    graph.add_stmt(Stmt::Edge(edge!(node_id!(format!("\"{}\"",calling_fn_name)) => node_id!(format!("\"{}\"", &func_name)))));
                    tracked_edges.insert(func_name);
                }
            }
        }
    }
    fn add_contract_subgraphs(
        &self,
        func_name: &str,
        tracked_contracts: &mut HashMap<String, Subgraph>,
        tracked_fns: &mut HashSet<String>,
    ) {
        let (module_name, exact_func_name) = self.get_names(func_name);
        let formatted_fn_name = format!("\"{}\"", func_name);
        let function_node = node!(formatted_fn_name; attr!("color","blue"),attr!("shape","square"),attr!("label",&exact_func_name));
        let contract_subgraph = tracked_contracts.get(&module_name);
        // Update contract subgraph with new function, or generate subgraph if it doesn't exist yet
        match contract_subgraph {
            Some(subgraph) => {
                let mut new_subgraph = subgraph.clone();
                // Avoid adding a visited function to the subgraph again
                if !tracked_fns.contains(func_name) {
                    new_subgraph.stmts.push(Stmt::from(function_node));
                    tracked_fns.insert(func_name.to_string());
                    tracked_contracts.insert(module_name.clone(), new_subgraph);
                }
            }
            None => {
                let name = if let Some(file_name) = module_name.rsplit_once("::") {
                    file_name.1
                } else {
                    &module_name
                };
                let formatted_module_name = format!("\"{}\"", name);
                let cluster = format!("\"cluster_{}\"", &module_name);
                let stmt = subgraph!(cluster; function_node, attr!("label",formatted_module_name));
                tracked_fns.insert(func_name.to_string());
                tracked_contracts.insert(module_name.clone(), stmt);
            }
        }
    }

    // Helper to get node_id. Returns function name in quotes but module as raw name
    fn get_names(&self, func_name: &str) -> (String, String) {
        // Handle the case of generics
        if func_name.contains("<") {
            let original_name = func_name
                .rsplit_once("<")
                .unwrap()
                .0
                .strip_suffix("::")
                .unwrap();
            let (module_name, exact_func_name) = original_name.rsplit_once("::").unwrap();
            // Leave module name w/o quotes, we'll modify it when computing the subgraph name
            (
                format!("{}", &module_name),
                format!("\"{}\"", &exact_func_name),
            )
        } else {
            let (module_name, exact_func_name) = func_name.rsplit_once("::").unwrap();
            (
                format!("{}", &module_name),
                format!("\"{}\"", &exact_func_name),
            )
        }
    }
}
