use crate::parsed::{self, Parse};
use crate::parsed::{Parsed, U256};
use cainome_cairo_serde::{ByteArray, Bytes31};
use num_traits::{One, ToPrimitive, Zero};
use starknet_types_core::felt::Felt;
use std::collections::{HashMap, VecDeque};
#[derive(Debug)]
pub enum DojoTy {
    None,
    Primitive(DojoPrimitive),
    Struct(DojoStruct),
    Enum(DojoEnum),
    Tuple(Vec<DojoTy>),
    Array(Box<DojoTy>),
    ByteArray,
    FixedArray((Box<DojoTy>, u32)),
}

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

#[derive(Debug)]
pub enum DojoPrimitive {
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    I8,
    I16,
    I32,
    I64,
    I128,
    Felt252,
    ClassHash,
    ContractAddress,
    EthAddress,
}

#[derive(Debug)]
pub struct DojoStruct {
    pub name: String,
    pub attrs: Vec<Felt>,
    pub children: Vec<DojoMember>,
}

#[derive(Debug)]
pub struct DojoVariant {
    pub name: String,
    pub ty: DojoTy,
}

#[derive(Debug)]
pub struct DojoEnum {
    pub name: String,
    pub attrs: Vec<Felt>,
    pub variants: HashMap<Felt, DojoVariant>,
}

#[derive(Debug)]
pub struct DojoMember {
    pub name: String,
    pub attrs: Vec<Felt>,
    pub ty: DojoTy,
}

fn parse_byte_array(data: &mut VecDeque<Felt>) -> Option<Parsed> {
    let len = data.pop_front()?.to_usize()?;

    let mut bytes: Vec<Bytes31> = Vec::with_capacity(len);
    for _ in 0..len {
        bytes.push(Bytes31::new(data.pop_front()?).ok()?);
    }
    let pending_word = data.pop_front()?;
    let pending_word_len = data.pop_front()?.to_usize()?;

    Some(Parsed::ByteArray(
        ByteArray {
            data: bytes,
            pending_word,
            pending_word_len,
        }
        .to_string()
        .ok()?,
    ))
}

fn parse_tuple(tys: &Vec<DojoTy>, data: &mut VecDeque<Felt>) -> Option<Parsed> {
    let mut elements = Vec::with_capacity(tys.len());
    for ty in tys {
        elements.push(ty.parse(data)?);
    }
    Some(Parsed::Tuple(elements))
}

fn parse_array(ty: &DojoTy, data: &mut VecDeque<Felt>) -> Option<Vec<Parsed>> {
    let len = data.pop_front()?.to_usize()?;
    parse_array_internal(ty, len, data)
}

fn parse_fixed_array(ty: &DojoTy, size: usize, data: &mut VecDeque<Felt>) -> Option<Vec<Parsed>> {
    parse_array_internal(ty, size, data)
}

fn parse_array_internal(
    ty: &DojoTy,
    size: usize,
    data: &mut VecDeque<Felt>,
) -> Option<Vec<Parsed>> {
    let mut elements = Vec::with_capacity(size);
    for _ in 0..size {
        elements.push(ty.parse(data)?);
    }
    Some(elements)
}

impl Parse for DojoPrimitive {
    type Parsed = Parsed;
    fn parse(&self, data: &mut VecDeque<Felt>) -> Option<Self::Parsed> {
        match self {
            DojoPrimitive::Bool => Some(Parsed::Bool(!data.pop_front()?.is_zero())),
            DojoPrimitive::U8 => Some(Parsed::U8(data.pop_front()?.try_into().ok()?)),
            DojoPrimitive::U16 => Some(Parsed::U16(data.pop_front()?.try_into().ok()?)),
            DojoPrimitive::U32 => Some(Parsed::U32(data.pop_front()?.try_into().ok()?)),
            DojoPrimitive::U64 => Some(Parsed::U64(data.pop_front()?.try_into().ok()?)),
            DojoPrimitive::U128 => Some(Parsed::U128(data.pop_front()?.try_into().ok()?)),
            DojoPrimitive::U256 => {
                let low = data.pop_front()?.try_into().ok()?;
                let high = data.pop_front()?.try_into().ok()?;
                Some(Parsed::U256(U256 { low, high }))
            }
            DojoPrimitive::I8 => Some(Parsed::I8(data.pop_front()?.try_into().ok()?)),
            DojoPrimitive::I16 => Some(Parsed::I16(data.pop_front()?.try_into().ok()?)),
            DojoPrimitive::I32 => Some(Parsed::I32(data.pop_front()?.try_into().ok()?)),
            DojoPrimitive::I64 => Some(Parsed::I64(data.pop_front()?.try_into().ok()?)),
            DojoPrimitive::I128 => Some(Parsed::I128(data.pop_front()?.try_into().ok()?)),
            DojoPrimitive::Felt252 => Some(Parsed::Felt252(data.pop_front()?)),
            DojoPrimitive::ClassHash => Some(Parsed::ClassHash(data.pop_front()?)),
            DojoPrimitive::ContractAddress => Some(Parsed::ContractAddress(data.pop_front()?)),
            DojoPrimitive::EthAddress => Some(Parsed::EthAddress(data.pop_front()?)),
        }
    }
}

