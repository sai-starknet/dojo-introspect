use crate::parsed::{self, Parse};
use crate::parsed::{Parsed, U256};
use cainome_cairo_serde::{ByteArray, Bytes31};
use num_traits::{ToPrimitive, Zero};
use starknet_types_core::felt::Felt;
use std::collections::{HashMap, VecDeque};

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

pub struct DojoStruct {
    pub name: Felt,
    pub attrs: Vec<Felt>,
    pub children: Vec<DojoMember>,
}

pub struct DojoVariant {
    pub name: String,
    pub ty: DojoTy,
}

pub struct DojoEnum {
    pub name: Felt,
    pub attrs: Vec<Felt>,
    pub variants: HashMap<Felt, DojoVariant>,
}

pub struct DojoMember {
    pub name: Felt,
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
            DojoPrimitive::U8 => Some(Parsed::U8(data.pop_front()?.to_u8()?)),
            DojoPrimitive::U16 => Some(Parsed::U16(data.pop_front()?.to_u16()?)),
            DojoPrimitive::U32 => Some(Parsed::U32(data.pop_front()?.to_u32()?)),
            DojoPrimitive::U64 => Some(Parsed::U64(data.pop_front()?.to_u64()?)),
            DojoPrimitive::U128 => Some(Parsed::U128(data.pop_front()?.to_u128()?)),
            DojoPrimitive::U256 => {
                let low = data.pop_front()?.to_u128()?;
                let high = data.pop_front()?.to_u128()?;
                Some(Parsed::U256(U256 { low, high }))
            }
            DojoPrimitive::I8 => Some(Parsed::I8(data.pop_front()?.to_i8()?)),
            DojoPrimitive::I16 => Some(Parsed::I16(data.pop_front()?.to_i16()?)),
            DojoPrimitive::I32 => Some(Parsed::I32(data.pop_front()?.to_i32()?)),
            DojoPrimitive::I64 => Some(Parsed::I64(data.pop_front()?.to_i64()?)),
            DojoPrimitive::I128 => Some(Parsed::I128(data.pop_front()?.to_i128()?)),
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
            variant: variant.name.to_string(),
            value: variant.ty.parse(data)?,
        })
    }
}

impl Parse for DojoMember {
    type Parsed = parsed::Member;
    fn parse(&self, data: &mut VecDeque<Felt>) -> Option<Self::Parsed> {
        Some(parsed::Member {
            name: self.name.to_string(),
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
    data.remove(0);
    DojoTy::deserialize(data, legacy).map(Box::new)
}

fn deserialize_fixed_array(data: &mut VecDeque<Felt>, legacy: bool) -> Option<(Box<DojoTy>, u32)> {
    data.remove(0);
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

pub trait DojoTySerde: Sized {
    fn deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self>;
}

impl DojoTySerde for DojoMember {
    fn deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self> {
        let name = data.pop_front()?;
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
        let name = data.pop_front()?;
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
        let name = data.pop_front()?.to_string();
        let ty = DojoTy::deserialize(data, legacy)?;
        Some(DojoVariant { name, ty })
    }
}

impl DojoTySerde for DojoEnum {
    fn deserialize(data: &mut VecDeque<Felt>, legacy: bool) -> Option<Self> {
        let name = data.pop_front()?;
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
        let kind = data.pop_front()?.to_string();
        if kind == "bool" {
            Some(DojoPrimitive::Bool)
        } else if kind == "u8" {
            Some(DojoPrimitive::U8)
        } else if kind == "u16" {
            Some(DojoPrimitive::U16)
        } else if kind == "u32" {
            Some(DojoPrimitive::U32)
        } else if kind == "u64" {
            Some(DojoPrimitive::U64)
        } else if kind == "u128" {
            Some(DojoPrimitive::U128)
        } else if kind == "u256" {
            Some(DojoPrimitive::U256)
        } else if kind == "i8" {
            Some(DojoPrimitive::I8)
        } else if kind == "i16" {
            Some(DojoPrimitive::I16)
        } else if kind == "i32" {
            Some(DojoPrimitive::I32)
        } else if kind == "i64" {
            Some(DojoPrimitive::I64)
        } else if kind == "i128" {
            Some(DojoPrimitive::I128)
        } else if kind == "felt252" {
            Some(DojoPrimitive::Felt252)
        } else if kind == "ClassHash" || kind == "starknet::ClassHash" {
            Some(DojoPrimitive::ClassHash)
        } else if kind == "ContractAddress" || kind == "starknet::ContractAddress" {
            Some(DojoPrimitive::ContractAddress)
        } else if kind == "EthAddress" || kind == "starknet::EthAddress" {
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
