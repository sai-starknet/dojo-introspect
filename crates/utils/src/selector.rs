use anyhow::{anyhow, Result};
use cainome_cairo_serde::{ByteArray, CairoSerde};
use starknet_crypto::poseidon_hash_many;
use starknet_types_core::felt::Felt;

pub const TAG_SEPARATOR: char = '-';

pub fn compute_bytearray_hash(value: &str) -> Felt {
    let ba = ByteArray::from_string(value).unwrap_or_else(|_| panic!("Invalid ByteArray: {value}"));
    poseidon_hash_many(&ByteArray::cairo_serialize(&ba))
}

pub fn compute_selector_from_dojo_tag(tag: &str) -> Felt {
    let (namespace, name) =
        split_tag(tag).unwrap_or_else(|_| panic!("Invalid tag to split: {tag}"));
    compute_selector_from_namespace_and_name(&namespace, &name)
}

pub fn compute_selector_from_namespace_and_name(namespace: &str, name: &str) -> Felt {
    poseidon_hash_many(&[
        compute_bytearray_hash(namespace),
        compute_bytearray_hash(name),
    ])
}

/// Get the namespace and the name of a world element from its tag.
pub fn split_tag(tag: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = tag.split(TAG_SEPARATOR).collect();
    match parts.len() {
        2 => Ok((parts[0].to_string(), parts[1].to_string())),
        _ => Err(anyhow!(
            "Unexpected tag. Expected format: <NAMESPACE>{TAG_SEPARATOR}<NAME> or <NAME>"
        )),
    }
}
