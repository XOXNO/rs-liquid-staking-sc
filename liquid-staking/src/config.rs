multiversx_sc::imports!();
use crate::{
    structs::{ScoringConfig, State},
    ERROR_MAX_CHANGED_DELEGATION_ADDRESSES, ERROR_MAX_SELECTED_PROVIDERS, ERROR_NOT_MANAGER,
    ERROR_WEIGHTS_MUST_SUM_TO_100,
};

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
        let payment_amount = self.call_value().egld().clone_value();
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
        let payment_amount = self.call_value().egld().clone_value();
        self.unstake_token().issue_and_set_all_roles(
            EsdtTokenType::MetaFungible,
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
    #[endpoint(setMaxAddresses)]
    fn set_max_addresses(&self, number: usize) {
        require!(number >= 1, ERROR_MAX_SELECTED_PROVIDERS);
        self.max_delegation_addresses().set(number);
    }

    #[only_owner]
    #[endpoint(setMaxSelectedProviders)]
    fn set_max_selected_providers(&self, number: BigUint) {
        require!(
            number >= 1u64,
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
    #[endpoint(addManagers)]
    fn set_managers(&self, managers: MultiValueEncoded<ManagedAddress>) {
        self.managers().extend(managers);
    }

    #[only_owner]
    #[endpoint(removeManager)]
    fn remove_manager(&self, manager: ManagedAddress) {
        self.managers().swap_remove(&manager);
    }

    #[only_owner]
    #[endpoint(addLiquidityProvider)]
    fn add_liquidity_provider(&self, liquidity_provider: ManagedAddress) {
        self.liquidity_providers().insert(liquidity_provider);
    }

    #[only_owner]
    #[endpoint(removeLiquidityProviders)]
    fn remove_liquidity_provider(&self, liquidity_provider: ManagedAddress) {
        self.liquidity_providers().swap_remove(&liquidity_provider);
    }

    #[endpoint(setScoringConfig)]
    fn set_scoring_config(&self, config: ScoringConfig) {
        self.is_manager(&self.blockchain().get_caller(), true);
        require!(
            config.stake_weight + config.apy_weight + config.nodes_weight == 100,
            ERROR_WEIGHTS_MUST_SUM_TO_100
        );
        self.scoring_config().set(config);
    }

    fn is_manager(&self, address: &ManagedAddress, required: bool) -> bool {
        let owner = self.blockchain().get_owner_address();
        let is_manager = self.managers().contains(address) || address == &owner;
        if required && !is_manager {
            sc_panic!(ERROR_NOT_MANAGER);
        }
        is_manager
    }
}
