use introspect_types::{
    read_serialized_felt_array, EnumDef, FixedArrayDef, MemberDef, StructDef, TypeDef, VariantDef,
};
use num_traits::ToPrimitive;
use starknet_types_core::felt::Felt;
use std::{
    collections::{HashMap, VecDeque},
    vec,
};
#[cfg(test)]
mod tests;

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

fn span_is_singleton(data: &mut VecDeque<Felt>) -> bool {
    data.pop_front() == Some(Felt::ONE)
}

fn dojo_deserialize_array(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Box<TypeDef>> {
    match span_is_singleton(data) {
        true => Box::<TypeDef>::dojo_deserialize(data, legacy),
        false => None,
    }
}

fn dojo_deserialize_tuple(data: &mut VecDeque<Felt>, legacy: bool) -> Option<TypeDef> {
    let len = data.pop_front()?.to_usize()?;
    if len == 0 {
        return Some(TypeDef::None);
    }
    let mut elements = Vec::with_capacity(len);
    for _ in 0..len {
        elements.push(TypeDef::dojo_deserialize(data, legacy)?);
    }
    Some(TypeDef::Tuple(elements))
}

fn felt_to_utf8_string(felt: Felt) -> Option<String> {
    let bytes = felt.to_bytes_be();
    let first = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
    String::from_utf8(bytes[first..32].to_vec()).ok()
}

pub trait DojoTypeDefSerde: Sized {
    fn dojo_deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self>;
}

pub fn parse_attrs(data: &mut VecDeque<Felt>) -> Option<Vec<String>> {
    Some(
        read_serialized_felt_array(data)?
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>(),
    )
}

impl DojoTypeDefSerde for MemberDef {
    fn dojo_deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self> {
        let name = data.pop_front().and_then(felt_to_utf8_string)?;
        let attrs = parse_attrs(data)?;
        let type_def = TypeDef::dojo_deserialize(data, legacy)?;
        Some(MemberDef {
            name,
            attrs,
            type_def,
        })
    }
}

impl DojoTypeDefSerde for FixedArrayDef {
    fn dojo_deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self> {
        if !span_is_singleton(data) {
            return None;
        }
        let type_def = Box::<TypeDef>::dojo_deserialize(data, legacy)?;
        let size = data.pop_front()?.to_u32()?;
        Some(FixedArrayDef { type_def, size })
    }
}

impl DojoTypeDefSerde for StructDef {
    fn dojo_deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self> {
        let name = data.pop_front().and_then(felt_to_utf8_string)?;
        let attrs = parse_attrs(data)?;
        let children_len = data.pop_front()?.to_usize()?;
        let mut children = Vec::with_capacity(children_len);
        for _ in 0..children_len {
            children.push(MemberDef::dojo_deserialize(data, legacy)?);
        }
        Some(StructDef {
            name,
            attrs,
            children,
        })
    }
}

impl DojoTypeDefSerde for VariantDef {
    fn dojo_deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self> {
        let name = data.pop_front().and_then(felt_to_utf8_string)?;
        let attrs = vec![];
        let type_def = TypeDef::dojo_deserialize(data, legacy)?;
        Some(VariantDef {
            name,
            attrs,
            type_def,
        })
    }
}

impl DojoTypeDefSerde for EnumDef {
    fn dojo_deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self> {
        let name = data.pop_front().and_then(felt_to_utf8_string)?;

        let attrs = parse_attrs(data)?;
        let legacy_mod: usize = (!legacy).into();

        let variants_len = data.pop_front()?.to_usize()?;
        let mut variants = HashMap::with_capacity(variants_len);
        for i in 0..variants_len {
            let variant = VariantDef::dojo_deserialize(data, legacy)?;
            variants.insert((i + legacy_mod).into(), variant);
        }
        Some(EnumDef {
            name,
            attrs,
            variants,
        })
    }
}

fn dojo_deserialize_primitive(data: &mut VecDeque<Felt>, _legacy: bool) -> Option<TypeDef> {
    let kind = data.pop_front()?;
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

impl DojoTypeDefSerde for TypeDef {
    fn dojo_deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self> {
        let kind = data.pop_front()?.to_u32()?;
        match kind {
            0 => dojo_deserialize_primitive(data, legacy),
            1 => StructDef::dojo_deserialize(data, legacy).map(TypeDef::Struct),
            2 => EnumDef::dojo_deserialize(data, legacy).map(TypeDef::Enum),
            3 => dojo_deserialize_tuple(data, legacy),
            4 => dojo_deserialize_array(data, legacy).map(TypeDef::Array),
            5 => Some(TypeDef::ByteArray),
            6 => FixedArrayDef::dojo_deserialize(data, legacy).map(TypeDef::FixedArray),
            _ => None,
        }
    }
}

impl DojoTypeDefSerde for Box<TypeDef> {
    fn dojo_deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self> {
        TypeDef::dojo_deserialize(data, legacy).map(Box::new)
    }
}
