use crate::{DojoIntrospectError, DojoIntrospectResult, DojoSchema, DojoSerde};
use async_trait::async_trait;
use introspect_types::CairoDeserialize;
use num_traits::One;
use starknet::core::types::{BlockId, BlockTag, FunctionCall, StarknetError};
use starknet::macros::selector;
use starknet::providers::{Provider, ProviderError};
use starknet_types_core::felt::Felt;

const SCHEMA_ENTRYPOINT_SELECTOR: Felt = selector!("schema");
const USE_LEGACY_STORAGE_ENTRYPOINT_SELECTOR: Felt = selector!("use_legacy_storage");

/// Makes a call to a contract's entrypoint with an empty calldata.
async fn empty_call(
    provider: &impl Provider,
    contract_address: Felt,
    entry_point: Felt,
) -> Result<Vec<Felt>, ProviderError> {
    let call = FunctionCall {
        contract_address,
        entry_point_selector: entry_point,
        calldata: vec![],
    };

    Ok(provider
        .call(call, BlockId::Tag(BlockTag::PreConfirmed))
        .await?)
}

/// Determines is a contract is using legacy storage.
///
/// Every model deployed prior `1.7.0` is considered to be using legacy storage.
/// Since `1.7.0`, the user can opt-in to use the legacy storage for backwards compatibility.
///
/// Therefore, if the entrypoint is not found, we assume the contract is using legacy storage. New models deployed after `1.7.0` exposes a new entrypoint to determine if the contract is using legacy storage.
async fn is_legacy(provider: &impl Provider, contract_address: Felt) -> DojoIntrospectResult<bool> {
    let legacy_call = empty_call(
        provider,
        contract_address,
        USE_LEGACY_STORAGE_ENTRYPOINT_SELECTOR,
    );
    match legacy_call.await {
        Ok(felts) => match felts.len() {
            1 => Ok(felts[0].is_one()),
            _ => Err(DojoIntrospectError::InvalidLegacyResponse),
        },
        Err(ProviderError::StarknetError(StarknetError::EntrypointNotFound)) => Ok(true),
        Err(err) => Err(DojoIntrospectError::ProviderError(err)),
    }
}

#[async_trait]
pub trait DojoSchemaFetcher {
    async fn schema(&self, contract_address: Felt) -> DojoIntrospectResult<DojoSchema>;
}

#[async_trait]
impl<P> DojoSchemaFetcher for P
where
    P: Provider + Send + Sync,
{
    async fn schema(&self, contract_address: Felt) -> DojoIntrospectResult<DojoSchema> {
        let schema_call = empty_call(self, contract_address, SCHEMA_ENTRYPOINT_SELECTOR).await;
        let legacy_call = is_legacy(self, contract_address).await;

        let legacy = legacy_call?;
        let schema_call_result = match schema_call {
            Ok(felts) => felts,
            Err(err) => return Err(DojoIntrospectError::ProviderError(err)),
        };
        let mut deserializer = DojoSerde::new_from_source(schema_call_result, legacy);
        DojoSchema::deserialize(&mut deserializer).map_err(Into::into)
    }
}
