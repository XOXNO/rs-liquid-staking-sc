use crate::structs::{DelegationContractInfo, ScoringConfig};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait StorageModule {
    #[view(getDelegationAddressesList)]
    #[storage_mapper("delegationAddressesMap")]
    fn delegation_addresses_list(&self) -> SetMapper<ManagedAddress>;

    #[view(getDelegationContractInfo)]
    #[storage_mapper("delegationContractInfo")]
    fn delegation_contract_data(
        &self,
        contract_address: &ManagedAddress,
    ) -> SingleValueMapper<DelegationContractInfo<Self::Api>>;

    #[view(getManagers)]
    #[storage_mapper("managers")]
    fn managers(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getScoringConfig)]
    #[storage_mapper("scoringConfig")]
    fn scoring_config(&self) -> SingleValueMapper<ScoringConfig>;
}
