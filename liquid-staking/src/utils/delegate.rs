multiversx_sc::imports!();
use crate::{
    StorageCache, ERROR_BAD_PAYMENT_AMOUNT, ERROR_INSUFFICIENT_PENDING_EGLD,
    ERROR_INSUFFICIENT_UNSTAKE_PENDING_EGLD, MIN_EGLD_TO_DELEGATE,
};

#[multiversx_sc::module]
pub trait DelegateUtilsModule:
    crate::storage::StorageModule
    + crate::config::ConfigModule
    + crate::utils::generic::UtilsModule
    + crate::events::EventsModule
    + crate::score::ScoreModule
    + crate::selection::SelectionModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    fn process_delegation(
        &self,
        storage_cache: &mut StorageCache<Self>,
        egld_from_pending_used: &BigUint,
        egld_to_add_liquidity: &BigUint,
        caller: &ManagedAddress,
    ) {
        let mut final_amount_to_mint = BigUint::zero();

        // Process redemption of pending xEGLD by the user via his EGLD
        if egld_from_pending_used > &BigUint::zero() {
            self.decrease_pending_egld(
                storage_cache,
                egld_from_pending_used,
                &mut final_amount_to_mint,
            );
        }

        // Increase the pending EGLD by the amount left to be staked if any
        if egld_to_add_liquidity > &BigUint::zero() {
            self.increase_pending_egld(
                storage_cache,
                egld_to_add_liquidity,
                &mut final_amount_to_mint,
            );
        }

        if final_amount_to_mint > BigUint::zero() {
            // Add the liquidity to the pool and mint the corresponding xEGLD
            let ls_amount = self.pool_add_liquidity(&final_amount_to_mint, storage_cache);
            let user_payment = self.mint_ls_token(ls_amount);

            // Send the final amount to the user, including the xEGLD from pending redemption if any and the fresh minted xEGLD if any
            self.tx().to(caller).esdt(user_payment).transfer();
            // Emit the add liquidity event
            self.emit_add_liquidity_event(
                &storage_cache,
                &(egld_to_add_liquidity + egld_from_pending_used),
                Some(caller.clone()),
            );
        }
    }

    fn decrease_pending_egld(
        &self,
        storage_cache: &mut StorageCache<Self>,
        egld_from_pending_used: &BigUint,
        final_amount_to_mint: &mut BigUint,
    ) {
        storage_cache.pending_egld_for_unstake -= egld_from_pending_used;

        // Add the instant_unbound_balance to the total_withdrawn_egld
        storage_cache.total_withdrawn_egld += egld_from_pending_used;

        // Ensure the remaining pending xEGLD is higher or equal to min_xegld_amount or is zero
        require!(
            storage_cache.pending_egld_for_unstake >= BigUint::from(MIN_EGLD_TO_DELEGATE)
                || storage_cache.pending_egld_for_unstake == BigUint::zero(),
            ERROR_INSUFFICIENT_UNSTAKE_PENDING_EGLD
        );

        // Add the redeemed xEGLD to the final amount to send
        *final_amount_to_mint += egld_from_pending_used;
    }

    fn increase_pending_egld(
        &self,
        storage_cache: &mut StorageCache<Self>,
        egld_to_add_liquidity: &BigUint,
        final_amount_to_mint: &mut BigUint,
    ) {
        storage_cache.pending_egld += egld_to_add_liquidity;

        // Ensure the remaining pending EGLD is not less than 1 EGLD
        require!(
            storage_cache.pending_egld >= BigUint::from(MIN_EGLD_TO_DELEGATE),
            ERROR_INSUFFICIENT_PENDING_EGLD
        );

        // Add the minted xEGLD to the final amount to send
        *final_amount_to_mint += egld_to_add_liquidity;
    }

    fn validate_delegate_conditions(
        &self,
        storage_cache: &mut StorageCache<Self>,
        amount: &BigUint,
    ) {
        self.is_state_active(storage_cache.contract_state);

        require!(amount > &BigUint::zero(), ERROR_BAD_PAYMENT_AMOUNT);
    }
}
