use crate::{
    structs::{State, UnstakeTokenAttributes},
    ERROR_MAX_CHANGED_DELEGATION_ADDRESSES, ERROR_MAX_SELECTED_PROVIDERS, ERROR_NOT_ACTIVE,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const MAX_PERCENTAGE: u64 = 100_000;

#[multiversx_sc::module]
pub trait ConfigModule: crate::storage::StorageModule {
    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(registerLsToken)]
    fn register_ls_token(
        &self,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        let payment_amount = self.call_value().egld_value().clone_value();
        self.ls_token().issue_and_set_all_roles(
            payment_amount,
            token_display_name,
            token_ticker,
            num_decimals,
            None,
        );
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(registerUnstakeToken)]
    fn register_unstake_token(
        &self,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        let payment_amount = self.call_value().egld_value().clone_value();
        self.unstake_token().issue_and_set_all_roles(
            EsdtTokenType::Meta,
            payment_amount,
            token_display_name,
            token_ticker,
            num_decimals,
            None,
        );
    }

    #[only_owner]
    #[endpoint(setStateActive)]
    fn set_state_active(&self) {
        self.state().set(State::Active);
    }

    #[only_owner]
    #[endpoint(setStateInactive)]
    fn set_state_inactive(&self) {
        self.state().set(State::Inactive);
    }

    #[only_owner]
    #[endpoint(setAccumulatorContract)]
    fn set_accumulator_contract(&self, accumulator_contract: ManagedAddress) {
        self.accumulator_contract().set(accumulator_contract);
    }

    #[only_owner]
    #[endpoint(setFees)]
    fn set_fees(&self, fees: BigUint) {
        self.fees().set(fees);
    }

    #[only_owner]
    #[endpoint(setMinimumRounds)]
    fn set_minimum_rounds(&self, minimum_rounds: u64) {
        self.minimum_rounds().set(minimum_rounds);
    }

    #[only_owner]
    #[endpoint(setMaxDelegationAddresses)]
    fn set_max_delegation_addresses(&self, number: usize) {
        require!(number >= 1, ERROR_MAX_SELECTED_PROVIDERS);
        self.max_delegation_addresses().set(number);
    }

    #[only_owner]
    #[endpoint(setMaxSelectedProviders)]
    fn set_max_selected_providers(&self, number: BigUint) {
        require!(
            number >= BigUint::from(1u64),
            ERROR_MAX_CHANGED_DELEGATION_ADDRESSES
        );

        self.max_selected_providers().set(number);
    }

    #[only_owner]
    #[endpoint(setUnbondPeriod)]
    fn set_unbond_period(&self, period: u64) {
        self.unbond_period().set(period);
    }

    #[only_owner]
    #[endpoint(setManagers)]
    fn set_managers(&self, managers: MultiValueEncoded<ManagedAddress>) {
        self.managers().extend(managers);
    }

    #[only_owner]
    #[endpoint(removeManager)]
    fn remove_manager(&self, manager: ManagedAddress) {
        self.managers().swap_remove(&manager);
    }

    #[endpoint(cleanUnbondEpochs)]
    fn clean_unbond_epochs(&self, nonce: u64) {
        let epoch = self.blockchain().get_block_epoch();
        let map_token = self.unstake_token();

        let balance = map_token.get_balance(nonce);

        if balance == BigUint::zero() {
            return;
        }

        let attributes: UnstakeTokenAttributes = map_token.get_token_attributes(nonce);
        if attributes.unstake_epoch < epoch {
            self.unstake_token_nonce(attributes.unbond_epoch).clear();
            // The protocol always holds 1 unit of the MetaESDT token in the contract
            let balance = map_token.get_balance(nonce);
            map_token.nft_burn(nonce, &balance);
        }
    }

    #[inline]
    fn is_state_active(&self, state: State) {
        require!(State::Active == state, ERROR_NOT_ACTIVE);
    }

    #[view(fees)]
    #[storage_mapper("fees")]
    fn fees(&self) -> SingleValueMapper<BigUint>;

    #[view(getAccumulatorContract)]
    #[storage_mapper("accumulatorContract")]
    fn accumulator_contract(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(roundsPerEpoch)]
    #[storage_mapper("roundsPerEpoch")]
    fn rounds_per_epoch(&self) -> SingleValueMapper<u64>;

    #[view(minimumRounds)]
    #[storage_mapper("minimumRounds")]
    fn minimum_rounds(&self) -> SingleValueMapper<u64>;

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

    #[view(getRewardsReserve)]
    #[storage_mapper("rewardsReserve")]
    fn rewards_reserve(&self) -> SingleValueMapper<BigUint>;

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
