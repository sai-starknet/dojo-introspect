use introspect_types::{
    read_serialized_felt_array, EnumDef, FieldDef, FixedArrayDef, MemberDef, StructDef, TypeDef,
};
use num_traits::{One, ToPrimitive};
use starknet_types_core::felt::Felt;
use std::{
    collections::{HashMap, VecDeque},
    vec,
};

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

fn dojo_deserialize_array(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Box<TypeDef>> {
    if data.pop_front()? != Felt::ONE {
        return None;
    }
    TypeDef::dojo_deserialize(data, legacy).map(Box::new)
}

fn dojo_deserialize_fixed_array(data: &mut VecDeque<Felt>, legacy: bool) -> Option<FixedArrayDef> {
    if data.pop_front()?.is_one() {
        return None;
    }
    let ty = TypeDef::dojo_deserialize(data, legacy).map(Box::new)?;
    let size = data.pop_front()?.to_u32()?;
    Some(FixedArrayDef { ty, size })
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
        let ty = TypeDef::dojo_deserialize(data, legacy)?;
        Some(MemberDef { name, attrs, ty })
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

impl DojoTypeDefSerde for FieldDef {
    fn dojo_deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self> {
        let name = data.pop_front().and_then(felt_to_utf8_string)?;
        let attrs = vec![];
        let ty = TypeDef::dojo_deserialize(data, legacy)?;
        Some(FieldDef { name, attrs, ty })
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
            let variant = FieldDef::dojo_deserialize(data, legacy)?;
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
            6 => dojo_deserialize_fixed_array(data, legacy).map(TypeDef::FixedArray),
            _ => None,
        }
    }
}

mod tests {
    use super::DojoTypeDefSerde;
    use introspect_types::StructDef;
    use introspect_value::ToValue;
    use starknet_types_core::felt::Felt;
    use std::collections::VecDeque;

    fn test_schema_felts() -> Vec<Felt> {
        vec![
            "0x41747461636b576974684e616d65",
            "0x0",
            "0x6",
            "0x6e616d65",
            "0x0",
            "0x5",
            "0x7370656564",
            "0x0",
            "0x0",
            "0x753136",
            "0x6368616e6365",
            "0x0",
            "0x0",
            "0x7538",
            "0x636f6f6c646f776e",
            "0x0",
            "0x0",
            "0x753332",
            "0x73756363657373",
            "0x0",
            "0x4",
            "0x1",
            "0x1",
            "0x456666656374",
            "0x0",
            "0x3",
            "0x746172676574",
            "0x0",
            "0x2",
            "0x546172676574",
            "0x0",
            "0x2",
            "0x41747461636b6572",
            "0x3",
            "0x0",
            "0x446566656e646572",
            "0x3",
            "0x0",
            "0x6475726174696f6e",
            "0x0",
            "0x2",
            "0x4475726174696f6e",
            "0x0",
            "0x4",
            "0x496e7374616e74",
            "0x3",
            "0x0",
            "0x526f756e64",
            "0x0",
            "0x753332",
            "0x526f756e6473",
            "0x0",
            "0x753332",
            "0x496e66696e697465",
            "0x3",
            "0x0",
            "0x616666656374",
            "0x0",
            "0x2",
            "0x416666656374",
            "0x0",
            "0x28",
            "0x4e6f6e65",
            "0x3",
            "0x0",
            "0x4865616c7468",
            "0x0",
            "0x693136",
            "0x5374756e",
            "0x0",
            "0x7538",
            "0x426c6f636b",
            "0x0",
            "0x7538",
            "0x537472656e677468",
            "0x0",
            "0x6938",
            "0x566974616c697479",
            "0x0",
            "0x6938",
            "0x446578746572697479",
            "0x0",
            "0x6938",
            "0x4c75636b",
            "0x0",
            "0x6938",
            "0x5374756e526573697374616e6365",
            "0x0",
            "0x6938",
            "0x426c756467656f6e526573697374616e6365",
            "0x0",
            "0x6938",
            "0x4d61676963526573697374616e6365",
            "0x0",
            "0x6938",
            "0x506965726365526573697374616e6365",
            "0x0",
            "0x6938",
            "0x426c756467656f6e56756c6e65726162696c697479",
            "0x0",
            "0x693136",
            "0x4d6167696356756c6e65726162696c697479",
            "0x0",
            "0x693136",
            "0x50696572636556756c6e65726162696c697479",
            "0x0",
            "0x693136",
            "0x4162696c6974696573",
            "0x1",
            "0x4162696c6974794d6f6473",
            "0x0",
            "0x4",
            "0x737472656e677468",
            "0x0",
            "0x0",
            "0x6938",
            "0x766974616c697479",
            "0x0",
            "0x0",
            "0x6938",
            "0x646578746572697479",
            "0x0",
            "0x0",
            "0x6938",
            "0x6c75636b",
            "0x0",
            "0x0",
            "0x6938",
            "0x526573697374616e636573",
            "0x1",
            "0x526573697374616e63654d6f6473",
            "0x0",
            "0x4",
            "0x7374756e",
            "0x0",
            "0x0",
            "0x6938",
            "0x626c756467656f6e",
            "0x0",
            "0x0",
            "0x6938",
            "0x6d61676963",
            "0x0",
            "0x0",
            "0x6938",
            "0x706965726365",
            "0x0",
            "0x0",
            "0x6938",
            "0x56756c6e65726162696c6974696573",
            "0x1",
            "0x56756c6e65726162696c6974794d6f6473",
            "0x0",
            "0x3",
            "0x626c756467656f6e",
            "0x0",
            "0x0",
            "0x693136",
            "0x6d61676963",
            "0x0",
            "0x0",
            "0x693136",
            "0x706965726365",
            "0x0",
            "0x0",
            "0x693136",
            "0x537472656e67746854656d70",
            "0x0",
            "0x6938",
            "0x566974616c69747954656d70",
            "0x0",
            "0x6938",
            "0x44657874657269747954656d70",
            "0x0",
            "0x6938",
            "0x4c75636b54656d70",
            "0x0",
            "0x6938",
            "0x5374756e526573697374616e636554656d70",
            "0x0",
            "0x6938",
            "0x426c756467656f6e526573697374616e636554656d70",
            "0x0",
            "0x6938",
            "0x4d61676963526573697374616e636554656d70",
            "0x0",
            "0x6938",
            "0x506965726365526573697374616e636554656d70",
            "0x0",
            "0x6938",
            "0x426c756467656f6e56756c6e65726162696c69747954656d70",
            "0x0",
            "0x693136",
            "0x4d6167696356756c6e65726162696c69747954656d70",
            "0x0",
            "0x693136",
            "0x50696572636556756c6e65726162696c69747954656d70",
            "0x0",
            "0x693136",
            "0x4162696c697469657354656d70",
            "0x1",
            "0x4162696c6974794d6f6473",
            "0x0",
            "0x4",
            "0x737472656e677468",
            "0x0",
            "0x0",
            "0x6938",
            "0x766974616c697479",
            "0x0",
            "0x0",
            "0x6938",
            "0x646578746572697479",
            "0x0",
            "0x0",
            "0x6938",
            "0x6c75636b",
            "0x0",
            "0x0",
            "0x6938",
            "0x526573697374616e63657354656d70",
            "0x1",
            "0x526573697374616e63654d6f6473",
            "0x0",
            "0x4",
            "0x7374756e",
            "0x0",
            "0x0",
            "0x6938",
            "0x626c756467656f6e",
            "0x0",
            "0x0",
            "0x6938",
            "0x6d61676963",
            "0x0",
            "0x0",
            "0x6938",
            "0x706965726365",
            "0x0",
            "0x0",
            "0x6938",
            "0x56756c6e65726162696c697469657354656d70",
            "0x1",
            "0x56756c6e65726162696c6974794d6f6473",
            "0x0",
            "0x3",
            "0x626c756467656f6e",
            "0x0",
            "0x0",
            "0x693136",
            "0x6d61676963",
            "0x0",
            "0x0",
            "0x693136",
            "0x706965726365",
            "0x0",
            "0x0",
            "0x693136",
            "0x44616d616765",
            "0x1",
            "0x44616d616765",
            "0x0",
            "0x3",
            "0x637269746963616c",
            "0x0",
            "0x0",
            "0x7538",
            "0x706f776572",
            "0x0",
            "0x0",
            "0x7538",
            "0x64616d6167655f74797065",
            "0x0",
            "0x2",
            "0x44616d61676554797065",
            "0x0",
            "0x4",
            "0x4e6f6e65",
            "0x3",
            "0x0",
            "0x426c756467656f6e",
            "0x3",
            "0x0",
            "0x4d61676963",
            "0x3",
            "0x0",
            "0x506965726365",
            "0x3",
            "0x0",
            "0x5365744865616c7468",
            "0x0",
            "0x7538",
            "0x466c6f6f724865616c7468",
            "0x0",
            "0x7538",
            "0x4365696c4865616c7468",
            "0x0",
            "0x7538",
            "0x4865616c746850657263656e744d6178",
            "0x0",
            "0x6938",
            "0x5365744865616c746850657263656e744d6178",
            "0x0",
            "0x7538",
            "0x466c6f6f724865616c746850657263656e744d6178",
            "0x0",
            "0x7538",
            "0x4365696c4865616c746850657263656e744d6178",
            "0x0",
            "0x7538",
            "0x6661696c",
            "0x0",
            "0x4",
            "0x1",
            "0x1",
            "0x456666656374",
            "0x0",
            "0x3",
            "0x746172676574",
            "0x0",
            "0x2",
            "0x546172676574",
            "0x0",
            "0x2",
            "0x41747461636b6572",
            "0x3",
            "0x0",
            "0x446566656e646572",
            "0x3",
            "0x0",
            "0x6475726174696f6e",
            "0x0",
            "0x2",
            "0x4475726174696f6e",
            "0x0",
            "0x4",
            "0x496e7374616e74",
            "0x3",
            "0x0",
            "0x526f756e64",
            "0x0",
            "0x753332",
            "0x526f756e6473",
            "0x0",
            "0x753332",
            "0x496e66696e697465",
            "0x3",
            "0x0",
            "0x616666656374",
            "0x0",
            "0x2",
            "0x416666656374",
            "0x0",
            "0x28",
            "0x4e6f6e65",
            "0x3",
            "0x0",
            "0x4865616c7468",
            "0x0",
            "0x693136",
            "0x5374756e",
            "0x0",
            "0x7538",
            "0x426c6f636b",
            "0x0",
            "0x7538",
            "0x537472656e677468",
            "0x0",
            "0x6938",
            "0x566974616c697479",
            "0x0",
            "0x6938",
            "0x446578746572697479",
            "0x0",
            "0x6938",
            "0x4c75636b",
            "0x0",
            "0x6938",
            "0x5374756e526573697374616e6365",
            "0x0",
            "0x6938",
            "0x426c756467656f6e526573697374616e6365",
            "0x0",
            "0x6938",
            "0x4d61676963526573697374616e6365",
            "0x0",
            "0x6938",
            "0x506965726365526573697374616e6365",
            "0x0",
            "0x6938",
            "0x426c756467656f6e56756c6e65726162696c697479",
            "0x0",
            "0x693136",
            "0x4d6167696356756c6e65726162696c697479",
            "0x0",
            "0x693136",
            "0x50696572636556756c6e65726162696c697479",
            "0x0",
            "0x693136",
            "0x4162696c6974696573",
            "0x1",
            "0x4162696c6974794d6f6473",
            "0x0",
            "0x4",
            "0x737472656e677468",
            "0x0",
            "0x0",
            "0x6938",
            "0x766974616c697479",
            "0x0",
            "0x0",
            "0x6938",
            "0x646578746572697479",
            "0x0",
            "0x0",
            "0x6938",
            "0x6c75636b",
            "0x0",
            "0x0",
            "0x6938",
            "0x526573697374616e636573",
            "0x1",
            "0x526573697374616e63654d6f6473",
            "0x0",
            "0x4",
            "0x7374756e",
            "0x0",
            "0x0",
            "0x6938",
            "0x626c756467656f6e",
            "0x0",
            "0x0",
            "0x6938",
            "0x6d61676963",
            "0x0",
            "0x0",
            "0x6938",
            "0x706965726365",
            "0x0",
            "0x0",
            "0x6938",
            "0x56756c6e65726162696c6974696573",
            "0x1",
            "0x56756c6e65726162696c6974794d6f6473",
            "0x0",
            "0x3",
            "0x626c756467656f6e",
            "0x0",
            "0x0",
            "0x693136",
            "0x6d61676963",
            "0x0",
            "0x0",
            "0x693136",
            "0x706965726365",
            "0x0",
            "0x0",
            "0x693136",
            "0x537472656e67746854656d70",
            "0x0",
            "0x6938",
            "0x566974616c69747954656d70",
            "0x0",
            "0x6938",
            "0x44657874657269747954656d70",
            "0x0",
            "0x6938",
            "0x4c75636b54656d70",
            "0x0",
            "0x6938",
            "0x5374756e526573697374616e636554656d70",
            "0x0",
            "0x6938",
            "0x426c756467656f6e526573697374616e636554656d70",
            "0x0",
            "0x6938",
            "0x4d61676963526573697374616e636554656d70",
            "0x0",
            "0x6938",
            "0x506965726365526573697374616e636554656d70",
            "0x0",
            "0x6938",
            "0x426c756467656f6e56756c6e65726162696c69747954656d70",
            "0x0",
            "0x693136",
            "0x4d6167696356756c6e65726162696c69747954656d70",
            "0x0",
            "0x693136",
            "0x50696572636556756c6e65726162696c69747954656d70",
            "0x0",
            "0x693136",
            "0x4162696c697469657354656d70",
            "0x1",
            "0x4162696c6974794d6f6473",
            "0x0",
            "0x4",
            "0x737472656e677468",
            "0x0",
            "0x0",
            "0x6938",
            "0x766974616c697479",
            "0x0",
            "0x0",
            "0x6938",
            "0x646578746572697479",
            "0x0",
            "0x0",
            "0x6938",
            "0x6c75636b",
            "0x0",
            "0x0",
            "0x6938",
            "0x526573697374616e63657354656d70",
            "0x1",
            "0x526573697374616e63654d6f6473",
            "0x0",
            "0x4",
            "0x7374756e",
            "0x0",
            "0x0",
            "0x6938",
            "0x626c756467656f6e",
            "0x0",
            "0x0",
            "0x6938",
            "0x6d61676963",
            "0x0",
            "0x0",
            "0x6938",
            "0x706965726365",
            "0x0",
            "0x0",
            "0x6938",
            "0x56756c6e65726162696c697469657354656d70",
            "0x1",
            "0x56756c6e65726162696c6974794d6f6473",
            "0x0",
            "0x3",
            "0x626c756467656f6e",
            "0x0",
            "0x0",
            "0x693136",
            "0x6d61676963",
            "0x0",
            "0x0",
            "0x693136",
            "0x706965726365",
            "0x0",
            "0x0",
            "0x693136",
            "0x44616d616765",
            "0x1",
            "0x44616d616765",
            "0x0",
            "0x3",
            "0x637269746963616c",
            "0x0",
            "0x0",
            "0x7538",
            "0x706f776572",
            "0x0",
            "0x0",
            "0x7538",
            "0x64616d6167655f74797065",
            "0x0",
            "0x2",
            "0x44616d61676554797065",
            "0x0",
            "0x4",
            "0x4e6f6e65",
            "0x3",
            "0x0",
            "0x426c756467656f6e",
            "0x3",
            "0x0",
            "0x4d61676963",
            "0x3",
            "0x0",
            "0x506965726365",
            "0x3",
            "0x0",
            "0x5365744865616c7468",
            "0x0",
            "0x7538",
            "0x466c6f6f724865616c7468",
            "0x0",
            "0x7538",
            "0x4365696c4865616c7468",
            "0x0",
            "0x7538",
            "0x4865616c746850657263656e744d6178",
            "0x0",
            "0x6938",
            "0x5365744865616c746850657263656e744d6178",
            "0x0",
            "0x7538",
            "0x466c6f6f724865616c746850657263656e744d6178",
            "0x0",
            "0x7538",
            "0x4365696c4865616c746850657263656e744d6178",
            "0x0",
            "0x7538",
        ]
        .into_iter()
        .map(Felt::from_hex_unchecked)
        .collect()
    }

    fn test_record_felts() -> Vec<Felt> {
        [
            "0x0",
            "0x4865616462757474",
            "0x8",
            "0x13ba",
            "0x4b",
            "0x3",
            "0x3",
            "0x1",
            "0x0",
            "0x20",
            "0xa",
            "0x3c",
            "0x1",
            "0x1",
            "0x0",
            "0x2",
            "0x32",
            "0x0",
            "0x0",
            "0x1",
            "0x800000000000010ffffffffffffffffffffffffffffffffffffffffffffffed",
            "0x0",
        ]
        .into_iter()
        .map(Felt::from_hex_unchecked)
        .collect()
    }

    #[test]
    fn test_parse_struct() {
        println!("Testing struct deserialization and parsing");
        let felts = test_schema_felts();
        let mut data = VecDeque::from(felts);
        let struct_def = StructDef::dojo_deserialize(&mut data, true).unwrap();

        println!("{:?}", struct_def);
        let parsed = struct_def
            .to_value(&mut VecDeque::from(test_record_felts()))
            .unwrap();
        println!("{:?}", parsed);
    }
}
