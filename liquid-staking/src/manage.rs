multiversx_sc::imports!();
use crate::{
    callback::{CallbackModule, CallbackProxy},
    errors::ERROR_NO_DELEGATION_CONTRACTS,
    proxy::{proxy_accumulator, proxy_delegation, proxy_delegation_manager},
    StorageCache, DELEGATION_MANAGER, ERROR_INSUFFICIENT_FEES_RESERVE,
    ERROR_INSUFFICIENT_PENDING_EGLD, ERROR_NOT_WHITELISTED, MIN_EGLD_TO_DELEGATE,
    MIN_GAS_FOR_ASYNC_CALL, MIN_GAS_FOR_ASYNC_CALL_CLAIM_REWARDS, MIN_GAS_FOR_CALLBACK,
};

#[multiversx_sc::module]
pub trait ManageModule:
    crate::config::ConfigModule
    + crate::events::EventsModule
    + crate::callback::CallbackModule
    + crate::delegation::DelegationModule
    + crate::storage::StorageModule
    + crate::score::ScoreModule
    + crate::selection::SelectionModule
    + crate::utils::generic::UtilsModule
    + crate::utils::delegate::DelegateUtilsModule
    + crate::utils::un_delegation::UnDelegateUtilsModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    /// Delegates pending EGLD from the liquid staking contract to a list of providers,
    /// ensuring fair distribution by allocating set amounts to multiple providers in batches.
    ///
    /// Arguments:
    /// - `amount`: Optional. The specific amount to delegate; if not provided, uses the
    ///             entire pending EGLD accumulated in the contract.
    #[endpoint(delegatePending)]
    fn delegate_pending(&self, amount: OptionalValue<BigUint>) {
        let mut storage_cache = StorageCache::new(self);

        self.is_state_active(storage_cache.contract_state);

        self.is_manager(&self.blockchain().get_caller(), true);

        self.require_min_rounds_passed();

        require!(
            storage_cache.pending_egld >= BigUint::from(MIN_EGLD_TO_DELEGATE),
            ERROR_INSUFFICIENT_PENDING_EGLD
        );

        let amount_to_delegate = match amount {
            OptionalValue::Some(amount) => {
                require!(
                    &amount <= &storage_cache.pending_egld,
                    ERROR_INSUFFICIENT_PENDING_EGLD
                );

                require!(
                    amount >= BigUint::from(MIN_EGLD_TO_DELEGATE),
                    ERROR_INSUFFICIENT_PENDING_EGLD
                );

                let left_over = &storage_cache.pending_egld - &amount;

                require!(
                    left_over >= BigUint::from(MIN_EGLD_TO_DELEGATE)
                        || left_over == BigUint::zero(),
                    ERROR_INSUFFICIENT_PENDING_EGLD
                );

                amount
            }
            OptionalValue::None => storage_cache.pending_egld.clone(),
        };

        let contracts = self.get_contracts_for_delegate(&amount_to_delegate, &mut storage_cache);

        // Important before delegating the amount to the new contracts, set the pending egld to 0 or deduct the amount delegated when not full
        storage_cache.pending_egld -= amount_to_delegate;

        for data in &contracts {
            self.move_delegation_contract_to_back(&data.delegation_address);
            // Important before delegating the amount to the new contracts, update the pending staking callback amount
            // Reverse the amount when the callback fails or succeeds
            // Required to avoid concurrency issues when the same contract is delegated to multiple times in different transactions simultaneously, might reach the cap and throw an error if not updated
            self.delegation_contract_data(&data.delegation_address)
                .update(|contract_data| {
                    contract_data.pending_staking_callback_amount += &data.amount;
                });
            self.tx()
                .to(&data.delegation_address)
                .typed(proxy_delegation::DelegationMockProxy)
                .delegate()
                .egld(&data.amount)
                .gas(MIN_GAS_FOR_ASYNC_CALL)
                .callback(
                    CallbackModule::callbacks(self)
                        .add_liquidity_callback(&data.delegation_address, &data.amount),
                )
                .gas_for_callback(MIN_GAS_FOR_CALLBACK)
                .register_promise();
        }
        self.emit_general_liquidity_event(&storage_cache);
    }

    /// Un-delegates pending EGLD from multiple providers, reducing impact on individual
    /// providers by un-delegating in small batches. This supports a balanced distribution
    /// without penalizing any provider heavily.
    ///
    /// Arguments:
    /// - `amount`: Optional. Specific amount to un-delegate; if omitted, the function
    ///             un-delegates from pending EGLD accumulated in the contract.
    #[endpoint(unDelegatePending)]
    fn un_delegate_pending(&self, amount: OptionalValue<BigUint>) {
        let mut storage_cache = StorageCache::new(self);

        self.is_state_active(storage_cache.contract_state);

        self.is_manager(&self.blockchain().get_caller(), true);

        self.require_min_rounds_passed();

        require!(
            &storage_cache.pending_egld_for_unstake >= &BigUint::from(MIN_EGLD_TO_DELEGATE),
            ERROR_INSUFFICIENT_PENDING_EGLD
        );

        let amount_to_unstake = match amount {
            OptionalValue::Some(amount) => {
                require!(
                    &amount <= &storage_cache.pending_egld_for_unstake,
                    ERROR_INSUFFICIENT_PENDING_EGLD
                );

                require!(
                    amount >= BigUint::from(MIN_EGLD_TO_DELEGATE),
                    ERROR_INSUFFICIENT_PENDING_EGLD
                );

                let left_over = &storage_cache.pending_egld_for_unstake - &amount;

                require!(
                    left_over >= BigUint::from(MIN_EGLD_TO_DELEGATE)
                        || left_over == BigUint::zero(),
                    ERROR_INSUFFICIENT_PENDING_EGLD
                );

                amount
            }
            OptionalValue::None => storage_cache.pending_egld_for_unstake.clone(),
        };

        let contracts = self.get_contracts_for_undelegate(&amount_to_unstake, &mut storage_cache);

        // Important before un delegating the amount from the new contracts, set the amount to 0
        storage_cache.pending_egld_for_unstake -= amount_to_unstake;

        for data in &contracts {
            self.move_un_delegation_contract_to_back(&data.delegation_address);
            // Important before un delegating the amount from the new contracts, update the pending unstaking callback amount
            // Reverse the amount when the callback fails or succeeds
            // Required to avoid concurrency issues when the same contract is un delegated from multiple times in different transactions simultaneously, might try to over unstake something that was already unstaked
            self.delegation_contract_data(&data.delegation_address)
                .update(|contract_data| {
                    contract_data.pending_unstaking_callback_amount += &data.amount;
                });
            self.tx()
                .to(&data.delegation_address)
                .typed(proxy_delegation::DelegationMockProxy)
                .undelegate(&data.amount)
                .gas(MIN_GAS_FOR_ASYNC_CALL)
                .callback(
                    CallbackModule::callbacks(self)
                        .remove_liquidity_callback(&data.delegation_address, &data.amount),
                )
                .gas_for_callback(MIN_GAS_FOR_CALLBACK)
                .register_promise();
        }

        self.emit_general_liquidity_event(&storage_cache);
    }

    /// Withdraws pending funds from a specified delegation contract, an essential function
    /// to maintain liquidity for instant withdrawals when users leave the staking pool.
    ///
    /// Arguments:
    /// - `contract`: Address of the delegation contract from which pending funds will
    ///               be withdrawn.
    #[endpoint(withdrawPending)]
    fn withdraw_pending(&self, contract: ManagedAddress) {
        let storage_cache = StorageCache::new(self);

        self.is_manager(&self.blockchain().get_caller(), true);

        self.is_state_active(storage_cache.contract_state);

        require!(
            !self.delegation_contract_data(&contract).is_empty(),
            ERROR_NOT_WHITELISTED
        );

        self.tx()
            .to(&contract)
            .typed(proxy_delegation::DelegationMockProxy)
            .withdraw()
            .gas(MIN_GAS_FOR_ASYNC_CALL)
            .callback(CallbackModule::callbacks(self).withdraw_tokens_callback(&contract))
            .gas_for_callback(MIN_GAS_FOR_CALLBACK)
            .register_promise();
    }

    /// Claims accumulated staking rewards from the providers, optimizing the process
    /// by delegating these rewards directly back into the contract to generate compounding
    /// returns for xEGLD holders. This endpoint prevents repeated withdrawals and staking,
    /// improving gas efficiency and yield.
    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        let storage_cache = StorageCache::new(self);

        self.is_manager(&self.blockchain().get_caller(), true);

        self.is_state_active(storage_cache.contract_state);

        let list_mapper = self.delegation_addresses_list();

        require!(!list_mapper.is_empty(), ERROR_NO_DELEGATION_CONTRACTS);

        let mut addresses = MultiValueEncoded::new();

        for provider in list_mapper.iter() {
            addresses.push(provider);
        }

        let gas = MIN_GAS_FOR_ASYNC_CALL_CLAIM_REWARDS * addresses.len() as u64;

        self.tx()
            .to(&ManagedAddress::new_from_bytes(&DELEGATION_MANAGER))
            .typed(proxy_delegation_manager::DelegationManagerMockProxy)
            .claim_multiple(addresses)
            .gas(gas)
            .callback(CallbackModule::callbacks(self).claim_rewards_callback())
            .gas_for_callback(MIN_GAS_FOR_CALLBACK)
            .register_promise();
    }

    #[endpoint(claimFees)]
    fn claim_fees(&self) {
        let mut storage_cache = StorageCache::new(self);

        require!(
            storage_cache.fees_reserve > BigUint::zero(),
            ERROR_INSUFFICIENT_FEES_RESERVE
        );

        self.tx()
            .to(&self.accumulator_contract().get())
            .typed(proxy_accumulator::AccumulatorProxy)
            .deposit()
            .egld(&storage_cache.fees_reserve)
            .sync_call();

        storage_cache.fees_reserve = BigUint::zero();
    }
}
