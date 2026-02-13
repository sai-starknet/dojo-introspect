use starknet::{ClassHash, ContractAddress};
use crate::types::Struct;

#[derive(Drop, starknet::Event)]
pub struct ModelRegistered {
    #[key]
    pub name: ByteArray,
    #[key]
    pub namespace: ByteArray,
    pub class_hash: ClassHash,
    pub address: ContractAddress,
}

#[derive(Drop, starknet::Event)]
pub struct ModelWithSchemaRegistered {
    #[key]
    pub name: ByteArray,
    #[key]
    pub namespace: ByteArray,
    pub schema: Struct,
}

#[derive(Drop, starknet::Event)]
pub struct ModelUpgraded {
    #[key]
    pub selector: felt252,
    pub class_hash: ClassHash,
    pub address: ContractAddress,
    pub prev_address: ContractAddress,
}

#[derive(Drop, starknet::Event)]
pub struct EventRegistered {
    #[key]
    pub name: ByteArray,
    #[key]
    pub namespace: ByteArray,
    pub class_hash: ClassHash,
    pub address: ContractAddress,
}

#[derive(Drop, starknet::Event)]
pub struct EventUpgraded {
    #[key]
    pub selector: felt252,
    pub class_hash: ClassHash,
    pub address: ContractAddress,
    pub prev_address: ContractAddress,
}

#[derive(Drop, starknet::Event)]
pub struct StoreSetRecord {
    #[key]
    pub selector: felt252,
    #[key]
    pub entity_id: felt252,
    pub keys: Span<felt252>,
    pub values: Span<felt252>,
}

#[derive(Drop, starknet::Event)]
pub struct StoreUpdateRecord {
    #[key]
    pub selector: felt252,
    #[key]
    pub entity_id: felt252,
    pub values: Span<felt252>,
}

#[derive(Drop, starknet::Event)]
pub struct StoreUpdateMember {
    #[key]
    pub selector: felt252,
    #[key]
    pub entity_id: felt252,
    #[key]
    pub member_selector: felt252,
    pub values: Span<felt252>,
}

#[derive(Drop, starknet::Event)]
pub struct StoreDelRecord {
    #[key]
    pub selector: felt252,
    #[key]
    pub entity_id: felt252,
}

#[derive(Drop, starknet::Event)]
pub struct EventEmitted {
    #[key]
    pub selector: felt252,
    #[key]
    pub system_address: ContractAddress,
    pub keys: Span<felt252>,
    pub values: Span<felt252>,
}

