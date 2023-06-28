use crate::core::core_unit::CoreUnit;
use std::fmt;
use std::str::FromStr;

pub trait Printer {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn run(&self, core: &CoreUnit, opts: PrintOpts) -> Vec<Result>;
}

impl fmt::Display for dyn Printer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} | {}", self.name(), self.description())
    }
}

#[derive(Debug)]
pub struct Result {
    pub name: String,
    pub message: String,
}

impl fmt::Display for Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Options passed to the printer in run()
pub struct PrintOpts {
    pub filter: Filter,
}

#[derive(Debug, Clone, Copy)]
pub enum Filter {
    /// All the functions in the program (core library functions, wrapper functions...)
    All,
    /// Only user defined functions
    UserFunctions,
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Filter::All => "all",
            Filter::UserFunctions => "user-functions",
        };
        write!(f, "{string}")
    }
}

impl FromStr for Filter {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "all" => Ok(Filter::All),
            "user-functions" => Ok(Filter::UserFunctions),
            s => Err(format!("Unknown filter: {s}")),
        }
    }
}
