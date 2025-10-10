use introspect_types::read_serialized_felt_array;
use starknet::core::{codec::Decode, types::ByteArray};
use starknet_types_core::felt::Felt;
use std::slice::Iter;

#[allow(non_upper_case_globals)]
pub mod selectors {
    use starknet::macros::selector;
    use starknet_types_core::felt::Felt;
    pub const ModelRegistered: Felt = selector!("ModelRegistered");
    pub const ModelWithSchemaRegistered: Felt = selector!("ModelWithSchemaRegistered");
    pub const ModelUpgraded: Felt = selector!("ModelUpgraded");
    pub const EventRegistered: Felt = selector!("EventRegistered");
    pub const EventUpgraded: Felt = selector!("EventUpgraded");
    pub const StoreSetRecord: Felt = selector!("StoreSetRecord");
    pub const StoreUpdateRecord: Felt = selector!("StoreUpdateRecord");
    pub const StoreUpdateMember: Felt = selector!("StoreUpdateMember");
    pub const StoreDelRecord: Felt = selector!("StoreDelRecord");
    pub const EventEmitted: Felt = selector!("EventEmitted");
}

fn decode_byte_array_to_string(data: &mut Iter<Felt>) -> Option<String> {
    ByteArray::decode_iter(data).ok()?.try_into().ok()
}

pub struct ModelRegistered {
    pub name: String,
    pub namespace: String,
    pub class_hash: Felt,
    pub address: Felt,
}

impl ModelRegistered {
    pub fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
        let mut keys = keys.iter();
        let mut data = data.into_iter();
        let name = decode_byte_array_to_string(&mut keys)?;
        let namespace = decode_byte_array_to_string(&mut keys)?;
        let class_hash = data.next()?;
        let address = data.next()?;
        match (keys.next(), data.next()) {
            (None, None) => Some(Self {
                name,
                namespace,
                class_hash,
                address,
            }),
            _ => return None,
        }
    }
}

pub struct ModelWithSchemaRegistered {
    pub name: String,
    pub namespace: String,
    pub schema: Vec<Felt>,
}

impl ModelWithSchemaRegistered {
    pub fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
        let mut keys = keys.iter();
        let mut data = data.into_iter();
        let name = decode_byte_array_to_string(&mut keys)?;
        let namespace = decode_byte_array_to_string(&mut keys)?;
        let schema = read_serialized_felt_array(&mut data)?;
        match (keys.next(), data.next()) {
            (None, None) => Some(Self {
                name,
                namespace,
                schema,
            }),
            _ => return None,
        }
    }
}

pub struct ModelUpgraded {
    pub selector: Felt,
    pub class_hash: Felt,
    pub address: Felt,
    pub prev_address: Felt,
}

impl ModelUpgraded {
    pub fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
        let mut keys = keys.into_iter();
        let mut data = data.into_iter();
        let selector = keys.next()?;
        let class_hash = data.next()?;
        let address = data.next()?;
        let prev_address = data.next()?;
        match (keys.next(), data.next()) {
            (None, None) => Some(Self {
                selector,
                class_hash,
                address,
                prev_address,
            }),
            _ => return None,
        }
    }
}

pub struct EventRegistered {
    pub name: String,
    pub namespace: String,
    pub class_hash: Felt,
    pub address: Felt,
}

impl EventRegistered {
    pub fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
        let mut keys = keys.iter();
        let mut data = data.into_iter();
        let name = decode_byte_array_to_string(&mut keys)?;
        let namespace = decode_byte_array_to_string(&mut keys)?;
        let class_hash = data.next()?;
        let address = data.next()?;
        match (keys.next(), data.next()) {
            (None, None) => Some(Self {
                name,
                namespace,
                class_hash,
                address,
            }),
            _ => return None,
        }
    }
}

pub struct EventUpgraded {
    pub selector: Felt,
    pub class_hash: Felt,
    pub address: Felt,
    pub prev_address: Felt,
}

impl EventUpgraded {
    pub fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
        let mut keys = keys.into_iter();
        let mut data = data.into_iter();
        let selector = keys.next()?;
        let class_hash = data.next()?;
        let address = data.next()?;
        let prev_address = data.next()?;
        match (keys.next(), data.next()) {
            (None, None) => Some(Self {
                selector,
                class_hash,
                address,
                prev_address,
            }),
            _ => return None,
        }
    }
}

pub struct StoreSetRecord {
    pub selector: Felt,
    pub entity_id: Felt,
    pub keys: Vec<Felt>,
    pub values: Vec<Felt>,
}

impl StoreSetRecord {
    pub fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
        let mut keys = keys.into_iter();
        let mut data = data.into_iter();
        let selector = keys.next()?;
        let entity_id = keys.next()?;
        let record_keys = read_serialized_felt_array(&mut data)?;
        let record_values = read_serialized_felt_array(&mut data)?;
        match (keys.next(), data.next()) {
            (None, None) => Some(Self {
                selector,
                entity_id,
                keys: record_keys,
                values: record_values,
            }),
            _ => return None,
        }
    }
}

pub struct StoreUpdateRecord {
    pub selector: Felt,
    pub entity_id: Felt,
    pub values: Vec<Felt>,
}

impl StoreUpdateRecord {
    pub fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
        let mut keys = keys.into_iter();
        let mut data = data.into_iter();
        let selector = keys.next()?;
        let entity_id = keys.next()?;
        let record_values = read_serialized_felt_array(&mut data)?;
        match (keys.next(), data.next()) {
            (None, None) => Some(Self {
                selector,
                entity_id,
                values: record_values,
            }),
            _ => return None,
        }
    }
}

pub struct StoreUpdateMember {
    pub selector: Felt,
    pub entity_id: Felt,
    pub member_selector: Felt,
    pub values: Vec<Felt>,
}

impl StoreUpdateMember {
    pub fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
        let mut keys = keys.into_iter();
        let mut data = data.into_iter();
        let selector = keys.next()?;
        let entity_id = keys.next()?;
        let member_selector = keys.next()?;
        let member_values = read_serialized_felt_array(&mut data)?;
        match (keys.next(), data.next()) {
            (None, None) => Some(Self {
                selector,
                entity_id,
                member_selector,
                values: member_values,
            }),
            _ => return None,
        }
    }
}

pub struct StoreDelRecord {
    pub selector: Felt,
    pub entity_id: Felt,
}

impl StoreDelRecord {
    pub fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
        let mut keys = keys.into_iter();
        let selector = keys.next()?;
        let entity_id = keys.next()?;
        match (keys.next(), data.len()) {
            (None, 0) => Some(Self {
                selector,
                entity_id,
            }),
            _ => return None,
        }
    }
}

pub struct EventEmitted {
    pub selector: Felt,
    pub system_address: Felt,
    pub keys: Vec<Felt>,
    pub values: Vec<Felt>,
}

impl EventEmitted {
    pub fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
        let mut keys = keys.into_iter();
        let mut data = data.into_iter();
        let selector = keys.next()?;
        let system_address = keys.next()?;
        let event_keys = read_serialized_felt_array(&mut data)?;
        let event_values = read_serialized_felt_array(&mut data)?;
        match (keys.next(), data.next()) {
            (None, None) => Some(Self {
                selector,
                system_address,
                keys: event_keys,
                values: event_values,
            }),
            _ => return None,
        }
    }
}
