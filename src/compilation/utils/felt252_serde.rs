// Taken from https://github.com/starkware-libs/cairo/blob/77ae28f996c0960ce5cfc926703f60bae8d5db5a/crates/cairo-lang-starknet/src/felt252_serde.rs
// Removed the serialization process

use cairo_lang_sierra::extensions::starknet::interoperability::ContractAddressTryFromFelt252Libfunc;
use cairo_lang_sierra::extensions::starknet::storage::{
    StorageAddressFromBaseAndOffsetLibfunc, StorageAddressTryFromFelt252Trait,
    StorageBaseAddressFromFelt252Libfunc,
};
use cairo_lang_sierra::extensions::try_from_felt252::TryFromFelt252;
use cairo_lang_sierra::extensions::NamedLibfunc;
use cairo_lang_sierra::ids::{
    ConcreteLibfuncId, ConcreteTypeId, FunctionId, GenericLibfuncId, GenericTypeId, UserTypeId,
    VarId,
};
use cairo_lang_sierra::program::{
    BranchInfo, BranchTarget, ConcreteLibfuncLongId, ConcreteTypeLongId, Function,
    FunctionSignature, GenericArg, Invocation, LibfuncDeclaration, Param, Program, Statement,
    StatementIdx, TypeDeclaration,
};
use cairo_lang_utils::bigint::BigUintAsHex;
use cairo_lang_utils::ordered_hash_set::OrderedHashSet;
use cairo_lang_utils::unordered_hash_map::UnorderedHashMap;
use num_bigint::{BigInt, BigUint, ToBigInt};
use num_traits::ToPrimitive;
use once_cell::sync::Lazy;
use smol_str::SmolStr;

use crate::compilation::utils::felt252_vec_compression::decompress;
use cairo_lang_starknet::contract::starknet_keccak;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VersionId {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Felt252SerdeError {
    InvalidInputForDeserialization,
}

/// Deserializes a Sierra program from a slice of felt252s.
///
/// Returns (sierra_version_id, compiler_version_id, program).
/// See [crate::compiler_version].
pub fn sierra_from_felt252s(
    felts: &[BigUintAsHex],
) -> Result<(VersionId, VersionId, Program), Felt252SerdeError> {
    let (sierra_version_id, remaining) = VersionId::deserialize(felts)?;
    let (compiler_version_id, remaining) = VersionId::deserialize(remaining)?;
    let mut program_felts = vec![];
    decompress(remaining, &mut program_felts)
        .ok_or(Felt252SerdeError::InvalidInputForDeserialization)?;
    Ok((
        sierra_version_id,
        compiler_version_id,
        Program::deserialize(&program_felts)?.0,
    ))
}

/// Trait for serializing and deserializing into a felt252 vector.
trait Felt252Serde: Sized {
    fn deserialize(input: &[BigUintAsHex]) -> Result<(Self, &[BigUintAsHex]), Felt252SerdeError>;
}

// Impls for basic types.

impl Felt252Serde for usize {
    fn deserialize(input: &[BigUintAsHex]) -> Result<(Self, &[BigUintAsHex]), Felt252SerdeError> {
        let head = input
            .first()
            .and_then(|size| size.value.to_usize())
            .ok_or(Felt252SerdeError::InvalidInputForDeserialization)?;
        Ok((head, &input[1..]))
    }
}

impl Felt252Serde for u64 {
    fn deserialize(input: &[BigUintAsHex]) -> Result<(Self, &[BigUintAsHex]), Felt252SerdeError> {
        let head = input
            .first()
            .and_then(|size| size.value.to_u64())
            .ok_or(Felt252SerdeError::InvalidInputForDeserialization)?;
        Ok((head, &input[1..]))
    }
}

impl<T: Felt252Serde> Felt252Serde for Vec<T> {
    fn deserialize(input: &[BigUintAsHex]) -> Result<(Self, &[BigUintAsHex]), Felt252SerdeError> {
        let (size, mut input) = usize::deserialize(input)?;
        let mut result = Vec::with_capacity(size);
        for _ in 0..size {
            let (value, next) = T::deserialize(input)?;
            result.push(value);
            input = next;
        }
        Ok((result, input))
    }
}

impl Felt252Serde for BigInt {
    fn deserialize(input: &[BigUintAsHex]) -> Result<(Self, &[BigUintAsHex]), Felt252SerdeError> {
        let first = input
            .first()
            .ok_or(Felt252SerdeError::InvalidInputForDeserialization)?;
        Ok((
            first
                .value
                .to_bigint()
                .expect("Unsigned should always be convertible to signed."),
            &input[1..],
        ))
    }
}

impl Felt252Serde for StatementIdx {
    fn deserialize(input: &[BigUintAsHex]) -> Result<(Self, &[BigUintAsHex]), Felt252SerdeError> {
        let (value, input) = usize::deserialize(input)?;
        Ok((Self(value), input))
    }
}

/// A set of all the supported long generic ids.
static SERDE_SUPPORTED_LONG_IDS: Lazy<OrderedHashSet<&'static str>> = Lazy::new(|| {
    OrderedHashSet::from_iter(
        [
            StorageAddressFromBaseAndOffsetLibfunc::STR_ID,
            ContractAddressTryFromFelt252Libfunc::STR_ID,
            StorageBaseAddressFromFelt252Libfunc::STR_ID,
            StorageAddressTryFromFelt252Trait::STR_ID,
        ]
        .into_iter(),
    )
});
/// A mapping of all the long names when fixing them from the hashed keccak representation.
static LONG_NAME_FIX: Lazy<UnorderedHashMap<BigUint, &'static str>> = Lazy::new(|| {
    UnorderedHashMap::from_iter(
        SERDE_SUPPORTED_LONG_IDS
            .iter()
            .map(|name| (starknet_keccak(name.as_bytes()), *name)),
    )
});

