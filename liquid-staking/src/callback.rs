multiversx_sc::imports!();
use crate::{utils, StorageCache};

#[multiversx_sc::module]
pub trait CallbackModule:
    crate::config::ConfigModule
    + crate::events::EventsModule
    + crate::storage::StorageModule
    + crate::score::ScoreModule
    + utils::generic::UtilsModule
    + crate::selection::SelectionModule
    + crate::liquidity_pool::LiquidityPoolModule
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
                storage_cache.pending_egld_for_unbond += egld_to_unstake;
                self.delegation_contract_data(&delegation_contract)
                    .update(|contract_data| {
                        contract_data.total_staked_from_ls_contract -= egld_to_unstake;
                        contract_data.total_unstaked_from_ls_contract += egld_to_unstake;
                        contract_data.pending_unstaking_callback_amount -= egld_to_unstake;
                    });
            }
            ManagedAsyncCallResult::Err(_) => {
                storage_cache.pending_egld_for_unstake += egld_to_unstake;
                self.delegation_contract_data(&delegation_contract)
                    .update(|contract_data| {
                        contract_data.pending_unstaking_callback_amount -= egld_to_unstake;
                    });
            }
        }
        self.emit_general_liquidity_event(&storage_cache);
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
                self.delegation_contract_data(&delegation_contract)
                    .update(|contract_data| {
                        contract_data.total_staked_from_ls_contract += staked_tokens;
                        contract_data.pending_staking_callback_amount -= staked_tokens;
                    });
            }
            ManagedAsyncCallResult::Err(_) => {
                storage_cache.pending_egld += staked_tokens;
                self.delegation_contract_data(&delegation_contract)
                    .update(|contract_data| {
                        contract_data.eligible = false;
                        contract_data.pending_staking_callback_amount -= staked_tokens;
                    });
                self.emit_general_liquidity_event(&storage_cache);
            }
        }
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
    fn claim_rewards_callback(&self, #[call_result] result: ManagedAsyncCallResult<BigUint>) {
        match result {
            ManagedAsyncCallResult::Ok(total_rewards) => {
                if total_rewards > BigUint::zero() {
                    let mut storage_cache = StorageCache::new(self);
                    let fees = self.calculate_share(&total_rewards, &self.fees().get());

                    let post_fees_amount = &total_rewards - &fees;

                    storage_cache.fees_reserve += &fees;
                    storage_cache.pending_egld += &post_fees_amount;
                    storage_cache.virtual_egld_reserve += &post_fees_amount;

                    self.emit_claim_rewards_event(&storage_cache, &total_rewards);
                    self.protocol_revenue_event(&fees, self.blockchain().get_block_epoch());
                }
            }
            _ => {}
        }
    }

    #[promises_callback]
    fn whitelist_delegation_contract_callback(
        &self,
        contract_address: ManagedAddress,
        staked_tokens: &BigUint,
        caller: &ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        let mut storage_cache = StorageCache::new(self);
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.delegation_contract_data(&contract_address)
                    .update(|contract_data| {
                        contract_data.total_staked_from_ls_contract += staked_tokens;
                    });

                self.add_delegation_address_in_list(contract_address.clone());
                self.add_un_delegation_address_in_list(contract_address);

                let ls_amount = self.pool_add_liquidity(&staked_tokens, &mut storage_cache);
                let user_payment = self.mint_ls_token(ls_amount);

                self.emit_add_liquidity_event(&storage_cache, staked_tokens, Some(caller.clone()));
                self.tx().to(caller).esdt(user_payment).transfer();
            }
            ManagedAsyncCallResult::Err(_) => {
                self.delegation_contract_data(&contract_address).clear();
                self.tx().to(caller).egld(staked_tokens).transfer();
            }
        }
    }
}
