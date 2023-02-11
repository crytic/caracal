use std::collections::{HashMap, HashSet};

use super::function::{Function, Type};
use crate::analysis::taint::Taint;
use crate::analysis::taint::WrapperVariable;
use cairo_lang_sierra::extensions::core::{CoreConcreteLibfunc, CoreLibfunc, CoreType};
use cairo_lang_sierra::ids::VarId;
use cairo_lang_sierra::program::{
    Function as SierraFunction, GenStatement, Program, Statement as SierraStatement,
};
use cairo_lang_sierra::program_registry::ProgramRegistry;
use cairo_lang_starknet::abi::{
    Contract,
    Item::{Event, Function as AbiFunction},
};

pub struct CompilationUnit<'a> {
    /// The compiled sierra program
    sierra_program: &'a Program,
    /// Functions of the program
    functions: Vec<Function>,
    /// Abi of the compiled starknet contract
    abi: Contract,
    /// Helper registry to get the concrete type from an id
    registry: ProgramRegistry<CoreType, CoreLibfunc>,
    /// Function name to taints
    taint: HashMap<String, Taint>,
}

impl<'a> CompilationUnit<'a> {
    pub fn new(
        sierra_program: &'a Program,
        abi: Contract,
        registry: ProgramRegistry<CoreType, CoreLibfunc>,
    ) -> Self {
        CompilationUnit {
            sierra_program,
            functions: Vec::new(),
            abi,
            registry,
            taint: HashMap::new(),
        }
    }

    /// Returns all the functions in the Sierra program
    pub fn functions(&self) -> impl Iterator<Item = &Function> {
        self.functions.iter()
    }

    /// Returns the functions that are defined by the user
    /// Constructor - External - View - Private
    pub fn functions_user_defined(&self) -> impl Iterator<Item = &Function> {
        self.functions.iter().filter(|f| {
            matches!(
                f.ty(),
                Type::Constructor | Type::External | Type::View | Type::Private
            )
        })
    }

    pub fn registry(&self) -> &ProgramRegistry<CoreType, CoreLibfunc> {
        &self.registry
    }

    /// Return true if the variable is tainted i.e. user inputs can control it in some way
    pub fn is_tainted(&self, function_name: String, variable: VarId) -> bool {
        let wrapped_variable = WrapperVariable::new(function_name, variable);
        let mut parameters = HashSet::new();
        for external_function in self.functions.iter().filter(|f| *f.ty() == Type::External) {
            for param in external_function.params() {
                parameters.insert(WrapperVariable::new(
                    external_function.name(),
                    param.id.clone(),
                ));
            }
        }
        // Get the taint for the function where the variable appear
        let taint = self.taint.get(wrapped_variable.function()).unwrap();
        if taint.taints_any_sources(&parameters, &wrapped_variable) {
            return true;
        }

        false
    }

    fn append_function(&mut self, data: SierraFunction, statements: Vec<SierraStatement>) {
        self.functions.push(Function::new(data, statements));
    }