macro_rules! generic_id_serde {
    ($Obj:ident) => {
        impl Felt252Serde for $Obj {
            fn deserialize(
                input: &[BigUintAsHex],
            ) -> Result<(Self, &[BigUintAsHex]), Felt252SerdeError> {
                let head = input
                    .first()
                    .and_then(|id| {
                        LONG_NAME_FIX
                            .get(&id.value)
                            .map(|s| Self(SmolStr::new(s)))
                            .or_else(|| {
                                std::str::from_utf8(&id.value.to_bytes_be())
                                    .ok()
                                    .map(|s| Self(s.into()))
                            })
                    })
                    .ok_or(Felt252SerdeError::InvalidInputForDeserialization)?;
                Ok((head, &input[1..]))
            }
        }
    };
}

generic_id_serde!(GenericTypeId);
generic_id_serde!(GenericLibfuncId);

impl Felt252Serde for UserTypeId {
    fn deserialize(input: &[BigUintAsHex]) -> Result<(Self, &[BigUintAsHex]), Felt252SerdeError> {
        let first = input
            .first()
            .ok_or(Felt252SerdeError::InvalidInputForDeserialization)?;
        Ok((
            Self {
                id: first.value.clone(),
                debug_name: None,
            },
            &input[1..],
        ))
    }
}

// Impls for other ids.

macro_rules! id_serde {
    ($Obj:ident) => {
        impl Felt252Serde for $Obj {
            fn deserialize(
                input: &[BigUintAsHex],
            ) -> Result<(Self, &[BigUintAsHex]), Felt252SerdeError> {
                let (id, input) = u64::deserialize(input)?;
                Ok((Self::new(id), input))
            }
        }
    };
}

id_serde!(ConcreteTypeId);
id_serde!(ConcreteLibfuncId);
id_serde!(VarId);
id_serde!(FunctionId);

// Impls for structs.

macro_rules! struct_deserialize_impl {
    ($input:ident, { $($field_name:ident : $field_type:ty),* }) => {
        let __input = $input;
        $(
            let ($field_name, __input) = <$field_type>::deserialize(__input)?;
        )*
        $input = __input;
    };
}

macro_rules! struct_deserialize {
    ($Obj:ident { $($field_name:ident : $field_type:ty),* }) => {
        fn deserialize(
            mut input: &[BigUintAsHex],
        ) -> Result<(Self, &[BigUintAsHex]), Felt252SerdeError> {
            struct_deserialize_impl!(input, {$($field_name : $field_type),*});
            Ok((Self {$($field_name),*}, input))
        }
    };
}

macro_rules! struct_serde {
    ($Obj:ident { $($field_name:ident : $field_type:ty),* $(,)? }) => {
        impl Felt252Serde for $Obj {
            struct_deserialize! { $Obj { $($field_name : $field_type),* } }
        }
    }
}

