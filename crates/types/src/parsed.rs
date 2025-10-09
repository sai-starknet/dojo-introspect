use std::collections::VecDeque;

use starknet_types_core::felt::Felt;

#[derive(Debug)]
pub enum Parsed {
    None,
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    U256(U256),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Felt252(Felt),
    ClassHash(Felt),
    ContractAddress(Felt),
    EthAddress(Felt),
    Struct(Struct),
    Enum(Box<Enum>),
    Tuple(Vec<Parsed>),
    Array(Vec<Parsed>),
    ByteArray(String),
    FixedArray(Vec<Parsed>),
}

#[derive(Debug)]
pub struct U256 {
    pub low: u128,
    pub high: u128,
}

#[derive(Debug)]
pub struct Member {
    pub name: String,
    pub attrs: Vec<String>,
    pub value: Parsed,
}

#[derive(Debug)]
pub struct Struct {
    pub name: String,
    pub attrs: Vec<String>,
    pub children: Vec<Member>,
}
#[derive(Debug)]
pub struct Enum {
    pub name: String,
    pub attrs: Vec<String>,
    pub variant: String,
    pub value: Parsed,
}

pub trait Parse {
    type Parsed;
    fn parse(&self, data: &mut VecDeque<Felt>) -> Option<Self::Parsed>;
}
