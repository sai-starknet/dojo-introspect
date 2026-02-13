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

use introspect_types::deserialize::CairoDeserializer;
use introspect_types::deserialize_def::CairoDeserializeItemDef;
use introspect_types::{
    ArrayDef, Attribute, CairoDeserialize, CairoSerde, ColumnDef, DecodeError, DecodeResult,
    EnumDef, FeltSource, FixedArrayDef, MemberDef, PrimaryDef, PrimaryTypeDef, SliceFeltSource,
    StructDef, TableSchema, TupleDef, TypeDef, VariantDef, ascii_str_to_limbs,
};
use serde::{Deserialize, Serialize};
use starknet::core::utils::get_selector_from_name;
use starknet_types_core::felt::Felt;

use crate::selector::compute_selector_from_namespace_and_name;

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

pub trait IsDojoKey {
    fn is_dojo_key(&self) -> bool;
}

// fn span_is_singleton(data: &mut FeltIterator) -> bool {
//     data.next() == Some(Felt::ONE)
// }

// fn pop_short_string(data: &mut FeltIterator) -> Option<String> {
//     data.next().and_then(felt_to_utf8_string)
// }

fn dojo_deserialize_primitive(kind: Felt) -> DecodeResult<TypeDef> {
    if kind == primitive::BOOL_FELT {
        Ok(TypeDef::Bool)
    } else if kind == primitive::U8_FELT {
        Ok(TypeDef::U8)
    } else if kind == primitive::U16_FELT {
        Ok(TypeDef::U16)
    } else if kind == primitive::U32_FELT {
        Ok(TypeDef::U32)
    } else if kind == primitive::U64_FELT {
        Ok(TypeDef::U64)
    } else if kind == primitive::U128_FELT {
        Ok(TypeDef::U128)
    } else if kind == primitive::U256_FELT {
        Ok(TypeDef::U256)
    } else if kind == primitive::I8_FELT {
        Ok(TypeDef::I8)
    } else if kind == primitive::I16_FELT {
        Ok(TypeDef::I16)
    } else if kind == primitive::I32_FELT {
        Ok(TypeDef::I32)
    } else if kind == primitive::I64_FELT {
        Ok(TypeDef::I64)
    } else if kind == primitive::I128_FELT {
        Ok(TypeDef::I128)
    } else if kind == primitive::FELT252_FELT {
        Ok(TypeDef::Felt252)
    } else if kind == primitive::CLASS_HASH_FELT || kind == primitive::STARKNET_CLASS_HASH {
        Ok(TypeDef::ClassHash)
    } else if kind == primitive::CONTRACT_ADDRESS_FELT
        || kind == primitive::STARKNET_CONTRACT_ADDRESS
    {
        Ok(TypeDef::ContractAddress)
    } else if kind == primitive::ETH_ADDRESS_FELT || kind == primitive::STARKNET_ETH_ADDRESS {
        Ok(TypeDef::EthAddress)
    } else {
        Err(DecodeError::invalid_enum_selector("Dojo Primitive", kind))
    }
}

pub struct DojoSerde<I: FeltSource> {
    serde: CairoSerde<I>,
    legacy: bool,
}

impl<I: FeltSource> FeltSource for DojoSerde<I> {
    fn next(&mut self) -> Result<Felt, DecodeError> {
        self.serde.next_felt()
    }

    fn position(&self) -> usize {
        self.serde.position()
    }
}

impl<'a> DojoSerde<SliceFeltSource<'a>> {
    pub fn from_slice(slice: &'a [Felt], legacy: bool) -> Self {
        let serde = CairoSerde::<SliceFeltSource>::from(slice);
        Self::new(serde, legacy)
    }
}

impl<'a, I: FeltSource> DojoSerde<I> {
    pub fn new(serde: CairoSerde<I>, legacy: bool) -> Self {
        Self { serde, legacy }
    }
    pub fn new_from_source<S>(source: S, legacy: bool) -> Self
    where
        S: Into<CairoSerde<I>>,
    {
        Self::new(source.into(), legacy)
    }
    pub fn singleton_span(&mut self) -> DecodeResult<TypeDef> {
        if self.serde.next_felt() != Ok(Felt::ONE) {
            return Err(DecodeError::message("Expected singleton span"));
        }
        CairoDeserialize::<Self>::deserialize(self)
    }
    pub fn next_tuple_def(&mut self) -> DecodeResult<TypeDef> {
        let size = self.serde.next_u32()?;
        match size {
            0 => Ok(TypeDef::None),
            _ => CairoDeserialize::deserialize_multiple(self, size as usize)
                .map(TupleDef::new)
                .map(TypeDef::Tuple),
        }
    }
    #[inline]
    fn with_legacy<R>(
        &mut self,
        legacy: bool,
        f: impl FnOnce(&mut Self) -> DecodeResult<R>,
    ) -> DecodeResult<R> {
        let prev = self.legacy;
        self.legacy = legacy;
        let r = f(self);
        self.legacy = prev;
        r
    }
}

impl<'a, I, T> CairoDeserialize<DojoSerde<I>> for Vec<T>
where
    I: FeltSource,
    T: CairoDeserialize<DojoSerde<I>>,
{
    fn deserialize(deserializer: &mut DojoSerde<I>) -> DecodeResult<Self> {
        (0..deserializer.serde.next_u32()?)
            .map(|_| T::deserialize(deserializer))
            .collect()
    }
}

