use std::collections::HashSet;

use super::function::{Function, Type};
use cairo_lang_sierra::extensions::core::{CoreLibfunc, CoreType};
use cairo_lang_sierra::program_registry::ProgramRegistry;

use cairo_lang_sierra::program::{
    Function as SierraFunction, Program, Statement as SierraStatement,
};
use cairo_lang_starknet::abi::{
    Contract,
    Item::{Event, Function as AbiFunction},
};

pub struct CompilationUnit<'a> {
    sierra_program: &'a Program,
    functions: Vec<Function<'a>>,
    abi: Contract,
    registry: ProgramRegistry<CoreType, CoreLibfunc>,
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
        }
    }

    /// Returns all the functions in the Sierra program
    pub fn functions(&self) -> impl Iterator<Item = &Function> {
        self.functions.iter()
    }

    /// Returns the functions that are defined by the user
    /// Constructor - External - View - Private
    pub fn functions_user_defined(&self) -> impl Iterator<Item = &Function> {
        self.functions.iter().filter(|f| match f.ty() {
            Type::Constructor | Type::External | Type::View | Type::Private => true,
            _ => false,
        })
    }

    pub fn registry(&self) -> &ProgramRegistry<CoreType, CoreLibfunc> {
        &self.registry
    }

    fn append_function(&mut self, data: &'a SierraFunction, statements: Vec<SierraStatement>) {
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
                function,
                self.sierra_program.statements
                    [function.entry_point.0..self.sierra_program.statements.len()]
                    .to_vec(),
            );
        } else {
            while let Some(funcs) = funcs_chunks.next() {
                if funcs_chunks.peek().is_some() {
                    self.append_function(
                        &funcs[0],
                        self.sierra_program.statements
                            [funcs[0].entry_point.0..funcs[1].entry_point.0]
                            .to_vec(),
                    );
                } else {
                    // Last pair
                    self.append_function(
                        &funcs[0],
                        self.sierra_program.statements
                            [funcs[0].entry_point.0..funcs[1].entry_point.0]
                            .to_vec(),
                    );
                    self.append_function(
                        &funcs[1],
                        self.sierra_program.statements
                            [funcs[1].entry_point.0..self.sierra_program.statements.len()]
                            .to_vec(),
                    );
                }
            }
        }

        self.set_functions_type();
        // Analyze each function
        self.functions.iter_mut().for_each(|f| f.analyze());
    }
}
