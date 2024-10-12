use crate::{
    structs::UnstakeTokenAttributes, StorageCache, ERROR_BAD_PAYMENT_AMOUNT,
    ERROR_BAD_PAYMENT_TOKEN, ERROR_INSUFFICIENT_PENDING_EGLD,
    ERROR_INSUFFICIENT_UNSTAKE_PENDING_EGLD, ERROR_LS_TOKEN_NOT_ISSUED, MIN_EGLD_TO_DELEGATE,
};

pub const UNBOND_PERIOD: u64 = 10;
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
        unstaked_egld: &BigUint,
    ) -> (BigUint, BigUint) {
        let min_egld_amount = &BigUint::from(MIN_EGLD_TO_DELEGATE);
        if self.can_fully_instant_redeem(storage_cache, unstaked_egld, min_egld_amount) {
            // Case 1: Full instant redemption
            (unstaked_egld.clone(), BigUint::zero())
        } else if self.can_fully_pending_redeem(storage_cache, unstaked_egld, min_egld_amount) {
            // Case 2: Full pending redemption + undelegation
            let difference = unstaked_egld - &storage_cache.pending_egld;
            (storage_cache.pending_egld.clone(), difference)
        } else {
            // Case 3: Partial pending redemption + undelegation
            self.calculate_partial_undelegate(storage_cache, unstaked_egld, min_egld_amount)
        }
    }

    fn can_fully_instant_redeem(
        &self,
        storage_cache: &mut StorageCache<Self>,
        unstaked_egld: &BigUint,
        min_egld_amount: &BigUint,
    ) -> bool {
        unstaked_egld == &storage_cache.pending_egld
            || (&storage_cache.pending_egld >= unstaked_egld
                && (&storage_cache.pending_egld - unstaked_egld) >= *min_egld_amount)
    }

    fn can_fully_pending_redeem(
        &self,
        storage_cache: &mut StorageCache<Self>,
        total_egld: &BigUint,
        min_egld_amount: &BigUint,
    ) -> bool {
        storage_cache.pending_egld > BigUint::zero()
            && total_egld > &storage_cache.pending_egld
            && &(total_egld - &storage_cache.pending_egld) >= min_egld_amount
    }

    fn calculate_partial_undelegate(
        &self,
        storage_cache: &mut StorageCache<Self>,
        unstaked_egld: &BigUint,
        min_egld_amount: &BigUint,
    ) -> (BigUint, BigUint) {
        let possible_instant_amount = self.calculate_instant_amount(
            unstaked_egld,
            &storage_cache.pending_egld,
            min_egld_amount,
        );
        if possible_instant_amount >= *min_egld_amount
            && unstaked_egld >= &possible_instant_amount
            && (unstaked_egld - &possible_instant_amount) >= *min_egld_amount
        {
            let undelegate_amount = unstaked_egld - &possible_instant_amount;
            (possible_instant_amount, undelegate_amount)
        } else {
            // Fallback: full undelegation
            (BigUint::zero(), unstaked_egld.clone())
        }
    }

    fn process_instant_redemption(
        &self,
        storage_cache: &mut StorageCache<Self>,
        caller: &ManagedAddress,
        instant_amount: &BigUint,
    ) {
        if *instant_amount > BigUint::zero() {
            storage_cache.pending_egld -= instant_amount;

            require!(
                &storage_cache.pending_egld >= &BigUint::from(MIN_EGLD_TO_DELEGATE)
                    || storage_cache.pending_egld == BigUint::zero(),
                ERROR_INSUFFICIENT_PENDING_EGLD
            );

            self.tx().to(caller).egld(instant_amount).transfer();
        }
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

        require!(payment.amount > BigUint::zero(), ERROR_BAD_PAYMENT_AMOUNT);
    }

    fn undelegate_amount(
        &self,
        storage_cache: &mut StorageCache<Self>,
        egld_to_unstake: &BigUint,
        caller: &ManagedAddress,
    ) {
        if *egld_to_unstake == BigUint::zero() {
            return;
        }

        storage_cache.pending_egld_for_unstake += egld_to_unstake;

        require!(
            storage_cache.pending_egld_for_unstake >= BigUint::from(MIN_EGLD_TO_DELEGATE)
                || storage_cache.pending_egld_for_unstake == BigUint::zero(),
            ERROR_INSUFFICIENT_UNSTAKE_PENDING_EGLD
        );

        let current_epoch = self.blockchain().get_block_epoch();
        let unbond_epoch = current_epoch + UNBOND_PERIOD;

        let virtual_position = UnstakeTokenAttributes {
            unstake_epoch: current_epoch,
            unbond_epoch,
        };

        let user_payment =
            self.mint_unstake_tokens(&virtual_position, egld_to_unstake, unbond_epoch);

        self.tx()
            .to(caller)
            .single_esdt(
                &user_payment.token_identifier,
                user_payment.token_nonce,
                &user_payment.amount,
            )
            .transfer();
    }
}
