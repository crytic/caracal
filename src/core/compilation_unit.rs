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
    Item::Function as AbiFunction,
};

pub struct CompilationUnit {
    /// The compiled sierra program
    sierra_program: Program,
    /// Functions of the program
    functions: Vec<Function>,
    /// Abi of the compiled starknet contracts
    abi: Contract,
    /// Helper registry to get the concrete type from an id
    registry: ProgramRegistry<CoreType, CoreLibfunc>,
    /// Function name to taints
    taint: HashMap<String, Taint>,
}

impl CompilationUnit {
    pub fn new(
        sierra_program: Program,
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
    /// Constructor - External - View - Private - L1Handler
    pub fn functions_user_defined(&self) -> impl Iterator<Item = &Function> {
        self.functions.iter().filter(|f| {
            matches!(
                f.ty(),
                Type::Constructor | Type::External | Type::View | Type::Private | Type::L1Handler
            )
        })
    }

    pub fn function_by_name(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|f| f.name().as_str() == name)
    }

    pub fn registry(&self) -> &ProgramRegistry<CoreType, CoreLibfunc> {
        &self.registry
    }

    /// Return true if the variable is tainted i.e. user inputs can control it in some way
    pub fn is_tainted(&self, function_name: String, variable: VarId) -> bool {
        let wrapped_variable = WrapperVariable::new(function_name, variable);
        let mut parameters = HashSet::new();
        for external_function in self
            .functions
            .iter()
            .filter(|f| matches!(f.ty(), Type::External | Type::L1Handler))
        {
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

    /// Return the function_name's Taint if exist.
    /// This can be useful to access to low level taint functions present in Taint
    /// compared to the more general is_tainted
    pub fn get_taint(&self, function_name: &str) -> Option<&Taint> {
        self.taint.get(function_name)
    }

    fn append_function(&mut self, data: SierraFunction, statements: Vec<SierraStatement>) {
        // The compiler adds unsafe_new_contract_state which holds the storage variables
        // for now we don't consider it
        if !data.id.to_string().ends_with("::unsafe_new_contract_state") {
            self.functions.push(Function::new(data, statements));
        }
    }

    fn set_functions_type(&mut self) {
        let mut external_functions = HashSet::new();
        let mut constructors = HashSet::new();
        let mut l1_handler_functions = HashSet::new();

        // Gather all the external/l1_handler functions and the constructor of each contract
        for f in self.sierra_program.funcs.iter() {
            let full_name = f.id.to_string();
            if full_name.contains("::__external::") {
                external_functions.insert(full_name.replace("__external::", ""));
            } else if full_name.contains("::__constructor::") {
                constructors.insert(full_name.replace("__constructor::", ""));
            } else if full_name.contains("::__l1_handler::") {
                l1_handler_functions.insert(full_name.replace("__l1_handler::", ""));
            }
        }

        // Set the function type
        for f in self.functions.iter_mut() {
            let full_name = f.name();

            if full_name.starts_with("core::") {
                f.set_ty(Type::Core);
            } else if full_name.contains("::__external::")
                || full_name.contains("::__constructor::")
                || full_name.contains("::__l1_handler::")
            {
                f.set_ty(Type::Wrapper);
            } else if constructors.contains(&full_name) {
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
            } else if l1_handler_functions.contains(&full_name) {
                f.set_ty(Type::L1Handler);
            } else if full_name.ends_with("::address")
                || full_name.ends_with("::read")
                || full_name.ends_with("::write")
            {
                // A user defined function named address/read/write can be incorrectly set to Storage
                f.set_ty(Type::Storage);
            // ABI trait function for library call
            } else if full_name.contains("LibraryDispatcherImpl::") {
                f.set_ty(Type::AbiLibraryCall)
            // ABI trait function for call contract
            } else if full_name.contains("DispatcherImpl::") {
                f.set_ty(Type::AbiCallContract)
            } else {
                // Event or private function
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

    /// Analyze the Sierra program and set the internal data structure
    /// such as create the functions with the corresponding statements
    pub fn analyze(&mut self) {
        // Add the functions in the sierra program
        let funcs = self.sierra_program.funcs.clone();
        let mut funcs_chunks = funcs.windows(2).peekable();

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

        // Analyze each function
        let mut functions = Vec::with_capacity(self.functions.len());
        functions.clone_from(&self.functions);
        self.functions
            .iter_mut()
            .for_each(|f| f.analyze(&functions, &self.registry));

        // Run analyses on each function after all the functions have been analyzed
        functions.clone_from(&self.functions);
        self.functions
            .iter_mut()
            .for_each(|f| f.run_analyses(&functions, &self.registry));

        // Compute taints
        self.functions.iter().for_each(|f| {
            self.taint
                .insert(f.name(), Taint::new(f.get_statements(), f.name()));
        });

        // Propagate taints to private functions
        self.propagate_taints();
    }

    /// Propagate the taints from external/l1_handler functions to private functions
    fn propagate_taints(&mut self) {
        // Collect the arguments of all the external/l1_handler functions
        let mut arguments_external_functions: HashSet<WrapperVariable> = HashSet::new();
        for function in self
            .functions
            .iter()
            .filter(|f| matches!(f.ty(), Type::External | Type::L1Handler))
        {
            for param in function.params() {
                arguments_external_functions
                    .insert(WrapperVariable::new(function.name(), param.id.clone()));
            }
        }

        // There aren't external functions we don't need to propagate anything
        if arguments_external_functions.is_empty() {
            return;
        }

        let mut changed = true;
        // Iterate external, l1_handler, private functions and propagate the taints to each private function they call
        // until a fixpoint when no new informations were propagated
        while changed {
            changed = false;

            for calling_function in self
                .functions
                .iter()
                .filter(|f| matches!(f.ty(), Type::External | Type::L1Handler | Type::Private))
            {
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
