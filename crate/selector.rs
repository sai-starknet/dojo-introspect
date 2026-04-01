use introspect_types::utils::string_to_cairo_serialize_byte_array;
use starknet_crypto::poseidon_hash_many;
use starknet_types_core::felt::Felt;

use crate::{DojoIntrospectError, DojoIntrospectResult};

pub const TAG_SEPARATOR: char = '-';

pub fn compute_bytearray_hash(value: &str) -> Felt {
    poseidon_hash_many(&string_to_cairo_serialize_byte_array(value))
}

pub fn compute_selector_from_dojo_tag(tag: &str) -> DojoIntrospectResult<Felt> {
    split_tag(tag)
        .map(|(namespace, name)| compute_selector_from_namespace_and_name(namespace, name))
}

pub fn compute_selector_from_namespace_and_name(namespace: &str, name: &str) -> Felt {
    poseidon_hash_many(&[
        compute_bytearray_hash(namespace),
        compute_bytearray_hash(name),
    ])
}

/// Get the namespace and the name of a world element from its tag.
pub fn split_tag(tag: &str) -> DojoIntrospectResult<(&str, &str)> {
    let parts: Vec<&str> = tag.split(TAG_SEPARATOR).collect();
    match parts.len() {
        2 => Ok((parts[0], parts[1])),
        _ => Err(DojoIntrospectError::InvalidTagFormat(tag.to_string())),
    }
}
