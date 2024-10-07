use multiversx_sc_modules::ongoing_operation::{
    CONTINUE_OP, DEFAULT_MIN_GAS_TO_SAVE_PROGRESS, STOP_OP,
};

use crate::{
    accumulator,
    callback::{CallbackModule, CallbackProxy},
    delegation_proxy,
    errors::ERROR_NO_DELEGATION_CONTRACTS,
    structs::{ClaimStatus, ClaimStatusType},
    StorageCache, ERROR_INSUFFICIENT_PENDING_EGLD, ERROR_INSUFFICIENT_REWARDS,
    ERROR_NOT_WHITELISTED, ERROR_RECOMPUTE_RESERVES, MIN_EGLD_TO_DELEGATE, MIN_GAS_FOR_ASYNC_CALL,
    MIN_GAS_FOR_CALLBACK,
};

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

        self.require_min_rounds_passed();

        require!(
            storage_cache.pending_egld >= BigUint::from(MIN_EGLD_TO_DELEGATE),
            ERROR_INSUFFICIENT_PENDING_EGLD
        );

        let delegation_contract =
            self.get_delegation_contract_for_delegate(&storage_cache.pending_egld);

        for data in &delegation_contract {
            // !!!! Required to prevent double delegation from the same amount, while the callback is not executed !!!!
            storage_cache.pending_egld -= &data.amount;
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
    }

    #[endpoint(unDelegatePending)]
    fn un_delegate_pending(&self) {
        let mut storage_cache = StorageCache::new(self);

        self.is_state_active(storage_cache.contract_state);

        self.require_min_rounds_passed();

        let pending = storage_cache.pending_ls_for_unstake.clone();
        let egld_to_unstake = self.pool_remove_liquidity(&pending, &mut storage_cache);

        require!(
            egld_to_unstake >= BigUint::from(MIN_EGLD_TO_DELEGATE),
            ERROR_INSUFFICIENT_PENDING_EGLD
        );

        self.burn_ls_token(&pending);

        storage_cache.pending_ls_for_unstake = BigUint::zero();

        self.emit_remove_liquidity_event(&storage_cache, &pending);

        let delegation_contract = self.get_delegation_contract_for_undelegate(&egld_to_unstake);

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
    }

    #[endpoint(withdrawPending)]
    fn withdraw_pending(&self, contract: ManagedAddress) {
        let storage_cache = StorageCache::new(self);

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

        self.is_state_active(storage_cache.contract_state);

        let delegation_addresses_mapper = self.delegation_addresses_list();

        require!(
            !delegation_addresses_mapper.is_empty(),
            ERROR_NO_DELEGATION_CONTRACTS
        );

        let claim_status_mapper = self.delegation_claim_status();

        let old_claim_status = claim_status_mapper.get();
        let current_epoch = self.blockchain().get_block_epoch();

        let mut current_claim_status = self.load_operation::<ClaimStatus>();

        self.check_claim_operation(&current_claim_status, old_claim_status, current_epoch);
        self.prepare_claim_operation(&mut current_claim_status, current_epoch);

        let run_result = self.run_while_it_has_gas(DEFAULT_MIN_GAS_TO_SAVE_PROGRESS, || {
            let delegation_address_node = delegation_addresses_mapper
                .get_node_by_id(current_claim_status.current_node)
                .unwrap();

            let next_node = delegation_address_node.get_next_node_id();
            let delegation_address = delegation_address_node.into_value();

            self.tx()
                .to(&delegation_address)
                .typed(delegation_proxy::DelegationMockProxy)
                .claim_rewards()
                .gas(MIN_GAS_FOR_ASYNC_CALL)
                .callback(
                    CallbackModule::callbacks(self).claim_rewards_callback(&delegation_address),
                )
                .gas_for_callback(MIN_GAS_FOR_CALLBACK)
                .register_promise();

            if next_node == 0 {
                sc_print!("No next node {}", next_node);
                claim_status_mapper.set(current_claim_status.clone());
                return STOP_OP;
            } else {
                current_claim_status.current_node = next_node;
            }

            CONTINUE_OP
        });

        match run_result {
            OperationCompletionStatus::InterruptedBeforeOutOfGas => {
                self.save_progress(&current_claim_status);
            }
            OperationCompletionStatus::Completed => {
                sc_print!("Claim status updated {}", 0);
                claim_status_mapper.update(|claim_status| {
                    claim_status.status = ClaimStatusType::Finished;
                });
            }
        };
    }

    #[endpoint(delegateRewards)]
    fn delegate_rewards(&self) {
        let mut storage_cache = StorageCache::new(self);
        let claim_status = self.delegation_claim_status().get();

        self.is_state_active(storage_cache.contract_state);

        require!(
            claim_status.status == ClaimStatusType::Finished,
            ERROR_RECOMPUTE_RESERVES
        );

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

        for data in &delegation_contract {
            storage_cache.rewards_reserve -= &data.amount;
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
        if storage_cache.rewards_reserve == BigUint::zero() {
            self.delegation_claim_status()
                .update(|claim_status| claim_status.status = ClaimStatusType::Redelegated);
        }
    }
}
