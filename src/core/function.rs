use std::io::Write;

use super::cfg::{Cfg, CfgOptimized, CfgRegular};
use cairo_lang_sierra::ids::ConcreteTypeId;
use cairo_lang_sierra::program::{Function as SierraFunction, Param, Statement as SierraStatement};

#[derive(Debug)]
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
}

#[derive(Debug)]
pub struct Function<'a> {
    /// Underlying Function data
    data: &'a SierraFunction,
    /// Type of function
    ty: Option<Type>,
    /// The sequence of statements
    statements: Vec<SierraStatement>,
    /// A regular CFG from the statements
    cfg_regular: CfgRegular,
    /// An optimized CFG from the statements
    cfg_optimized: CfgOptimized,
}

impl<'a> Function<'a> {
    pub fn new(data: &'a SierraFunction, statements: Vec<SierraStatement>) -> Self {
        Function {
            data,
            ty: None,
            statements,
            cfg_regular: CfgRegular::new(),
            cfg_optimized: CfgOptimized::new(),
        }
    }

    pub fn name(&self) -> String {
        self.data.id.to_string()
    }

    pub fn ty(&self) -> &Type {
        // At this point is always initialized
        self.ty.as_ref().unwrap()
    }

    pub fn returns(&self) -> impl Iterator<Item = &ConcreteTypeId> {
        self.data.signature.ret_types.iter()
    }

    pub fn params(&self) -> impl Iterator<Item = &Param> {
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

    pub fn analyze(&mut self) {
        self.cfg_regular
            .analyze(&self.statements, self.data.entry_point.0);
        self.cfg_optimized
            .analyze(self.cfg_regular.get_basic_blocks().to_vec());
    }

    pub(super) fn set_ty(&mut self, ty: Type) {
        self.ty = Some(ty);
    }

    pub fn cfg_to_dot(&self, cfg: &dyn Cfg) {
        // name for now good enough
        let name = format!(
            "{}.dot",
            self.name()
                .split('<')
                .take(1)
                .next()
                .expect("Error when creating the filename")
        )
        .replace("::", "_");
        println!("FILENAME {name}");
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(name)
            .expect("Error when creating file");
        f.write_all(b"digraph{\n").unwrap();
        for bb in cfg.get_basic_blocks() {
            let mut ins = String::new();
            bb.get_instructions()
                .iter()
                .for_each(|i| ins.push_str(&format!("{i}\n")));
            f.write_all(format!("{}[label=\"{}\"]\n", bb.get_id(), ins).as_bytes())
                .unwrap();

            for destination in bb.get_outgoing_basic_blocks().iter() {
                f.write_all(format!("{} -> {}\n", bb.get_id(), destination).as_bytes())
                    .unwrap();
            }
        }
        f.write_all(b"}\n").unwrap();
    }
}
