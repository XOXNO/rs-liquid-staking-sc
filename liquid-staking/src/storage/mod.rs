use crate::structs::{DelegationContractData, ScoringConfig, State};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait StorageModule {
    #[view(getDelegationAddressesList)]
    #[storage_mapper("delegationAddressesMap")]
    fn delegation_addresses_list(&self) -> SetMapper<ManagedAddress>;

    #[view(getUnDelegationAddressesList)]
    #[storage_mapper("unDelegationAddressesMap")]
    fn un_delegation_addresses_list(&self) -> SetMapper<ManagedAddress>;

    #[view(getDelegationContractInfo)]
    #[storage_mapper("delegationContract")]
    fn delegation_contract_data(
        &self,
        contract_address: &ManagedAddress,
    ) -> SingleValueMapper<DelegationContractData<Self::Api>>;

    #[view(getManagers)]
    #[storage_mapper("managers")]
    fn managers(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getScoringConfig)]
    #[storage_mapper("scoringConfig")]
    fn scoring_config(&self) -> SingleValueMapper<ScoringConfig>;

    #[view(fees)]
    #[storage_mapper("fees")]
    fn fees(&self) -> SingleValueMapper<BigUint>;

    #[view(getAccumulatorContract)]
    #[storage_mapper("accumulatorContract")]
    fn accumulator_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getState)]
    #[storage_mapper("state")]
    fn state(&self) -> SingleValueMapper<State>;

    #[view(getLsTokenId)]
    #[storage_mapper("lsTokenId")]
    fn ls_token(&self) -> FungibleTokenMapper<Self::Api>;

    #[view(getLsSupply)]
    #[storage_mapper("lsTokenSupply")]
    fn ls_token_supply(&self) -> SingleValueMapper<BigUint>;

    #[view(getVirtualEgldReserve)]
    #[storage_mapper("virtualEgldReserve")]
    fn virtual_egld_reserve(&self) -> SingleValueMapper<BigUint>;

    #[view(getFeesReserve)]
    #[storage_mapper("feesReserve")]
    fn fees_reserve(&self) -> SingleValueMapper<BigUint>;

    #[view(getTotalWithdrawnEgld)]
    #[storage_mapper("totalWithdrawnEgld")]
    fn total_withdrawn_egld(&self) -> SingleValueMapper<BigUint>;

    #[view(getUnstakeTokenId)]
    #[storage_mapper("unstakeTokenId")]
    fn unstake_token(&self) -> NonFungibleTokenMapper<Self::Api>;

    #[view(getPendingEGLDForDelegate)]
    #[storage_mapper("pendingEgld")]
    fn pending_egld(&self) -> SingleValueMapper<BigUint>;

    #[view(getPendingEGLDForUnDelegate)]
    #[storage_mapper("pendingEgldForUnstake")]
    fn pending_egld_for_unstake(&self) -> SingleValueMapper<BigUint>;

    #[view(getPendingEgldForUnbond)]
    #[storage_mapper("pendingEgldForUnbond")]
    fn pending_egld_for_unbond(&self) -> SingleValueMapper<BigUint>;

    #[view(getUnstakeTokenNonce)]
    #[storage_mapper("unstakeTokenNonce")]
    fn unstake_token_nonce(&self, epoch: u64) -> SingleValueMapper<u64>;

    #[view(maxDelegationAddresses)]
    #[storage_mapper("maxDelegationAddresses")]
    fn max_delegation_addresses(&self) -> SingleValueMapper<usize>;

    #[view(maxSelectedProviders)]
    #[storage_mapper("maxSelectedProviders")]
    fn max_selected_providers(&self) -> SingleValueMapper<BigUint>;

    #[view(unbondPeriod)]
    #[storage_mapper("unbondPeriod")]
    fn unbond_period(&self) -> SingleValueMapper<u64>;
}
