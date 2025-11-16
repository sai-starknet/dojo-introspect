//! Types matching the Dojo introspect implementation.
//!
//! An important note is about how Dojo handles introspect for legacy and non-legacy models.
//!
//! The legacy models use serde to serialize the keys AND all the value fields. Since in Dojo
//! the introspect wasn't decoupled enough from the storage system, the tech debt is having
//! the enums first variant stored as 0. Which can cause issues when reading uninitialized storage.
//!
//! To solve that, the `DojoStore` introduced recently in Dojo offsets the first variant of the
//! enums by 1.
//! However, since the keys are never stored, they are still using the serde logic (where the first variant is 0).
//!
//! For this reason, it is important while deserializing a typedef or a value to correctly map the legacy flag.

use dojo_introspect_utils::selector::compute_selector_from_namespace_and_name;
use introspect_types::{
    ArrayDef, Attribute, ByteArrayDeserialization, ColumnDef, EnumDef, FeltIterator, FixedArrayDef,
    MemberDef, PrimaryDef, PrimaryTypeDef, StructDef, TableSchema, TupleDef, TypeDef, VariantDef,
    ascii_str_to_limbs, pop_primitive,
};
use num_traits::ToPrimitive;
use starknet::core::utils::get_selector_from_name;
use starknet_types_core::felt::Felt;
pub mod contract;

pub use contract::{DojoSchemaFetcher, DojoSchemaFetcherError};
#[cfg(test)]
mod tests;

pub const KEY_ATTRIBUTE_LIMBS: [u64; 4] = ascii_str_to_limbs("key");
pub const KEY_ATTRIBUTE_FELT: Felt = Felt::from_raw(KEY_ATTRIBUTE_LIMBS);

pub mod primitive {
    use starknet_types_core::felt::Felt;
    pub const BOOL_FELT: Felt = Felt::from_hex_unchecked("0x626f6f6c");
    pub const U8_FELT: Felt = Felt::from_hex_unchecked("0x7538");
    pub const U16_FELT: Felt = Felt::from_hex_unchecked("0x753136");
    pub const U32_FELT: Felt = Felt::from_hex_unchecked("0x753332");
    pub const U64_FELT: Felt = Felt::from_hex_unchecked("0x753634");
    pub const U128_FELT: Felt = Felt::from_hex_unchecked("0x75313238");
    pub const U256_FELT: Felt = Felt::from_hex_unchecked("0x75323536");
    pub const I8_FELT: Felt = Felt::from_hex_unchecked("0x6938");
    pub const I16_FELT: Felt = Felt::from_hex_unchecked("0x693136");
    pub const I32_FELT: Felt = Felt::from_hex_unchecked("0x693332");
    pub const I64_FELT: Felt = Felt::from_hex_unchecked("0x693634");
    pub const I128_FELT: Felt = Felt::from_hex_unchecked("0x69313238");
    pub const FELT252_FELT: Felt = Felt::from_hex_unchecked("0x66656c74323532");
    pub const CLASS_HASH_FELT: Felt = Felt::from_hex_unchecked("0x436c61737348617368");
    pub const CONTRACT_ADDRESS_FELT: Felt =
        Felt::from_hex_unchecked("0x436f6e747261637441646472657373");
    pub const ETH_ADDRESS_FELT: Felt = Felt::from_hex_unchecked("0x45746841646472657373");
    pub const STARKNET_CLASS_HASH: Felt =
        Felt::from_hex_unchecked("0x737461726b6e65743a3a436c61737348617368");
    pub const STARKNET_CONTRACT_ADDRESS: Felt =
        Felt::from_hex_unchecked("0x737461726b6e65743a3a436f6e747261637441646472657373");
    pub const STARKNET_ETH_ADDRESS: Felt =
        Felt::from_hex_unchecked("0x737461726b6e65743a3a45746841646472657373");
}


pub trait IsDojoKey{
    fn is_dojo_key(&self) -> bool;
}

fn span_is_singleton(data: &mut FeltIterator) -> bool {
    data.next() == Some(Felt::ONE)
}

fn felt_to_utf8_string(felt: Felt) -> Option<String> {
    let bytes = felt.to_bytes_be();
    let first = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
    String::from_utf8(bytes[first..32].to_vec()).ok()
}

fn pop_short_string(data: &mut FeltIterator) -> Option<String> {
    data.next().and_then(felt_to_utf8_string)
}

