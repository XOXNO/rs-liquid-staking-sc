use multiversx_sc::hex_literal::hex;

use crate::{
    accumulator,
    callback::{CallbackModule, CallbackProxy},
    delegation_manager_proxy, delegation_proxy,
    errors::ERROR_NO_DELEGATION_CONTRACTS,
    StorageCache, ERROR_INSUFFICIENT_PENDING_EGLD, ERROR_INSUFFICIENT_REWARDS,
    ERROR_NOT_WHITELISTED, MIN_EGLD_TO_DELEGATE, MIN_GAS_FOR_ASYNC_CALL,
    MIN_GAS_FOR_ASYNC_CALL_CLAIM_REWARDS, MIN_GAS_FOR_CALLBACK,
};

pub const DELEGATION_MANAGER: [u8; 32] =
    hex!("000000000000000000010000000000000000000000000000000000000004ffff");

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait ManageModule:
    crate::config::ConfigModule
    + crate::events::EventsModule
    + crate::callback::CallbackModule
    + crate::delegation::DelegationModule
    + crate::storage::StorageModule
    + crate::utils::UtilsModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::ongoing_operation::OngoingOperationModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[endpoint(delegatePending)]
    fn delegate_pending(&self) {
        let mut storage_cache = StorageCache::new(self);

        self.is_state_active(storage_cache.contract_state);

        self.is_manager(&self.blockchain().get_caller(), true);

        self.require_min_rounds_passed();

        require!(
            storage_cache.pending_egld >= BigUint::from(MIN_EGLD_TO_DELEGATE),
            ERROR_INSUFFICIENT_PENDING_EGLD
        );

        let delegation_contract =
            self.get_delegation_contract_for_delegate(&storage_cache.pending_egld);

        // Important before delegating the amount to the new contracts, set the reserve to 0
        storage_cache.pending_egld = BigUint::zero();

        for data in &delegation_contract {
            self.tx()
                .to(&data.delegation_address)
                .typed(delegation_proxy::DelegationMockProxy)
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

    #[endpoint(unDelegatePending)]
    fn un_delegate_pending(&self) {
        let mut storage_cache = StorageCache::new(self);

        self.is_state_active(storage_cache.contract_state);

        self.is_manager(&self.blockchain().get_caller(), true);

        self.require_min_rounds_passed();

        require!(
            &storage_cache.pending_egld_for_unstake >= &BigUint::from(MIN_EGLD_TO_DELEGATE),
            ERROR_INSUFFICIENT_PENDING_EGLD
        );

        let delegation_contract =
            self.get_delegation_contract_for_undelegate(&storage_cache.pending_egld_for_unstake);

        // Important before un delegating the amount from the new contracts, set the amount to 0
        storage_cache.pending_egld_for_unstake = BigUint::zero();

        for data in &delegation_contract {
            self.tx()
                .to(&data.delegation_address)
                .typed(delegation_proxy::DelegationMockProxy)
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
            .typed(delegation_proxy::DelegationMockProxy)
            .withdraw()
            .gas(MIN_GAS_FOR_ASYNC_CALL)
            .callback(CallbackModule::callbacks(self).withdraw_tokens_callback(&contract))
            .gas_for_callback(MIN_GAS_FOR_CALLBACK)
            .register_promise();
    }

    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        let storage_cache = StorageCache::new(self);

        self.is_manager(&self.blockchain().get_caller(), true);

        self.is_state_active(storage_cache.contract_state);

        let delegation_addresses_mapper = self.delegation_addresses_list();

        require!(
            !delegation_addresses_mapper.is_empty(),
            ERROR_NO_DELEGATION_CONTRACTS
        );

        let mut addresses = MultiValueEncoded::new();

        for node in delegation_addresses_mapper.iter() {
            addresses.push(node.into_value());
        }

        let required_gas = MIN_GAS_FOR_ASYNC_CALL_CLAIM_REWARDS * addresses.len() as u64;

        self.tx()
            .to(&ManagedAddress::new_from_bytes(&DELEGATION_MANAGER))
            .typed(delegation_manager_proxy::DelegationManagerMockProxy)
            .claim_multiple(addresses)
            .gas(required_gas)
            .callback(CallbackModule::callbacks(self).claim_rewards_callback())
            .gas_for_callback(MIN_GAS_FOR_CALLBACK)
            .register_promise();
    }

    #[endpoint(delegateRewards)]
    fn delegate_rewards(&self) {
        let mut storage_cache = StorageCache::new(self);

        self.is_manager(&self.blockchain().get_caller(), true);

        self.is_state_active(storage_cache.contract_state);

        let min_egld = BigUint::from(MIN_EGLD_TO_DELEGATE);
        require!(
            storage_cache.rewards_reserve >= min_egld,
            ERROR_INSUFFICIENT_REWARDS
        );

        let fees = self.calculate_split(&storage_cache.rewards_reserve, &self.fees().get());

        let rewards_after = &storage_cache.rewards_reserve - &fees;

        if rewards_after >= min_egld {
            storage_cache.rewards_reserve = rewards_after;

            self.tx()
                .to(&self.accumulator_contract().get())
                .typed(accumulator::AccumulatorProxy)
                .deposit()
                .egld(&fees)
                .transfer_execute();

            self.protocol_revenue_event(&fees, self.blockchain().get_block_epoch());
        }

        let delegation_contract =
            self.get_delegation_contract_for_delegate(&storage_cache.rewards_reserve);
        // Important before delegating the rewards to the new contracts, set the rewards reserve to 0
        storage_cache.rewards_reserve = BigUint::zero();

        for data in &delegation_contract {
            self.tx()
                .to(&data.delegation_address)
                .typed(delegation_proxy::DelegationMockProxy)
                .delegate()
                .egld(&data.amount)
                .gas(MIN_GAS_FOR_ASYNC_CALL)
                .callback(
                    CallbackModule::callbacks(self)
                        .delegate_rewards_callback(&data.delegation_address, &data.amount),
                )
                .gas_for_callback(MIN_GAS_FOR_CALLBACK)
                .register_promise();
        }

        self.emit_general_liquidity_event(&storage_cache);
    }
}
