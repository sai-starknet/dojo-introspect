use crate::types::Struct;

#[starknet::interface]
pub trait IDojoModel<TContractState> {
    fn schema(self: @TContractState) -> Struct;
    fn use_legacy_storage(self: @TContractState) -> bool;
}