fn dojo_deserialize_primitive(data: &mut FeltIterator, _legacy: bool) -> Option<TypeDef> {
    let kind = data.next()?;
    if kind == primitive::BOOL_FELT {
        Some(TypeDef::Bool)
    } else if kind == primitive::U8_FELT {
        Some(TypeDef::U8)
    } else if kind == primitive::U16_FELT {
        Some(TypeDef::U16)
    } else if kind == primitive::U32_FELT {
        Some(TypeDef::U32)
    } else if kind == primitive::U64_FELT {
        Some(TypeDef::U64)
    } else if kind == primitive::U128_FELT {
        Some(TypeDef::U128)
    } else if kind == primitive::U256_FELT {
        Some(TypeDef::U256)
    } else if kind == primitive::I8_FELT {
        Some(TypeDef::I8)
    } else if kind == primitive::I16_FELT {
        Some(TypeDef::I16)
    } else if kind == primitive::I32_FELT {
        Some(TypeDef::I32)
    } else if kind == primitive::I64_FELT {
        Some(TypeDef::I64)
    } else if kind == primitive::I128_FELT {
        Some(TypeDef::I128)
    } else if kind == primitive::FELT252_FELT {
        Some(TypeDef::Felt252)
    } else if kind == primitive::CLASS_HASH_FELT || kind == primitive::STARKNET_CLASS_HASH {
        Some(TypeDef::ClassHash)
    } else if kind == primitive::CONTRACT_ADDRESS_FELT
        || kind == primitive::STARKNET_CONTRACT_ADDRESS
    {
        Some(TypeDef::ContractAddress)
    } else if kind == primitive::ETH_ADDRESS_FELT || kind == primitive::STARKNET_CLASS_HASH {
        Some(TypeDef::EthAddress)
    } else {
        None
    }
}

fn attribute_is_key(attribute: &Attribute) -> bool {
    attribute.id.to_raw() == KEY_ATTRIBUTE_LIMBS
}

impl<T> DojoTypeDefSerde for Vec<T>
where
    T: DojoTypeDefSerde,
{
    fn dojo_deserialize(data: &mut FeltIterator, legacy: bool) -> Option<Self> {
        (0..pop_primitive(data)?)
            .map(|_| T::dojo_deserialize(data, legacy))
            .collect()
    }
}

pub trait DojoTypeDefSerde: Sized {
    fn dojo_deserialize(data: &mut FeltIterator, legacy: bool) -> Option<Self>;
    fn dojo_deserialize_boxed(data: &mut FeltIterator, legacy: bool) -> Option<Box<Self>> {
        Self::dojo_deserialize(data, legacy).map(Box::new)
    }
}

impl DojoTypeDefSerde for FixedArrayDef {
    fn dojo_deserialize(data: &mut FeltIterator, legacy: bool) -> Option<Self> {
        if !span_is_singleton(data) {
            return None;
        }
        let type_def = TypeDef::dojo_deserialize(data, legacy)?;
        let size = data.next()?.to_u32()?;
        Some(FixedArrayDef { type_def, size })
    }
}

impl DojoTypeDefSerde for StructDef {
    fn dojo_deserialize(data: &mut FeltIterator, legacy: bool) -> Option<Self> {
        let name = data.next().and_then(felt_to_utf8_string)?;
        let attributes = Vec::<Attribute>::dojo_deserialize(data, legacy)?;
        let members = Vec::<MemberDef>::dojo_deserialize(data, legacy)?;
        Some(StructDef {
            name,
            attributes,
            members,
        })
    }
}

impl DojoTypeDefSerde for MemberDef {
    fn dojo_deserialize(data: &mut FeltIterator, legacy: bool) -> Option<Self> {
        let name = pop_short_string(data)?;
        let attributes = Vec::<Attribute>::dojo_deserialize(data, legacy)?;
        let type_def = TypeDef::dojo_deserialize(data, legacy)?;
        Some(MemberDef {
            name,
            attributes,
            type_def,
        })
    }
}

impl DojoTypeDefSerde for VariantDef {
    fn dojo_deserialize(data: &mut FeltIterator, legacy: bool) -> Option<Self> {
        let name = data.next().and_then(felt_to_utf8_string)?;
        let attributes = vec![];
        let type_def = TypeDef::dojo_deserialize(data, legacy)?;
        Some(VariantDef {
            name,
            attributes,
            type_def,
        })
    }
}