impl Felt252Serde for Program {
    fn deserialize(input: &[BigUintAsHex]) -> Result<(Self, &[BigUintAsHex]), Felt252SerdeError> {
        // Type declarations.
        let (size, mut input) = usize::deserialize(input)?;
        let mut type_declarations = Vec::with_capacity(size);
        for i in 0..size {
            let (long_id, next) = ConcreteTypeLongId::deserialize(input)?;
            type_declarations.push(TypeDeclaration {
                id: ConcreteTypeId::new(i as u64),
                long_id,
                declared_type_info: None,
            });
            input = next;
        }
        // Libfunc declaration.
        let (size, mut input) = usize::deserialize(input)?;
        let mut libfunc_declarations = Vec::with_capacity(size);
        for i in 0..size {
            let (long_id, next) = ConcreteLibfuncLongId::deserialize(input)?;
            libfunc_declarations.push(LibfuncDeclaration {
                id: ConcreteLibfuncId::new(i as u64),
                long_id,
            });
            input = next;
        }
        // Statements.
        let (statements, input) = Felt252Serde::deserialize(input)?;
        // Function declaration.
        let (size, mut input) = usize::deserialize(input)?;
        let mut funcs = Vec::with_capacity(size);
        for i in 0..size {
            let (signature, next) = FunctionSignature::deserialize(input)?;
            input = next;
            let params = signature
                .param_types
                .iter()
                .cloned()
                .map(|ty| -> Result<Param, Felt252SerdeError> {
                    let (id, next) = VarId::deserialize(input)?;
                    input = next;
                    Ok(Param { id, ty })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let (entry_point, next) = StatementIdx::deserialize(input)?;
            funcs.push(Function {
                id: FunctionId::new(i as u64),
                signature,
                params,
                entry_point,
            });
            input = next;
        }
        Ok((
            Self {
                type_declarations,
                libfunc_declarations,
                statements,
                funcs,
            },
            input,
        ))
    }
}

struct_serde! {
    ConcreteTypeLongId {
        generic_id: GenericTypeId,
        generic_args: Vec<GenericArg>,
    }
}

struct_serde! {
    ConcreteLibfuncLongId {
        generic_id: GenericLibfuncId,
        generic_args: Vec<GenericArg>,
    }
}

struct_serde! {
    FunctionSignature {
        param_types:  Vec<ConcreteTypeId>,
        ret_types:  Vec<ConcreteTypeId>,
    }
}

struct_serde! {
    Invocation {
        libfunc_id: ConcreteLibfuncId,
        args: Vec<VarId>,
        branches: Vec<BranchInfo>,
    }
}

struct_serde! {
    BranchInfo {
        target: BranchTarget,
        results: Vec<VarId>,
    }
}

struct_serde!(VersionId {
    major: usize,
    minor: usize,
    patch: usize
});

// Impls for enums.

macro_rules! enum_deserialize {
    ($($variant_name:ident ( $variant_type:ty ) = $variant_id:literal),*) => {
        fn deserialize(
            input: &[BigUintAsHex],
        ) -> Result<(Self, &[BigUintAsHex]), Felt252SerdeError> {
            let (id, input) = u64::deserialize(input)?;
            match id {
                $($variant_id => {
                    let (value, input) = <$variant_type>::deserialize(input)?;
                    Ok((Self::$variant_name(value), input))
                },)*
                _ => Err(Felt252SerdeError::InvalidInputForDeserialization),
            }
        }
    };
}

macro_rules! enum_serde {
    ($Obj:ident { $($variant_name:ident ( $variant_type:ty ) = $variant_id:literal),* $(,)? }) => {
        impl Felt252Serde for $Obj {
            enum_deserialize! { $($variant_name($variant_type) = $variant_id),* }
        }
    }
}

enum_serde! {
    Statement {
        Invocation(Invocation) = 0,
        Return(Vec::<VarId>) = 1,
    }
}

enum_serde! {
    GenericArg {
        UserType(UserTypeId) = 0,
        Type(ConcreteTypeId) = 1,
        Value(BigInt) = 2,
        UserFunc(FunctionId) = 3,
        Libfunc(ConcreteLibfuncId) = 4,
    }
}

impl Felt252Serde for BranchTarget {
    fn deserialize(input: &[BigUintAsHex]) -> Result<(Self, &[BigUintAsHex]), Felt252SerdeError> {
        let (idx, input) = usize::deserialize(input)?;
        Ok((
            if idx == usize::MAX {
                Self::Fallthrough
            } else {
                Self::Statement(StatementIdx(idx))
            },
            input,
        ))
    }
}
