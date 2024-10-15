use crate::{structs::ClaimStatusType, StorageCache};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait CallbackModule:
    crate::config::ConfigModule
    + crate::events::EventsModule
    + crate::delegation::DelegationModule
    + crate::storage::StorageModule
    + crate::utils::UtilsModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::ongoing_operation::OngoingOperationModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[promises_callback]
    fn remove_liquidity_callback(
        &self,
        delegation_contract: &ManagedAddress,
        egld_to_unstake: &BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        let mut storage_cache = StorageCache::new(self);
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.delegation_contract_data(&delegation_contract)
                    .update(|contract_data| {
                        contract_data.total_staked_from_ls_contract -= egld_to_unstake;
                        contract_data.total_unstaked_from_ls_contract += egld_to_unstake;
                    });
                storage_cache.pending_egld_for_unbond += egld_to_unstake;
                self.emit_general_liquidity_event(&storage_cache);
            }
            ManagedAsyncCallResult::Err(_) => {
                storage_cache.pending_egld_for_unstake += egld_to_unstake;
                self.emit_general_liquidity_event(&storage_cache);
            }
        }
    }

    #[promises_callback]
    fn add_liquidity_callback(
        &self,
        delegation_contract: &ManagedAddress,
        staked_tokens: &BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        let mut storage_cache = StorageCache::new(self);
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.delegation_contract_data(delegation_contract)
                    .update(|contract_data| {
                        contract_data.total_staked_from_ls_contract += staked_tokens;
                    });
            }
            ManagedAsyncCallResult::Err(_) => {
                storage_cache.pending_egld += staked_tokens;
                self.emit_general_liquidity_event(&storage_cache);
                self.delegation_contract_data(&delegation_contract)
                    .update(|contract_data| {
                        contract_data.eligible = false;
                    });
            }
        }
        self.move_delegation_contract_to_back(delegation_contract);
    }

    #[promises_callback]
    fn withdraw_tokens_callback(&self, delegation_contract: &ManagedAddress) {
        let withdraw_amount = self.call_value().egld_value().clone_value();
        if withdraw_amount > BigUint::zero() {
            let mut storage_cache = StorageCache::new(self);
            let delegation_contract_mapper = self.delegation_contract_data(&delegation_contract);
            
            storage_cache.total_withdrawn_egld += &withdraw_amount;
            storage_cache.pending_egld_for_unbond -= &withdraw_amount;

            delegation_contract_mapper.update(|contract_data| {
                contract_data.total_unstaked_from_ls_contract -= &withdraw_amount;
            });
            self.emit_withdraw_pending_event(&storage_cache, &withdraw_amount, delegation_contract);
        }
    }

    #[promises_callback]
    fn claim_rewards_callback(&self, delegation_contract: &ManagedAddress) {
        let rewards = self.call_value().egld_value().clone_value();

        if rewards > BigUint::zero() {
            let mut storage_cache = StorageCache::new(self);

            storage_cache.rewards_reserve += &rewards;
            self.emit_claim_rewards_event(&storage_cache, &rewards, delegation_contract);
        }
    }

    #[promises_callback]
    fn delegate_rewards_callback(
        &self,
        delegation_contract: &ManagedAddress,
        staked_tokens: &BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        let mut storage_cache = StorageCache::new(self);
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.delegation_contract_data(&delegation_contract)
                    .update(|contract_data| {
                        contract_data.total_staked_from_ls_contract += staked_tokens;
                    });

                storage_cache.virtual_egld_reserve += staked_tokens;
                self.emit_delegate_rewards_event(
                    &storage_cache,
                    staked_tokens,
                    delegation_contract,
                );
            }
            ManagedAsyncCallResult::Err(_) => {
                // Revert the deduction made in the parent function
                storage_cache.rewards_reserve += staked_tokens;

                self.delegation_claim_status()
                    .update(|claim_status| claim_status.status = ClaimStatusType::Finished);

                self.move_delegation_contract_to_back(&delegation_contract);
                self.delegation_contract_data(&delegation_contract)
                    .update(|contract_data| {
                        contract_data.eligible = false;
                    });

                self.emit_general_liquidity_event(&storage_cache);
            }
        }
    }
}