impl<'a, I: FeltSource> CairoDeserialize<DojoSerde<I>> for TypeDef {
    fn deserialize(deserializer: &mut DojoSerde<I>) -> DecodeResult<Self> {
        let kind = deserializer.serde.next_u32()?;
        match kind {
            0 => dojo_deserialize_primitive(deserializer.next()?),
            1 => StructDef::deserialize_item(deserializer),
            2 => EnumDef::deserialize_item(deserializer),
            3 => deserializer.next_tuple_def(),
            4 => ArrayDef::deserialize_item(deserializer),
            5 => Ok(TypeDef::Utf8String),
            6 => FixedArrayDef::deserialize_item(deserializer),
            _ => Err(DecodeError::invalid_enum_selector("Dojo TypeDef", kind)),
        }
    }
}

impl<'a, I: FeltSource> CairoDeserialize<DojoSerde<I>> for Attribute {
    fn deserialize(deserializer: &mut DojoSerde<I>) -> DecodeResult<Self> {
        deserializer
            .serde
            .next_short_string()
            .map(Attribute::new_empty)
    }
}

impl<'a, I: FeltSource> CairoDeserialize<DojoSerde<I>> for ArrayDef {
    fn deserialize(deserializer: &mut DojoSerde<I>) -> DecodeResult<Self> {
        deserializer.singleton_span().map(ArrayDef::new)
    }
}
impl<'a, I: FeltSource> CairoDeserialize<DojoSerde<I>> for FixedArrayDef {
    fn deserialize(deserializer: &mut DojoSerde<I>) -> DecodeResult<Self> {
        let type_def = deserializer.singleton_span()?;
        let size = deserializer.serde.next_u32()?;
        Ok(FixedArrayDef { type_def, size })
    }
}

impl<'a, I: FeltSource> CairoDeserialize<DojoSerde<I>> for TupleDef {
    fn deserialize(deserializer: &mut DojoSerde<I>) -> DecodeResult<Self> {
        CairoDeserialize::deserialize(deserializer).map(TupleDef::new)
    }
}

impl<'a, I: FeltSource> CairoDeserialize<DojoSerde<I>> for StructDef {
    fn deserialize(deserializer: &mut DojoSerde<I>) -> DecodeResult<Self> {
        let name = deserializer.serde.next_short_string()?;
        let attributes = CairoDeserialize::deserialize(deserializer)?;
        let members = CairoDeserialize::deserialize(deserializer)?;
        Ok(StructDef {
            name,
            attributes,
            members,
        })
    }
}

impl<'a, I: FeltSource> CairoDeserialize<DojoSerde<I>> for EnumDef {
    fn deserialize(deserializer: &mut DojoSerde<I>) -> DecodeResult<Self> {
        let name = deserializer.serde.next_short_string()?;
        let attributes = CairoDeserialize::deserialize(deserializer)?;
        let legacy_mod: usize = (!deserializer.legacy).into();
        let variants: Vec<VariantDef> = CairoDeserialize::deserialize(deserializer)?;
        Ok(EnumDef::new(
            name,
            attributes,
            variants
                .into_iter()
                .enumerate()
                .map(|(i, v)| ((i + legacy_mod).into(), v))
                .collect(),
        ))
    }
}

impl<'a, I: FeltSource> CairoDeserialize<DojoSerde<I>> for MemberDef {
    fn deserialize(deserializer: &mut DojoSerde<I>) -> DecodeResult<Self> {
        let name = deserializer.serde.next_short_string()?;
        let attributes = CairoDeserialize::deserialize(deserializer)?;
        let type_def = CairoDeserialize::deserialize(deserializer)?;
        Ok(MemberDef {
            name,
            attributes,
            type_def,
        })
    }
}

impl<'a, I: FeltSource> CairoDeserialize<DojoSerde<I>> for VariantDef {
    fn deserialize(deserializer: &mut DojoSerde<I>) -> DecodeResult<Self> {
        let name = deserializer.serde.next_short_string()?;
        let attributes = vec![];
        let type_def = CairoDeserialize::deserialize(deserializer)?;
        Ok(VariantDef {
            name,
            attributes,
            type_def,
        })
    }
}

impl<'a, I: FeltSource> CairoDeserialize<DojoSerde<I>> for ColumnDef {
    fn deserialize(deserializer: &mut DojoSerde<I>) -> DecodeResult<Self> {
        let name = deserializer.serde.next_short_string()?;
        let attributes: Vec<Attribute> = CairoDeserialize::deserialize(deserializer)?;
        let is_key = attributes.iter().any(|a| a.name == "key");
        let type_def = deserializer.with_legacy(deserializer.legacy || is_key, |d| {
            CairoDeserialize::deserialize(d)
        })?;
        Ok(ColumnDef {
            id: get_selector_from_name(&name)
                .map_err(|_| DecodeError::message("Non Ascii Name Error"))?,
            name,
            attributes,
            type_def,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DojoSchema {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub columns: Vec<ColumnDef>,
}

impl<'a, I: FeltSource> CairoDeserialize<DojoSerde<I>> for DojoSchema {
    fn deserialize(deserializer: &mut DojoSerde<I>) -> DecodeResult<Self> {
        let name = deserializer.serde.next_short_string()?;
        let attributes = CairoDeserialize::deserialize(deserializer)?;
        let columns = CairoDeserialize::deserialize(deserializer)?;
        Ok(DojoSchema {
            name,
            attributes,
            columns,
        })
    }
}

impl DojoSchema {
    pub fn to_table_schema(&self, namespace: &str, name: &str) -> TableSchema {
        TableSchema {
            id: compute_selector_from_namespace_and_name(namespace, name),
            name: format!("{}-{}", namespace, name),
            attributes: self.attributes.clone(),
            primary: dojo_primary_def(),
            columns: self.columns.clone(),
        }
    }
}

fn dojo_primary_def() -> PrimaryDef {
    PrimaryDef {
        name: "entity_id".to_string(),
        attributes: vec![],
        type_def: PrimaryTypeDef::Felt252,
    }
}
