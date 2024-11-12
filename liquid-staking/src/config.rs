use crate::{
    structs::State, ERROR_MAX_CHANGED_DELEGATION_ADDRESSES, ERROR_MAX_SELECTED_PROVIDERS,
    ERROR_NOT_ACTIVE,
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

    #[inline]
    fn is_state_active(&self, state: State) {
        require!(State::Active == state, ERROR_NOT_ACTIVE);
    }
}
