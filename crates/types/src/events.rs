use crate::DojoSerde;
use introspect_types::deserialize::CairoDeserializer;
use introspect_types::{
    CairoDeserialize, CairoEvent, CairoSerde, DecodeResult, FeltSource, TypeDef,
    cairo_event_name_and_selector,
};
use starknet_types_core::felt::Felt;

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

macro_rules! impl_dojo_event_from {
    ($( $variant:ident ( $ty:ty ) ),+ $(,)?) => {
        $(
            impl From<$ty> for DojoEvent {
                fn from(value: $ty) -> Self {
                    DojoEvent::$variant(value)
                }
            }
        )+
    };
}

impl_dojo_event_from!(
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
);

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

impl CairoEvent<DojoEventSerde> for ModelRegistered {
    cairo_event_name_and_selector!("ModelRegistered");
    fn deserialize_event<K: FeltSource, D: FeltSource>(
        keys: &mut K,
        data: &mut D,
    ) -> DecodeResult<Self> {
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
    fn deserialize_and_verify_event_enum<K: FeltSource, E: FeltSource, T: From<Self>>(
        keys: &mut K,
        data: &mut E,
    ) -> DecodeResult<T> {
        Self::deserialize_and_verify_event(keys, data).map(Into::into)
    }
}

#[derive(Debug)]
pub struct ModelWithSchemaRegistered {
    pub name: String,
    pub namespace: String,
    pub schema: TypeDef,
}

impl CairoEvent<DojoEventSerde> for ModelWithSchemaRegistered {
    cairo_event_name_and_selector!("ModelWithSchemaRegistered");
    fn deserialize_event<K: FeltSource, D: FeltSource>(
        keys: &mut K,
        data: &mut D,
    ) -> DecodeResult<Self> {
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

impl CairoEvent<DojoEventSerde> for ModelUpgraded {
    cairo_event_name_and_selector!("ModelUpgraded");
    fn deserialize_event<K: FeltSource, D: FeltSource>(
        keys: &mut K,
        data: &mut D,
    ) -> DecodeResult<Self> {
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

impl CairoEvent<DojoEventSerde> for EventRegistered {
    cairo_event_name_and_selector!("EventRegistered");
    fn deserialize_event<K: FeltSource, D: FeltSource>(
        keys: &mut K,
        data: &mut D,
    ) -> DecodeResult<Self> {
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

impl CairoEvent<DojoEventSerde> for EventUpgraded {
    cairo_event_name_and_selector!("EventUpgraded");
    fn deserialize_event<K: FeltSource, D: FeltSource>(
        keys: &mut K,
        data: &mut D,
    ) -> DecodeResult<Self> {
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

impl CairoEvent<DojoEventSerde> for StoreSetRecord {
    cairo_event_name_and_selector!("StoreSetRecord");
    fn deserialize_event<K: FeltSource, D: FeltSource>(
        keys: &mut K,
        data: &mut D,
    ) -> DecodeResult<Self> {
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

impl CairoEvent<DojoEventSerde> for StoreUpdateRecord {
    cairo_event_name_and_selector!("StoreUpdateRecord");
    fn deserialize_event<K: FeltSource, D: FeltSource>(
        keys: &mut K,
        data: &mut D,
    ) -> DecodeResult<Self> {
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

impl CairoEvent<DojoEventSerde> for StoreUpdateMember {
    cairo_event_name_and_selector!("StoreUpdateMember");
    fn deserialize_event<K: FeltSource, D: FeltSource>(
        keys: &mut K,
        data: &mut D,
    ) -> DecodeResult<Self> {
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

impl CairoEvent<DojoEventSerde> for StoreDelRecord {
    cairo_event_name_and_selector!("StoreDelRecord");
    fn deserialize_event<K: FeltSource, D: FeltSource>(
        keys: &mut K,
        _data: &mut D,
    ) -> DecodeResult<Self> {
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

impl CairoEvent<DojoEventSerde> for EventEmitted {
    cairo_event_name_and_selector!("EventEmitted");
    fn deserialize_event<K: FeltSource, D: FeltSource>(
        keys: &mut K,
        data: &mut D,
    ) -> DecodeResult<Self> {
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
