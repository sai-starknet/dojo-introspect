use introspect_types::DecodeError;
use starknet::providers::ProviderError;

#[derive(Debug, thiserror::Error)]
pub enum DojoIntrospectError {
    #[error("provider error: {0}")]
    ProviderError(#[from] ProviderError),
    #[error("invalid legacy response")]
    InvalidLegacyResponse,
    #[error("invalid schema")]
    InvalidSchema,
    #[error("decode error: {0}")]
    DecoderError(#[from] DecodeError),
    #[error("Invalid model tag format: {0}. Expected format is 'namespace-name'")]
    InvalidTagFormat(String),
}

pub type DojoIntrospectResult<T> = Result<T, DojoIntrospectError>;
