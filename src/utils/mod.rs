use cairo_lang_sierra::extensions::lib_func::{OutputVarInfo, ParamSignature};
use cairo_lang_sierra::ids::VarId;

pub const BUILTINS: [&str; 7] = [
    "Pedersen",
    "RangeCheck",
    "Bitwise",
    "EcOp",
    "SegmentArena",
    "GasBuiltin",
    "System",
];

/// Filter the builtins arguments and returns only the user defined arguments
pub fn filter_builtins_from_arguments(
    signature: &[ParamSignature],
    arguments: Vec<VarId>,
) -> Vec<VarId> {
    signature
        .iter()
        .zip(arguments)
        .filter(|(sig_elem, _)| {
            !BUILTINS.contains(&sig_elem.ty.debug_name.clone().unwrap().as_str())
        })
        .map(|(_, arg_elem)| arg_elem)
        .collect()
}

/// Filter the builtins from the return variables and returns only the user defined variables
pub fn filter_builtins_from_returns(
    signature: &[OutputVarInfo],
    returns: Vec<VarId>,
) -> Vec<VarId> {
    signature
        .iter()
        .zip(returns)
        .filter(|(sig_elem, _)| {
            !BUILTINS.contains(&sig_elem.ty.debug_name.clone().unwrap().as_str())
        })
        .map(|(_, arg_elem)| arg_elem)
        .collect()
}

/// Get a number as input and return the ordinal representation
pub fn number_to_ordinal(n: u64) -> String {
    let s = n.to_string();
    if s.ends_with('1') && !s.ends_with("11") {
        format!("{}{}", n, "st")
    } else if s.ends_with('2') && !s.ends_with("12") {
        format!("{}{}", n, "nd")
    } else if s.ends_with('3') && !s.ends_with("13") {
        format!("{}{}", n, "rd")
    } else {
        format!("{}{}", n, "th")
    }
}
