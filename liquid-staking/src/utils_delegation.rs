use crate::{
    StorageCache, ERROR_BAD_PAYMENT_AMOUNT, ERROR_INSUFFICIENT_PENDING_EGLD,
    ERROR_INSUFFICIENT_PENDING_XEGLD, MIN_EGLD_TO_DELEGATE,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait DelegateUtilsModule:
    crate::storage::StorageModule
    + crate::config::ConfigModule
    + crate::utils::UtilsModule
    + crate::events::EventsModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    fn determine_delegate_amounts(
        &self,
        storage_cache: &mut StorageCache<Self>,
        payment: &BigUint,
    ) -> (BigUint, BigUint) {
        let min_egld_amount = BigUint::from(MIN_EGLD_TO_DELEGATE);
        if self.can_fully_instant_stake(storage_cache, payment, &min_egld_amount) {
            // Case 1: Full instant staking
            (payment.clone(), BigUint::zero())
        } else if self.can_handle_pending_redemption(storage_cache, &min_egld_amount) {
            // Handle both Case 2 and Case 3
            self.handle_pending_redemption(storage_cache, payment, &min_egld_amount)
        } else {
            // Fallback: use all the payment amount for normal staking flow
            (BigUint::zero(), payment.clone())
        }
    }

    fn handle_pending_redemption(
        &self,
        storage_cache: &StorageCache<Self>,
        payment: &BigUint,
        min_egld_amount: &BigUint,
    ) -> (BigUint, BigUint) {
        let egld_from_pending = &storage_cache.pending_egld_for_unstake;
        let possible_instant_amount =
            self.calculate_instant_amount(payment, egld_from_pending, min_egld_amount);

        if self.can_fully_redeem(storage_cache, payment, min_egld_amount) {
            let egld_to_add_liquidity = payment - egld_from_pending;
            (egld_from_pending.clone(), egld_to_add_liquidity)
        } else if self.can_partially_redeem(
            storage_cache,
            &possible_instant_amount,
            min_egld_amount,
        ) {
            let egld_to_add_liquidity = payment - &possible_instant_amount;
            (possible_instant_amount, egld_to_add_liquidity)
        } else {
            // Fallback: use all the payment amount for normal staking flow
            (BigUint::zero(), payment.clone())
        }
    }

    fn can_fully_redeem(
        &self,
        storage_cache: &StorageCache<Self>,
        payment: &BigUint,
        min_egld_amount: &BigUint,
    ) -> bool {
        payment > &storage_cache.pending_egld_for_unstake
            && (payment - &storage_cache.pending_egld_for_unstake) >= *min_egld_amount
    }

    fn can_partially_redeem(
        &self,
        storage_cache: &StorageCache<Self>,
        possible_instant_amount: &BigUint,
        min_xegld_amount: &BigUint,
    ) -> bool {
        possible_instant_amount > &BigUint::zero()
            && &storage_cache.pending_egld_for_unstake >= possible_instant_amount
            && (&storage_cache.pending_egld_for_unstake - possible_instant_amount)
                >= *min_xegld_amount
    }

    fn can_fully_instant_stake(
        &self,
        storage_cache: &StorageCache<Self>,
        staked_egld_amount: &BigUint,
        min_egld_amount: &BigUint,
    ) -> bool {
        staked_egld_amount == &storage_cache.pending_egld_for_unstake
            || (&storage_cache.pending_egld_for_unstake >= staked_egld_amount
                && staked_egld_amount <= &(&storage_cache.pending_egld_for_unstake - min_egld_amount))
    }
    

    fn can_handle_pending_redemption(
        &self,
        storage_cache: &StorageCache<Self>,
        min_egld_amount: &BigUint,
    ) -> bool {
        &storage_cache.pending_egld_for_unstake >= min_egld_amount
    }

    fn process_redemption_and_staking(
        &self,
        storage_cache: &mut StorageCache<Self>,
        egld_from_pending_used: &BigUint,
        egld_to_add_liquidity: &BigUint,
    ) {
        let mut final_amount_to_mint = BigUint::zero();

        // Process redemption of pending xEGLD by the user via his EGLD
        if egld_from_pending_used > &BigUint::zero() {
            self.process_pending_redemption(
                storage_cache,
                egld_from_pending_used,
                &mut final_amount_to_mint,
            );
        }

        let caller = self.blockchain().get_caller();

        // Increase the pending EGLD by the amount left to be staked if any
        if egld_to_add_liquidity > &BigUint::zero() {
            self.process_egld_staking(
                storage_cache,
                egld_to_add_liquidity,
                &mut final_amount_to_mint,
            );
        }

        if final_amount_to_mint > BigUint::zero() {
            // Add the liquidity to the pool and mint the corresponding xEGLD
            let ls_amount = self.pool_add_liquidity(&final_amount_to_mint, storage_cache);
            let user_payment = self.mint_ls_token(ls_amount);

            // Emit the add liquidity event
            self.emit_add_liquidity_event(&storage_cache, egld_to_add_liquidity);
            // Send the final amount to the user, including the xEGLD from pending redemption if any and the fresh minted xEGLD if any
            self.tx().to(&caller).esdt(user_payment).transfer();
        }
    }

    fn process_pending_redemption(
        &self,
        storage_cache: &mut StorageCache<Self>,
        egld_from_pending_used: &BigUint,
        final_amount_to_mint: &mut BigUint,
    ) {
        // Subtract the xEGLD from the pending_ls_for_unstake
        // Should never fail, but just in case
        require!(
            storage_cache.pending_egld_for_unstake >= *egld_from_pending_used,
            ERROR_INSUFFICIENT_PENDING_XEGLD
        );

        storage_cache.pending_egld_for_unstake -= egld_from_pending_used;

        // Add the instant_unbound_balance to the total_withdrawn_egld
        storage_cache.total_withdrawn_egld += egld_from_pending_used;

        // Ensure the remaining pending xEGLD is higher or equal to min_xegld_amount or is zero
        require!(
            storage_cache.pending_egld_for_unstake >= BigUint::from(MIN_EGLD_TO_DELEGATE)
                || storage_cache.pending_egld_for_unstake == BigUint::zero(),
            ERROR_INSUFFICIENT_PENDING_XEGLD
        );

        // Add the redeemed xEGLD to the final amount to send
        *final_amount_to_mint += egld_from_pending_used;
    }

    fn process_egld_staking(
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