impl DojoTypeDefSerde for EnumDef {
    fn dojo_deserialize(data: &mut FeltIterator, legacy: bool) -> Option<Self> {
        let name = data.next().and_then(felt_to_utf8_string)?;
        let attributes = Vec::<Attribute>::dojo_deserialize(data, legacy)?;
        let legacy_mod: usize = (!legacy).into();
        let variants = Vec::<VariantDef>::dojo_deserialize(data, legacy)?
            .into_iter()
            .enumerate()
            .map(|(i, v)| ((i + legacy_mod).into(), v))
            .collect::<Vec<_>>();
        Some(EnumDef::new(
            name.clone(),
            attributes.clone(),
            variants.clone(),
        ))
    }
}

impl DojoTypeDefSerde for ArrayDef {
    fn dojo_deserialize(data: &mut FeltIterator, legacy: bool) -> Option<Self> {
        match span_is_singleton(data) {
            true => TypeDef::dojo_deserialize(data, legacy).map(ArrayDef::new),
            false => None,
        }
    }
}

impl DojoTypeDefSerde for TupleDef {
    fn dojo_deserialize(data: &mut FeltIterator, legacy: bool) -> Option<Self> {
        Some(TupleDef {
            elements: Vec::<TypeDef>::dojo_deserialize(data, legacy)?,
        })
    }
}

impl DojoTypeDefSerde for ColumnDef {
    fn dojo_deserialize(data: &mut FeltIterator, legacy: bool) -> Option<Self> {
        let name = pop_short_string(data)?;
        let attributes = Vec::<Attribute>::dojo_deserialize(data, legacy)?;
        let is_key = attributes.iter().any(attribute_is_key);
        let type_def = TypeDef::dojo_deserialize(data, legacy || is_key)?;
        Some(ColumnDef {
            id: get_selector_from_name(&name).ok()?,
            name,
            attributes,
            type_def,
        })
    }
}

impl IsDojoKey for ColumnDef{
    fn is_dojo_key(&self) -> bool {
        self.attributes.iter().any(attribute_is_key)
    }
}

impl DojoTypeDefSerde for TypeDef {
    fn dojo_deserialize(data: &mut FeltIterator, legacy: bool) -> Option<Self> {
        let kind = data.next()?.to_u32()?;
        match kind {
            0 => dojo_deserialize_primitive(data, legacy),
            1 => StructDef::dojo_deserialize(data, legacy).map(TypeDef::Struct),
            2 => EnumDef::dojo_deserialize(data, legacy).map(TypeDef::Enum),
            3 => TupleDef::dojo_deserialize(data, legacy).map(TupleDef::to_type_def),
            4 => ArrayDef::dojo_deserialize_boxed(data, legacy).map(TypeDef::Array),
            5 => Some(TypeDef::ByteArray(ByteArrayDeserialization::Serde)),
            6 => FixedArrayDef::dojo_deserialize(data, legacy)
                .map(|x| TypeDef::FixedArray(Box::new(x))),
            _ => None,
        }
    }
}

impl DojoTypeDefSerde for Attribute {
    fn dojo_deserialize(data: &mut FeltIterator, _legacy: bool) -> Option<Self> {
        Some(Attribute {
            id: data.next()?,
            data: vec![],
        })
    }
}

impl DojoTypeDefSerde for Box<TypeDef> {
    fn dojo_deserialize(data: &mut FeltIterator, legacy: bool) -> Option<Self> {
        TypeDef::dojo_deserialize(data, legacy).map(Box::new)
    }
}

fn dojo_primary_def() -> PrimaryDef {
    PrimaryDef {
        name: "entity_id".to_string(),
        attributes: vec![],
        type_def: PrimaryTypeDef::Felt252,
    }
}

pub fn make_dojo_table(
    namespace: &str,
    name: &str,
    data: Vec<Felt>,
    legacy: bool,
) -> Option<TableSchema> {
    let mut data = data.into_iter();
    let _struct_name = pop_short_string(&mut data)?;
    let attributes = Vec::<Attribute>::dojo_deserialize(&mut data, legacy)?;
    let columns = Vec::<ColumnDef>::dojo_deserialize(&mut data, legacy)?.into();

    Some(TableSchema {
        id: compute_selector_from_namespace_and_name(namespace, name),
        name: format!("{}-{}", namespace, name),
        attributes,
        primary: dojo_primary_def(),
        columns,
    })
}
