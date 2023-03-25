use cairo_lang_sierra::extensions::lib_func::ParamSignature;
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
