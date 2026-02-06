use starknet_types_core::felt::Felt;
use thiserror::Error;

pub type DojoSerdeResult<T> = Result<T, DojoSerdeError>;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DojoSerdeError {
    #[error(transparent)]
    Decode(#[from] introspect_types::DecodeError),
    
}