impl Parse for DojoStruct {
    type Parsed = parsed::Struct;
    fn parse(&self, data: &mut VecDeque<Felt>) -> Option<Self::Parsed> {
        Some(parsed::Struct {
            name: self.name.to_string(),
            attrs: self.attrs.iter().map(|a| a.to_string()).collect(),
            children: self
                .children
                .iter()
                .map(|member| member.parse(data))
                .collect::<Option<Vec<parsed::Member>>>()?,
        })
    }
}

impl Parse for DojoEnum {
    type Parsed = parsed::Enum;
    fn parse(&self, data: &mut VecDeque<Felt>) -> Option<Self::Parsed> {
        let selector = data.pop_front()?;
        let variant = self.variants.get(&selector)?;
        Some(parsed::Enum {
            name: self.name.to_string(),
            attrs: self.attrs.iter().map(|a| a.to_string()).collect(),
            variant: variant.name.clone(),
            value: variant.ty.parse(data)?,
        })
    }
}

impl Parse for DojoMember {
    type Parsed = parsed::Member;
    fn parse(&self, data: &mut VecDeque<Felt>) -> Option<Self::Parsed> {
        Some(parsed::Member {
            name: self.name.clone(),
            attrs: self.attrs.iter().map(|a| a.to_string()).collect(),
            value: self.ty.parse(data)?,
        })
    }
}

impl Parse for DojoTy {
    type Parsed = Parsed;
    fn parse(&self, data: &mut VecDeque<Felt>) -> Option<Self::Parsed> {
        match self {
            DojoTy::None => Some(Parsed::None),
            DojoTy::Primitive(primitive) => primitive.parse(data),
            DojoTy::Struct(dojo_struct) => dojo_struct.parse(data).map(Parsed::Struct),
            DojoTy::Enum(dojo_enum) => dojo_enum.parse(data).map(|e| Parsed::Enum(Box::new(e))),
            DojoTy::Tuple(tys) => parse_tuple(tys, data),
            DojoTy::Array(ty) => parse_array(ty, data).map(Parsed::Array),
            DojoTy::ByteArray => parse_byte_array(data),
            DojoTy::FixedArray((ty, size)) => {
                parse_fixed_array(ty, *size as usize, data).map(Parsed::FixedArray)
            }
        }
    }
}

