use crate::structs::{ClaimStatus, DelegationContractInfo};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait StorageModule {
    #[view(getDelegationAddressesList)]
    #[storage_mapper("delegationAddressesList")]
    fn delegation_addresses_list(&self) -> LinkedListMapper<ManagedAddress>;

    #[view(getDelegationClaimStatus)]
    #[storage_mapper("delegationClaimStatus")]
    fn delegation_claim_status(&self) -> SingleValueMapper<ClaimStatus>;

    #[view(maxDelegationAddresses)]
    #[storage_mapper("maxDelegationAddresses")]
    fn max_delegation_addresses(&self) -> SingleValueMapper<usize>;

    #[view(maxSelectedProviders)]
    #[storage_mapper("maxSelectedProviders")]
    fn max_selected_providers(&self) -> SingleValueMapper<BigUint>;

    #[view(getDelegationContractInfo)]
    #[storage_mapper("delegationContractInfo")]
    fn delegation_contract_data(
        &self,
        contract_address: &ManagedAddress,
    ) -> SingleValueMapper<DelegationContractInfo<Self::Api>>;
}
