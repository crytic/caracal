use std::io::Write;

use super::cfg::{Cfg, CfgOptimized, CfgRegular};
use crate::utils::BUILTINS;
use cairo_lang_sierra::extensions::core::{CoreConcreteLibfunc, CoreLibfunc, CoreType};
use cairo_lang_sierra::ids::ConcreteTypeId;
use cairo_lang_sierra::program::{
    Function as SierraFunction, GenStatement, Param, Statement as SierraStatement,
};
use cairo_lang_sierra::program_registry::ProgramRegistry;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Type {
    /// External function defined by the user
    External,
    /// View function defined by the user
    View,
    /// Private function defined by the user
    Private,
    /// Constructor function defined by the user
    Constructor,
    /// Event function
    Event,
    /// Function made by the compiler for storage variables
    /// typically address, read, write
    Storage,
    /// Wrapper around an external function made by the compiler
    Wrapper,
    /// Function of the core library
    Core,
    /// Function of a trait with the ABI attribute that does a call contract
    AbiCallContract,
    /// Function of a trait with the ABI attribute that does a library call
    AbiLibraryCall,
    /// L1 handler function
    L1Handler,
}

#[derive(Clone)]
pub struct Function {
    /// Underlying Function data
    data: SierraFunction,
    /// Type of function
    ty: Option<Type>,
    /// The sequence of statements
    statements: Vec<SierraStatement>,
    /// A regular CFG from the statements
    cfg_regular: CfgRegular,
    /// An optimized CFG from the statements
    cfg_optimized: CfgOptimized,
    /// Storage variables read (NOTE it doesn't have vars read using the syscall directly)
    storage_vars_read: Vec<SierraStatement>,
    /// Storage variables written (NOTE it doesn't have vars written using the syscall directly)
    storage_vars_written: Vec<SierraStatement>,
    /// Core functions called
    core_functions_calls: Vec<SierraStatement>,
    /// Private functions called
    private_functions_calls: Vec<SierraStatement>,
    /// Events emitted (NOTE it doesn't have events emitted using the syscall directly)
    events_emitted: Vec<SierraStatement>,
    /// External functions called through an ABI trait (NOTE it doesn't have external functions called using the syscall directly)
    external_functions_calls: Vec<SierraStatement>,
    /// Library functions called through an ABI trait (NOTE it doesn't have library functions called using the syscall directly)
    library_functions_calls: Vec<SierraStatement>,
}

impl Function {
    pub fn new(data: SierraFunction, statements: Vec<SierraStatement>) -> Self {
        Function {
            data,
            ty: None,
            statements,
            cfg_regular: CfgRegular::new(),
            cfg_optimized: CfgOptimized::new(),
            storage_vars_read: Vec::new(),
            storage_vars_written: Vec::new(),
            core_functions_calls: Vec::new(),
            private_functions_calls: Vec::new(),
            events_emitted: Vec::new(),
            external_functions_calls: Vec::new(),
            library_functions_calls: Vec::new(),
        }
    }

    pub fn name(&self) -> String {
        self.data.id.to_string()
    }

    pub fn ty(&self) -> &Type {
        // At this point is always initialized
        self.ty.as_ref().unwrap()
    }

    pub fn storage_vars_read(&self) -> impl Iterator<Item = &SierraStatement> {
        self.storage_vars_read.iter()
    }

    pub fn storage_vars_written(&self) -> impl Iterator<Item = &SierraStatement> {
        self.storage_vars_written.iter()
    }

    pub fn core_functions_calls(&self) -> impl Iterator<Item = &SierraStatement> {
        self.core_functions_calls.iter()
    }

    pub fn private_functions_calls(&self) -> impl Iterator<Item = &SierraStatement> {
        self.private_functions_calls.iter()
    }

    pub fn events_emitted(&self) -> impl Iterator<Item = &SierraStatement> {
        self.events_emitted.iter()
    }

    pub fn external_functions_calls(&self) -> impl Iterator<Item = &SierraStatement> {
        self.external_functions_calls.iter()
    }

    pub fn library_functions_calls(&self) -> impl Iterator<Item = &SierraStatement> {
        self.library_functions_calls.iter()
    }