fn deserialize_array(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Box<DojoTy>> {
    if data.pop_front()? != Felt::ONE {
        return None;
    }
    DojoTy::deserialize(data, legacy).map(Box::new)
}

fn deserialize_fixed_array(data: &mut VecDeque<Felt>, legacy: bool) -> Option<(Box<DojoTy>, u32)> {
    if data.pop_front()?.is_one() {
        return None;
    }
    let ty = DojoTy::deserialize(data, legacy).map(Box::new)?;
    let size = data.pop_front()?.to_u32()?;
    Some((ty, size))
}
fn deserialize_tuple(data: &mut VecDeque<Felt>, legacy: bool) -> Option<DojoTy> {
    let len = data.pop_front()?.to_usize()?;
    if len == 0 {
        return Some(DojoTy::None);
    }
    let mut elements = Vec::with_capacity(len);
    for _ in 0..len {
        elements.push(DojoTy::deserialize(data, legacy)?);
    }
    Some(DojoTy::Tuple(elements))
}

fn felt_to_utf8_string(felt: Felt) -> Option<String> {
    let bytes = felt.to_bytes_be();
    let first = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
    String::from_utf8(bytes[first..32].to_vec()).ok()
}

pub trait DojoTySerde: Sized {
    fn deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self>;
}

impl DojoTySerde for DojoMember {
    fn deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self> {
        let name = data.pop_front().map(felt_to_utf8_string)??;
        let attrs_len = data.pop_front()?.to_usize()?;
        let mut attrs = Vec::with_capacity(attrs_len);
        for _ in 0..attrs_len {
            attrs.push(data.pop_front()?);
        }
        let ty = DojoTy::deserialize(data, legacy)?;
        Some(DojoMember { name, attrs, ty })
    }
}

impl DojoTySerde for DojoStruct {
    fn deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self> {
        let name = data.pop_front().map(felt_to_utf8_string)??;
        let attrs_len = data.pop_front()?.to_usize()?;
        let mut attrs = Vec::with_capacity(attrs_len);
        for _ in 0..attrs_len {
            attrs.push(data.pop_front()?);
        }
        let children_len = data.pop_front()?.to_usize()?;
        let mut children = Vec::with_capacity(children_len);
        for _ in 0..children_len {
            children.push(DojoMember::deserialize(data, legacy)?);
        }
        Some(DojoStruct {
            name,
            attrs,
            children,
        })
    }
}

impl DojoTySerde for DojoVariant {
    fn deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self> {
        let name = data.pop_front().map(felt_to_utf8_string)??;
        let ty = DojoTy::deserialize(data, legacy)?;
        Some(DojoVariant { name, ty })
    }
}

impl DojoTySerde for DojoEnum {
    fn deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self> {
        let name = data.pop_front().map(felt_to_utf8_string)??;
        let attrs_len = data.pop_front()?.to_usize()?;
        let mut attrs = Vec::with_capacity(attrs_len);
        for _ in 0..attrs_len {
            attrs.push(data.pop_front()?);
        }
        let variants_len = data.pop_front()?.to_usize()?;
        let mut variants = HashMap::with_capacity(variants_len);
        let legacy_mod: usize = (!legacy).into();
        for i in 0..variants_len {
            let variant = DojoVariant::deserialize(data, legacy)?;
            variants.insert((i + legacy_mod).into(), variant);
        }
        Some(DojoEnum {
            name,
            attrs,
            variants,
        })
    }
}

impl DojoTySerde for DojoPrimitive {
    fn deserialize(data: &mut VecDeque<Felt>, _legacy: bool) -> Option<Self> {
        let kind = data.pop_front()?;
        if kind == primitive::BOOL_FELT {
            Some(DojoPrimitive::Bool)
        } else if kind == primitive::U8_FELT {
            Some(DojoPrimitive::U8)
        } else if kind == primitive::U16_FELT {
            Some(DojoPrimitive::U16)
        } else if kind == primitive::U32_FELT {
            Some(DojoPrimitive::U32)
        } else if kind == primitive::U64_FELT {
            Some(DojoPrimitive::U64)
        } else if kind == primitive::U128_FELT {
            Some(DojoPrimitive::U128)
        } else if kind == primitive::U256_FELT {
            Some(DojoPrimitive::U256)
        } else if kind == primitive::I8_FELT {
            Some(DojoPrimitive::I8)
        } else if kind == primitive::I16_FELT {
            Some(DojoPrimitive::I16)
        } else if kind == primitive::I32_FELT {
            Some(DojoPrimitive::I32)
        } else if kind == primitive::I64_FELT {
            Some(DojoPrimitive::I64)
        } else if kind == primitive::I128_FELT {
            Some(DojoPrimitive::I128)
        } else if kind == primitive::FELT252_FELT {
            Some(DojoPrimitive::Felt252)
        } else if kind == primitive::CLASS_HASH_FELT || kind == primitive::STARKNET_CLASS_HASH {
            Some(DojoPrimitive::ClassHash)
        } else if kind == primitive::CONTRACT_ADDRESS_FELT
            || kind == primitive::STARKNET_CONTRACT_ADDRESS
        {
            Some(DojoPrimitive::ContractAddress)
        } else if kind == primitive::ETH_ADDRESS_FELT || kind == primitive::STARKNET_CLASS_HASH {
            Some(DojoPrimitive::EthAddress)
        } else {
            None
        }
    }
}

impl DojoTySerde for DojoTy {
    fn deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self> {
        let kind = data.pop_front()?.to_u32()?;
        match kind {
            0 => DojoPrimitive::deserialize(data, legacy).map(DojoTy::Primitive),
            1 => DojoStruct::deserialize(data, legacy).map(DojoTy::Struct),
            2 => DojoEnum::deserialize(data, legacy).map(DojoTy::Enum),
            3 => deserialize_tuple(data, legacy),
            4 => deserialize_array(data, legacy).map(DojoTy::Array),
            5 => Some(DojoTy::ByteArray),
            6 => deserialize_fixed_array(data, legacy).map(DojoTy::FixedArray),
            _ => None,
        }
    }
}

mod tests {

    use std::collections::VecDeque;

    use super::{DojoStruct, DojoTySerde};
    use crate::parsed::Parse;
    use starknet_types_core::felt::Felt;
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
        let dojo_struct = DojoStruct::deserialize(&mut data, true).unwrap();

        println!("{:?}", dojo_struct);
        let parsed = dojo_struct
            .parse(&mut VecDeque::from(test_record_felts()))
            .unwrap();
        println!("{:?}", parsed);
    }
}
