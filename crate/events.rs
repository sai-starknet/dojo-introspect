use crate::{DojoSchema, DojoSerde};
use introspect_rust_macros::EnumFrom;
use introspect_types::deserialize::CairoDeserializer;
use introspect_types::{
    CairoDeserialize, CairoEvent, CairoEventInfo, CairoSerde, DecodeResult, FeltSource,
};
use sai_felt::Felt;

#[derive(Debug, EnumFrom)]
pub enum DojoEvent {
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

pub struct DojoEventSerde;

// pub trait DojoEvent
// where
//     Self: Sized,
// {
//     const NAME: &'static str;
//     const SELECTOR: Felt;
//     fn deserialize<K: FeltSource, D: FeltSource>(keys: &mut K, data: &mut D) -> DecodeResult<Self>;
//     fn deserialize_complete(keys: Vec<Felt>, data: Vec<Felt>) -> DecodeResult<Self> {
//         let mut keys = keys.into_source();
//         let mut data = data.into_source();
//         let result = Self::deserialize(&mut keys, &mut data)?;
//         match (keys.next(), data.next()) {
//             (Err(DecodeError::Eof), Err(DecodeError::Eof)) => Ok(result),
//             _ => Err(DecodeError::NotEof),
//         }
//     }
// }

#[derive(Debug)]
pub struct ModelRegistered {
    pub name: String,
    pub namespace: String,
    pub class_hash: Felt,
    pub address: Felt,
}

impl<D: FeltSource + CairoDeserializer> CairoEvent<D> for ModelRegistered {
    fn deserialize_event<K: FeltSource>(keys: &mut K, data: &mut D) -> DecodeResult<Self> {
        let mut keys: CairoSerde<_> = keys.into();
        let name = keys.next_string()?;
        let namespace = keys.next_string()?;
        let class_hash = data.next()?;
        let address = data.next()?;
        Ok(Self {
            name,
            namespace,
            class_hash,
            address,
        })
    }
}

#[derive(Debug)]
pub struct ModelWithSchemaRegistered {
    pub name: String,
    pub namespace: String,
    pub schema: DojoSchema,
}

impl<D: FeltSource + CairoDeserializer> CairoEvent<D> for ModelWithSchemaRegistered {
    fn deserialize_event<K: FeltSource>(keys: &mut K, data: &mut D) -> DecodeResult<Self> {
        let mut keys: CairoSerde<_> = keys.into();
        let mut data = DojoSerde::new_from_source(data, true);
        let name = keys.next_string()?;
        let namespace = keys.next_string()?;
        let schema = CairoDeserialize::deserialize(&mut data)?;
        Ok(Self {
            name,
            namespace,
            schema,
        })
    }
}

#[derive(Debug)]
pub struct ModelUpgraded {
    pub selector: Felt,
    pub class_hash: Felt,
    pub address: Felt,
    pub prev_address: Felt,
}

impl<D: FeltSource + CairoDeserializer> CairoEvent<D> for ModelUpgraded {
    fn deserialize_event<K: FeltSource>(keys: &mut K, data: &mut D) -> DecodeResult<Self> {
        let selector = keys.next()?;
        let class_hash = data.next()?;
        let address = data.next()?;
        let prev_address = data.next()?;
        Ok(Self {
            selector,
            class_hash,
            address,
            prev_address,
        })
    }
}

#[derive(Debug)]
pub struct EventRegistered {
    pub name: String,
    pub namespace: String,
    pub class_hash: Felt,
    pub address: Felt,
}

impl<D: FeltSource + CairoDeserializer> CairoEvent<D> for EventRegistered {
    fn deserialize_event<K: FeltSource>(keys: &mut K, data: &mut D) -> DecodeResult<Self> {
        let mut keys: CairoSerde<_> = keys.into();
        let name = keys.next_string()?;
        let namespace = keys.next_string()?;
        let class_hash = data.next()?;
        let address = data.next()?;
        Ok(Self {
            name,
            namespace,
            class_hash,
            address,
        })
    }
}

#[derive(Debug)]
pub struct EventUpgraded {
    pub selector: Felt,
    pub class_hash: Felt,
    pub address: Felt,
    pub prev_address: Felt,
}

impl<D: FeltSource + CairoDeserializer> CairoEvent<D> for EventUpgraded {
    fn deserialize_event<K: FeltSource>(keys: &mut K, data: &mut D) -> DecodeResult<Self> {
        let selector = keys.next()?;
        let class_hash = data.next()?;
        let address = data.next()?;
        let prev_address = data.next()?;
        Ok(Self {
            selector,
            class_hash,
            address,
            prev_address,
        })
    }
}

#[derive(Debug)]
pub struct StoreSetRecord {
    pub selector: Felt,
    pub entity_id: Felt,
    pub keys: Vec<Felt>,
    pub values: Vec<Felt>,
}

impl<D: FeltSource + CairoDeserializer> CairoEvent<D> for StoreSetRecord {
    fn deserialize_event<K: FeltSource>(keys: &mut K, data: &mut D) -> DecodeResult<Self> {
        let mut data: CairoSerde<_> = data.into();
        let selector = keys.next()?;
        let entity_id = keys.next()?;
        let record_keys: Vec<Felt> = data.next_array()?;
        let record_values: Vec<Felt> = data.next_array()?;
        Ok(Self {
            selector,
            entity_id,
            keys: record_keys,
            values: record_values,
        })
    }
}

#[derive(Debug)]
pub struct StoreUpdateRecord {
    pub selector: Felt,
    pub entity_id: Felt,
    pub values: Vec<Felt>,
}

impl<D: FeltSource + CairoDeserializer> CairoEvent<D> for StoreUpdateRecord {
    fn deserialize_event<K: FeltSource>(keys: &mut K, data: &mut D) -> DecodeResult<Self> {
        let mut data: CairoSerde<_> = data.into();
        let selector = keys.next()?;
        let entity_id = keys.next()?;
        let record_values: Vec<Felt> = data.next_array()?;
        Ok(Self {
            selector,
            entity_id,
            values: record_values,
        })
    }
}

#[derive(Debug)]
pub struct StoreUpdateMember {
    pub selector: Felt,
    pub entity_id: Felt,
    pub member_selector: Felt,
    pub values: Vec<Felt>,
}

impl<D: FeltSource + CairoDeserializer> CairoEvent<D> for StoreUpdateMember {
    fn deserialize_event<K: FeltSource>(keys: &mut K, data: &mut D) -> DecodeResult<Self> {
        let mut data: CairoSerde<_> = data.into();
        let selector = keys.next()?;
        let entity_id = keys.next()?;
        let member_selector = keys.next()?;
        let member_values: Vec<Felt> = data.next_array()?;
        Ok(Self {
            selector,
            entity_id,
            member_selector,
            values: member_values,
        })
    }
}

#[derive(Debug)]
pub struct StoreDelRecord {
    pub selector: Felt,
    pub entity_id: Felt,
}

impl<D: FeltSource + CairoDeserializer> CairoEvent<D> for StoreDelRecord {
    fn deserialize_event<K: FeltSource>(keys: &mut K, _data: &mut D) -> DecodeResult<Self> {
        let selector = keys.next()?;
        let entity_id = keys.next()?;
        Ok(Self {
            selector,
            entity_id,
        })
    }
}

#[derive(Debug)]
pub struct EventEmitted {
    pub selector: Felt,
    pub system_address: Felt,
    pub keys: Vec<Felt>,
    pub values: Vec<Felt>,
}

impl<D: FeltSource + CairoDeserializer> CairoEvent<D> for EventEmitted {
    fn deserialize_event<K: FeltSource>(keys: &mut K, data: &mut D) -> DecodeResult<Self> {
        let mut data: CairoSerde<_> = data.into();
        let selector = keys.next()?;
        let system_address = keys.next()?;
        let event_keys: Vec<Felt> = data.next_array()?;
        let event_values: Vec<Felt> = data.next_array()?;
        Ok(Self {
            selector,
            system_address,
            keys: event_keys,
            values: event_values,
        })
    }
}
impl CairoEventInfo for ModelRegistered {
    const NAME: &'static str = "ModelRegistered";
}
impl CairoEventInfo for ModelWithSchemaRegistered {
    const NAME: &'static str = "ModelWithSchemaRegistered";
}
impl CairoEventInfo for ModelUpgraded {
    const NAME: &'static str = "ModelUpgraded";
}
impl CairoEventInfo for EventRegistered {
    const NAME: &'static str = "EventRegistered";
}
impl CairoEventInfo for EventUpgraded {
    const NAME: &'static str = "EventUpgraded";
}
impl CairoEventInfo for StoreSetRecord {
    const NAME: &'static str = "StoreSetRecord";
}
impl CairoEventInfo for StoreUpdateRecord {
    const NAME: &'static str = "StoreUpdateRecord";
}
impl CairoEventInfo for StoreUpdateMember {
    const NAME: &'static str = "StoreUpdateMember";
}
impl CairoEventInfo for StoreDelRecord {
    const NAME: &'static str = "StoreDelRecord";
}
impl CairoEventInfo for EventEmitted {
    const NAME: &'static str = "EventEmitted";
}
