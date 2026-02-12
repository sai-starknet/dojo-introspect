pub mod contract;
pub mod events;
pub mod serde;
#[cfg(test)]
mod tests;
pub use contract::{DojoSchemaFetcher, DojoSchemaFetcherError};
pub use serde::{DojoSchema, DojoSerde};