    fn set_functions_type(&mut self) {
        // Get a wrapper function and then get the base module from it
        let wrapper_function = self
            .sierra_program
            .funcs
            .iter()
            .find(|f| f.id.to_string().contains("__external::"));
        if let Some(f) = wrapper_function {
            let base_module =
                f.id.to_string()
                    .split_once("__external::")
                    .unwrap()
                    .0
                    .to_string();
            let mut external_functions = HashSet::new();
            let mut constructor = String::new();

            // Gather all the external functions and the constructor
            for f in self.sierra_program.funcs.iter() {
                let full_name = f.id.to_string();
                if full_name.contains("::__external::") {
                    external_functions.insert(full_name.replace("__external::", ""));
                } else if full_name.contains("__constructor") {
                    constructor = full_name.replace("__constructor::", "");
                }
            }

            // Set the function type
            for f in self.functions.iter_mut() {
                let full_name = f.name();

                if full_name.starts_with("core::") {
                    f.set_ty(Type::Core);
                } else if full_name.contains("::__external::")
                    || full_name.contains("::__constructor::")
                {
                    f.set_ty(Type::Wrapper);
                } else if full_name == constructor {
                    // Constructor
                    f.set_ty(Type::Constructor);
                } else if external_functions.contains(&full_name) {
                    // External function, we need to check in the abi if it's view or external
                    let function_name = full_name.rsplit_once("::").unwrap().1;

                    for item in self.abi.items.iter() {
                        if let AbiFunction(function) = item {
                            if function.name == function_name {
                                match function.state_mutability {
                                    cairo_lang_starknet::abi::StateMutability::External => {
                                        f.set_ty(Type::External);
                                        break;
                                    }
                                    cairo_lang_starknet::abi::StateMutability::View => {
                                        f.set_ty(Type::View);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                } else if full_name.ends_with("::address")
                    || full_name.ends_with("::read")
                    || full_name.ends_with("::write")
                {
                    // Storage variable functions e.g. erc20::erc20::ERC20::name::read
                    // Safe unwrap at this point it must start with the base module
                    let second_part = full_name
                        .split_once(&base_module)
                        .unwrap()
                        .1
                        .split("::")
                        .collect::<Vec<&str>>();
                    // We assume it's a function for a storage variable
                    // however if there is an immediate submodule with a read/write/address function
                    // it will be incorrectly set as Storage
                    if second_part.len() == 2 {
                        f.set_ty(Type::Storage);
                    } else {
                        f.set_ty(Type::Private);
                    }
                } else {
                    // Event or private function
                    // Safe unwrap at this point it must start with the base module
                    let second_part = full_name.split_once(&base_module).unwrap().1;
                    // If it contains :: it means it's a function in a submodule so it should be private
                    if second_part.contains("::") {
                        f.set_ty(Type::Private);
                    } else {
                        // Could be an event or a private function in the contract's module
                        let possible_event_name = full_name.rsplit_once("::").unwrap().1;

                        let mut found = false;
                        for item in self.abi.items.iter() {
                            if let Event(e) = item {
                                if e.name == possible_event_name {
                                    f.set_ty(Type::Event);
                                    found = true;
                                    break;
                                }
                            }
                        }
                        if !found {
                            f.set_ty(Type::Private);
                        }
                    }
                }
            }
        } else {
            // There aren't external functions could be a standard cairo file not a smart contract
            // set all of them to private but for now we don't handle standard cairo in the perfect way
            self.functions
                .iter_mut()
                .for_each(|f| f.set_ty(Type::Private));
        }
    }

    pub fn analyze(&mut self) {
        // Add the functions in the sierra program
        let mut funcs_chunks = self.sierra_program.funcs.windows(2).peekable();

        // There is only 1 function
        if funcs_chunks.peek().is_none() {
            let function = &self.sierra_program.funcs[0];

            self.append_function(
                function.clone(),
                self.sierra_program.statements
                    [function.entry_point.0..self.sierra_program.statements.len()]
                    .to_vec(),
            );
        } else {
            while let Some(funcs) = funcs_chunks.next() {
                if funcs_chunks.peek().is_some() {
                    self.append_function(
                        funcs[0].clone(),
                        self.sierra_program.statements
                            [funcs[0].entry_point.0..funcs[1].entry_point.0]
                            .to_vec(),
                    );
                } else {
                    // Last pair
                    self.append_function(
                        funcs[0].clone(),
                        self.sierra_program.statements
                            [funcs[0].entry_point.0..funcs[1].entry_point.0]
                            .to_vec(),
                    );
                    self.append_function(
                        funcs[1].clone(),
                        self.sierra_program.statements
                            [funcs[1].entry_point.0..self.sierra_program.statements.len()]
                            .to_vec(),
                    );
                }
            }
        }

        self.set_functions_type();

        let mut functions_copy = Vec::with_capacity(self.functions.len());
        functions_copy.clone_from(&self.functions);

        // Analyze each function
        self.functions
            .iter_mut()
            .for_each(|f| f.analyze(&functions_copy, &self.registry));

        // Compute taints
        self.functions.iter().for_each(|f| {
            self.taint
                .insert(f.name(), Taint::new(f.get_statements(), f.name()));
        });

        // Propagate taints to private functions
        self.propagate_taints();
    }

    fn propagate_taints(&mut self) {
        // 1-per ogni external function itera private_functions_calls
        // 2-check se ext function args taint parametri usati in privat function call - taints_any_sinks_variable metti in sinks tutti i param usati in privat call
        // 3-propaga

        let mut arguments_external_functions: HashSet<WrapperVariable> = HashSet::new();
        for function in self.functions.iter().filter(|f| *f.ty() == Type::External) {
            for param in function.params() {
                arguments_external_functions
                    .insert(WrapperVariable::new(function.name(), param.id.clone()));
            }
        }

        let mut changed = true;
        while changed {
            for calling_function in self
                .functions
                .iter()
                .filter(|f| *f.ty() == Type::External || *f.ty() == Type::Private)
            {
                changed = false;
                for function_call in calling_function.private_functions_calls() {
                    // It will always be an invocation
                    if let GenStatement::Invocation(invoc) = function_call {
                        // The core lib func instance
                        let lib_func = self
                            .registry
                            .get_libfunc(&invoc.libfunc_id)
                            .expect("Library function not found in the registry");

                        // This is always true since private_function_calls contain only FunctionCall statement
                        if let CoreConcreteLibfunc::FunctionCall(f_called) = lib_func {
                            let taint_copy = self.taint.clone();
                            let external_taint = taint_copy.get(&calling_function.name()).unwrap();

                            // Variables used as arguments in the call to the private function
                            let function_called_args: HashSet<WrapperVariable> = invoc
                                .args
                                .iter()
                                .map(|arg| {
                                    WrapperVariable::new(calling_function.name(), arg.clone())
                                })
                                .collect();

                            // Calling function's parameters
                            for param in calling_function.params() {
                                // Check if the arguments used to call the private function are tainted by the calling function's parameters
                                for sink in external_taint.taints_any_sinks_variable(
                                    &WrapperVariable::new(
                                        calling_function.name(),
                                        param.id.clone(),
                                    ),
                                    &function_called_args,
                                ) {
                                    // If the sink is tainted by some parameters of external functions
                                    // then we need to add those parameters as source for the current sink
                                    for source in external_taint.taints_any_sources_variable(
                                        &arguments_external_functions,
                                        &sink,
                                    ) {
                                        let function_called_name = f_called
                                            .function
                                            .id
                                            .debug_name
                                            .as_ref()
                                            .unwrap()
                                            .to_string();

                                        let private_taint =
                                            self.taint.get_mut(&function_called_name).unwrap();

                                        // We convert the id to be the private function's formal parameter id and not the actual parameter id
                                        let sink_converted = WrapperVariable::new(
                                            function_called_name,
                                            VarId::new(sink.variable().id - invoc.args[0].id),
                                        );
                                        // Add the source i.e. the variable of the external function
                                        if private_taint.add_taint(source, sink_converted) {
                                            changed = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
