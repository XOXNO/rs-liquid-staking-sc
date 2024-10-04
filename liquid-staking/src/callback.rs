use crate::{
    structs::{ClaimStatus, ClaimStatusType},
    StorageCache,
};

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

                self.emit_remove_liquidity_event(
                    &storage_cache,
                    &BigUint::zero(),
                    &BigUint::zero(),
                );
            }
            ManagedAsyncCallResult::Err(_) => {
                let ls_amount = self.get_ls_amount(&egld_to_unstake, &mut storage_cache);
                storage_cache.pending_ls_for_unstake += ls_amount;
                self.move_delegation_contract_to_back(delegation_contract);
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
            }
        }
        self.move_delegation_contract_to_back(delegation_contract);
    }

    #[promises_callback]
    fn withdraw_tokens_callback(
        &self,
        delegation_contract: &ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        let mut storage_cache = StorageCache::new(self);
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let withdraw_amount = self.call_value().egld_value().clone_value();
                let delegation_contract_mapper =
                    self.delegation_contract_data(&delegation_contract);

                if withdraw_amount > 0u64 {
                    storage_cache.total_withdrawn_egld += &withdraw_amount;
                    delegation_contract_mapper.update(|contract_data| {
                        contract_data.total_unstaked_from_ls_contract -= &withdraw_amount;
                    });
                }
            }
            ManagedAsyncCallResult::Err(_) => {}
        }
    }

    #[promises_callback]
    fn claim_rewards_callback(
        &self,
        delegation_contract: &ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        let mut storage_cache = StorageCache::new(self);
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                let rewards = self.call_value().egld_value().clone_value();

                if rewards > 0u64 {
                    storage_cache.rewards_reserve += &rewards;
                    self.emit_claim_rewards_event(delegation_contract, &rewards);
                }
            }
            ManagedAsyncCallResult::Err(_) => {}
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

                let current_claim_status = self.load_operation::<ClaimStatus>();

                if current_claim_status.status == ClaimStatusType::Finished {
                    self.delegation_claim_status()
                        .update(|claim_status| claim_status.status = ClaimStatusType::Redelegated);
                }

                storage_cache.virtual_egld_reserve += staked_tokens;

                let sc_address = self.blockchain().get_sc_address();
                self.emit_add_liquidity_event(&storage_cache, &sc_address, BigUint::zero());
            }
            ManagedAsyncCallResult::Err(_) => {
                // Revert the deduction made in the parent function
                storage_cache.rewards_reserve += staked_tokens;
                self.move_delegation_contract_to_back(&delegation_contract);
            }
        }
    }
}