    /// Function return variables without the builtins
    pub fn returns(&self) -> impl Iterator<Item = &ConcreteTypeId> {
        self.data
            .signature
            .ret_types
            .iter()
            .filter(|r| !BUILTINS.contains(&r.debug_name.clone().unwrap().as_str()))
    }

    /// Function return variables
    pub fn returns_all(&self) -> impl Iterator<Item = &ConcreteTypeId> {
        self.data.signature.ret_types.iter()
    }

    /// Function parameters without the builtins
    pub fn params(&self) -> impl Iterator<Item = &Param> {
        self.data
            .params
            .iter()
            .filter(|p| !BUILTINS.contains(&p.ty.debug_name.clone().unwrap().as_str()))
    }

    /// Function parameters
    pub fn params_all(&self) -> impl Iterator<Item = &Param> {
        self.data.params.iter()
    }

    pub fn get_statements(&self) -> &Vec<SierraStatement> {
        &self.statements
    }

    pub fn get_statements_at(&self, at: usize) -> &[SierraStatement] {
        &self.statements[at..]
    }

    pub fn get_cfg(&self) -> &CfgRegular {
        &self.cfg_regular
    }

    pub fn get_cfg_optimized(&self) -> &CfgOptimized {
        &self.cfg_optimized
    }

    pub fn analyze(
        &mut self,
        functions: &[Function],
        registry: &ProgramRegistry<CoreType, CoreLibfunc>,
    ) {
        self.cfg_regular
            .analyze(&self.statements, self.data.entry_point.0);
        self.cfg_optimized
            .analyze(self.cfg_regular.get_basic_blocks().to_vec());
        self.set_meta_informations(functions, registry);
    }

    /// Set the meta informations such as storage variables read, storage variables written, core function called
    /// private function called, events emitted
    fn set_meta_informations(
        &mut self,
        functions: &[Function],
        registry: &ProgramRegistry<CoreType, CoreLibfunc>,
    ) {
        for s in self.statements.iter() {
            if let GenStatement::Invocation(invoc) = s {
                let lib_func = registry
                    .get_libfunc(&invoc.libfunc_id)
                    .expect("Library function not found in the registry");
                if let CoreConcreteLibfunc::FunctionCall(f_called) = lib_func {
                    // We search for the function called in our list of functions to know its type
                    for function in functions.iter() {
                        let function_name = function.name();
                        if function_name.as_str()
                            == f_called.function.id.debug_name.as_ref().unwrap()
                        {
                            match function.ty() {
                                Type::Storage => {
                                    if function_name.ends_with("read") {
                                        self.storage_vars_read.push(s.clone());
                                    } else if function_name.ends_with("write") {
                                        self.storage_vars_written.push(s.clone());
                                    }
                                }
                                Type::Event => self.events_emitted.push(s.clone()),
                                Type::Core => self.core_functions_calls.push(s.clone()),
                                Type::Private => self.private_functions_calls.push(s.clone()),
                                Type::AbiCallContract => {
                                    self.external_functions_calls.push(s.clone())
                                }
                                Type::AbiLibraryCall => {
                                    self.library_functions_calls.push(s.clone())
                                }
                                _ => (),
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    pub(super) fn set_ty(&mut self, ty: Type) {
        self.ty = Some(ty);
    }

    /// Write to a file the function's CFG and return the filename
    pub fn cfg_to_dot(&self, cfg: &dyn Cfg) -> String {
        // name for now good enough
        let file_name = format!(
            "{}.dot",
            self.name()
                .split('<')
                .take(1)
                .next()
                .expect("Error when creating the filename")
        )
        .replace("::", "_");

        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&file_name)
            .expect("Error when creating file");

        f.write_all(b"digraph{\n").unwrap();

        for bb in cfg.get_basic_blocks() {
            let mut ins = String::new();
            bb.get_instructions()
                .iter()
                .for_each(|i| ins.push_str(&format!("{i}\n")));
            f.write_all(
                format!("{}[label=\"BB {}\n{}\"]\n", bb.get_id(), bb.get_id(), ins).as_bytes(),
            )
            .unwrap();

            for destination in bb.get_outgoing_basic_blocks().iter() {
                f.write_all(format!("{} -> {}\n", bb.get_id(), destination).as_bytes())
                    .unwrap();
            }
        }

        f.write_all(b"}\n").unwrap();

        file_name
    }
}
