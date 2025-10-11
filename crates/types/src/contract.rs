use crate::DojoTypeDefSerde;
use introspect_types::StructDef;
use num_traits::One;
use starknet::core::types::StarknetError;
use starknet::macros::selector;
use starknet::{
    core::types::{BlockId, BlockTag, FunctionCall},
    providers::{Provider, ProviderError},
};
use starknet_types_core::felt::Felt;
use std::future::Future;

const SCHEMA_ENTRYPOINT_SELECTOR: Felt = selector!("schema");
const USE_LEGACY_STORAGE_ENTRYPOINT_SELECTOR: Felt = selector!("use_legacy_storage");

#[derive(Debug, thiserror::Error)]
pub enum DojoSchemaFetcherError {
    #[error("provider error: {0}")]
    ProviderError(#[from] ProviderError),
    #[error("invalid legacy response")]
    InvalidLegacyResponse,
    #[error("invalid schema")]
    InvalidSchema,
}

pub trait DojoSchemaFetcher {
    fn empty_call(
        &self,
        contract_address: Felt,
        entry_point: Felt,
    ) -> impl Future<Output = Result<Vec<Felt>, ProviderError>>;
    fn is_legacy(
        &self,
        contract_address: Felt,
    ) -> impl Future<Output = Result<bool, DojoSchemaFetcherError>>;
    fn schema(
        &self,
        contract_address: Felt,
    ) -> impl Future<Output = Result<StructDef, DojoSchemaFetcherError>>;
}

impl<P> DojoSchemaFetcher for P
where
    P: Provider,
{
    async fn empty_call(
        &self,
        contract_address: Felt,
        entry_point: Felt,
    ) -> Result<Vec<Felt>, ProviderError> {
        let call = FunctionCall {
            contract_address,
            entry_point_selector: entry_point,
            calldata: vec![],
        };
        Ok(self
            .call(call, BlockId::Tag(BlockTag::PreConfirmed))
            .await?)
    }

    async fn is_legacy(&self, contract_address: Felt) -> Result<bool, DojoSchemaFetcherError> {
        let legacy_call = self.empty_call(contract_address, USE_LEGACY_STORAGE_ENTRYPOINT_SELECTOR);
        match legacy_call.await {
            Ok(felts) => match felts.len() {
                1 => Ok(felts[0].is_one()),
                _ => Err(DojoSchemaFetcherError::InvalidLegacyResponse),
            },
            Err(ProviderError::StarknetError(StarknetError::EntrypointNotFound)) => Ok(false),
            Err(err) => Err(DojoSchemaFetcherError::ProviderError(err)),
        }
    }

    async fn schema(&self, contract_address: Felt) -> Result<StructDef, DojoSchemaFetcherError> {
        let schema_call = self.empty_call(contract_address, SCHEMA_ENTRYPOINT_SELECTOR);
        let legacy_call = self.is_legacy(contract_address);
        let legacy = legacy_call.await?;
        let schema_call_result = match schema_call.await {
            Ok(felts) => felts,
            Err(err) => return Err(DojoSchemaFetcherError::ProviderError(err)),
        };
        StructDef::dojo_deserialize(&mut schema_call_result.into_iter(), legacy)
            .ok_or(DojoSchemaFetcherError::InvalidSchema)
    }
}
