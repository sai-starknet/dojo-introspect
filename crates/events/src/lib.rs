use introspect_types::read_serialized_felt_array;
use starknet::core::{codec::Decode, types::ByteArray};
use starknet::macros::selector;
use starknet_types_core::felt::Felt;
use std::slice::Iter;

macro_rules! impl_dojo_event_name {
    ($name:literal) => {
        const NAME: &'static str = $name;
        const SELECTOR: Felt = selector!($name);
    };
}

pub enum DojoEvents {
    ModelRegistered(ModelRegistered),
    ModelWithSchemaRegistered(ModelWithSchemaRegistered),
    ModelUpgraded(ModelUpgraded),
    EventRegistered(EventRegistered),
    EventUpgraded(EventUpgraded),
    StoreSetRecord(StoreSetRecord),
    StoreUpdateRecord(StoreUpdateRecord),
    StoreUpdateMember(StoreUpdateMember),
    StoreDelRecord(StoreDelRecord),
    EventEmitted(EventEmitted),
}

fn decode_byte_array_to_string(data: &mut Iter<Felt>) -> Option<String> {
    ByteArray::decode_iter(data).ok()?.try_into().ok()
}

pub trait DojoEvent {
    const NAME: &'static str;
    const SELECTOR: Felt;
    fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self>
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct ModelRegistered {
    pub name: String,
    pub namespace: String,
    pub class_hash: Felt,
    pub address: Felt,
}

impl DojoEvent for ModelRegistered {
    impl_dojo_event_name!("ModelRegistered");
    fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self>
    where
        Self: Sized,
    {
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

#[derive(Debug)]
pub struct ModelWithSchemaRegistered {
    pub name: String,
    pub namespace: String,
    pub schema: Vec<Felt>,
}

impl DojoEvent for ModelWithSchemaRegistered {
    impl_dojo_event_name!("ModelWithSchemaRegistered");
    fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
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

#[derive(Debug)]
pub struct ModelUpgraded {
    pub selector: Felt,
    pub class_hash: Felt,
    pub address: Felt,
    pub prev_address: Felt,
}

impl DojoEvent for ModelUpgraded {
    impl_dojo_event_name!("ModelUpgraded");
    fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
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

#[derive(Debug)]
pub struct EventRegistered {
    pub name: String,
    pub namespace: String,
    pub class_hash: Felt,
    pub address: Felt,
}

impl DojoEvent for EventRegistered {
    impl_dojo_event_name!("EventRegistered");
    fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
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

#[derive(Debug)]
pub struct EventUpgraded {
    pub selector: Felt,
    pub class_hash: Felt,
    pub address: Felt,
    pub prev_address: Felt,
}

impl DojoEvent for EventUpgraded {
    impl_dojo_event_name!("EventUpgraded");
    fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
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

#[derive(Debug)]
pub struct StoreSetRecord {
    pub selector: Felt,
    pub entity_id: Felt,
    pub keys: Vec<Felt>,
    pub values: Vec<Felt>,
}

impl DojoEvent for StoreSetRecord {
    impl_dojo_event_name!("StoreSetRecord");
    fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
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

#[derive(Debug)]
pub struct StoreUpdateRecord {
    pub selector: Felt,
    pub entity_id: Felt,
    pub values: Vec<Felt>,
}

impl DojoEvent for StoreUpdateRecord {
    impl_dojo_event_name!("StoreUpdateRecord");
    fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
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

#[derive(Debug)]
pub struct StoreUpdateMember {
    pub selector: Felt,
    pub entity_id: Felt,
    pub member_selector: Felt,
    pub values: Vec<Felt>,
}

impl DojoEvent for StoreUpdateMember {
    impl_dojo_event_name!("StoreUpdateMember");
    fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
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

#[derive(Debug)]
pub struct StoreDelRecord {
    pub selector: Felt,
    pub entity_id: Felt,
}

impl DojoEvent for StoreDelRecord {
    impl_dojo_event_name!("StoreDelRecord");
    fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
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

#[derive(Debug)]
pub struct EventEmitted {
    pub selector: Felt,
    pub system_address: Felt,
    pub keys: Vec<Felt>,
    pub values: Vec<Felt>,
}

impl DojoEvent for EventEmitted {
    impl_dojo_event_name!("EventEmitted");
    fn new(keys: Vec<Felt>, data: Vec<Felt>) -> Option<Self> {
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
