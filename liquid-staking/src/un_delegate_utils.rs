use crate::{
    StorageCache, ERROR_BAD_PAYMENT_AMOUNT, ERROR_BAD_PAYMENT_TOKEN,
    ERROR_INSUFFICIENT_PENDING_EGLD, ERROR_INSUFFICIENT_PENDING_XEGLD, ERROR_LS_TOKEN_NOT_ISSUED,
    MIN_EGLD_TO_DELEGATE,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait UnDelegateUtilsModule:
    crate::storage::StorageModule
    + crate::config::ConfigModule
    + crate::utils::UtilsModule
    + crate::events::EventsModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    fn determine_undelegate_amounts(
        &self,
        storage_cache: &mut StorageCache<Self>,
        total_egld: &BigUint,
        min_egld_amount: &BigUint,
    ) -> (BigUint, BigUint) {
        if self.can_fully_instant_redeem(storage_cache, total_egld, min_egld_amount) {
            // Case 1: Full instant redemption
            (total_egld.clone(), BigUint::zero())
        } else if self.can_fully_pending_redeem(storage_cache, total_egld, min_egld_amount) {
            // Case 2: Full pending redemption + undelegation
            let difference = total_egld - &storage_cache.pending_egld;
            (storage_cache.pending_egld.clone(), difference)
        } else {
            // Case 3: Partial pending redemption + undelegation
            self.calculate_partial_undelegate(storage_cache, total_egld, min_egld_amount)
        }
    }

    fn can_fully_instant_redeem(
        &self,
        storage_cache: &mut StorageCache<Self>,
        total_egld: &BigUint,
        min_egld_amount: &BigUint,
    ) -> bool {
        total_egld == &storage_cache.pending_egld
            || (&storage_cache.pending_egld >= total_egld
                && (&storage_cache.pending_egld - total_egld) >= *min_egld_amount)
    }

    fn can_fully_pending_redeem(
        &self,
        storage_cache: &mut StorageCache<Self>,
        total_egld: &BigUint,
        min_egld_amount: &BigUint,
    ) -> bool {
        storage_cache.pending_egld > BigUint::from(0u64)
            && total_egld > &storage_cache.pending_egld
            && &(total_egld - &storage_cache.pending_egld) >= min_egld_amount
    }

    fn calculate_partial_undelegate(
        &self,
        storage_cache: &mut StorageCache<Self>,
        total_egld: &BigUint,
        min_egld_amount: &BigUint,
    ) -> (BigUint, BigUint) {
        let possible_instant_amount =
            self.calculate_instant_amount(total_egld, &storage_cache.pending_egld, min_egld_amount);
        if possible_instant_amount >= *min_egld_amount
            && (total_egld - &possible_instant_amount) >= *min_egld_amount
        {
            let undelegate_amount = total_egld - &possible_instant_amount;
            (possible_instant_amount, undelegate_amount)
        } else {
            // Fallback: full undelegation
            (BigUint::zero(), total_egld.clone())
        }
    }

    fn process_instant_redemption(
        &self,
        storage_cache: &mut StorageCache<Self>,
        caller: &ManagedAddress,
        payment: &EsdtTokenPayment<Self::Api>,
        total_egld: &BigUint,
        instant_amount: &BigUint,
    ) {
        if *instant_amount > BigUint::from(0u64) {
            let xegld_amount_to_burn = if instant_amount == total_egld {
                payment.amount.clone()
            } else {
                self.get_ls_amount(instant_amount, storage_cache)
            };

            storage_cache.pending_egld -= instant_amount;

            require!(
                &storage_cache.pending_egld >= &BigUint::from(MIN_EGLD_TO_DELEGATE)
                    || storage_cache.pending_egld == BigUint::zero(),
                ERROR_INSUFFICIENT_PENDING_EGLD
            );

            self.pool_remove_liquidity(&xegld_amount_to_burn, storage_cache);
            self.burn_ls_token(&xegld_amount_to_burn);
            self.tx().to(caller).egld(instant_amount).transfer();
        }
    }

    fn store_remaining_xegld(
        &self,
        storage_cache: &mut StorageCache<Self>,
        payment: &EsdtTokenPayment<Self::Api>,
        instant_amount: &BigUint,
    ) {
        let remaining_xegld = if payment.amount >= *instant_amount {
            &payment.amount - instant_amount
        } else {
            BigUint::zero()
        };

        storage_cache.pending_ls_for_unstake += remaining_xegld;
        let min_xegld_amount =
            self.get_ls_amount(&BigUint::from(MIN_EGLD_TO_DELEGATE), storage_cache);
        require!(
            storage_cache.pending_ls_for_unstake >= min_xegld_amount
                || storage_cache.pending_ls_for_unstake == BigUint::zero(),
            ERROR_INSUFFICIENT_PENDING_XEGLD
        );
    }

    fn validate_undelegate_conditions(
        &self,
        storage_cache: &mut StorageCache<Self>,
        payment: &EsdtTokenPayment<Self::Api>,
    ) {
        self.is_state_active(storage_cache.contract_state);

        require!(
            storage_cache.ls_token_id.is_valid_esdt_identifier(),
            ERROR_LS_TOKEN_NOT_ISSUED
        );

        require!(
            payment.token_identifier == storage_cache.ls_token_id,
            ERROR_BAD_PAYMENT_TOKEN
        );

        require!(payment.amount > 0, ERROR_BAD_PAYMENT_AMOUNT);
    }
}
