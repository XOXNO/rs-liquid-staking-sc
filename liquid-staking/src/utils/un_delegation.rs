multiversx_sc::imports!();
use crate::{
    structs::UnstakeTokenAttributes, StorageCache, ERROR_BAD_PAYMENT_AMOUNT,
    ERROR_BAD_PAYMENT_TOKEN, ERROR_INSUFFICIENT_PENDING_EGLD,
    ERROR_INSUFFICIENT_UNSTAKE_PENDING_EGLD, ERROR_LS_TOKEN_NOT_ISSUED, MIN_EGLD_TO_DELEGATE,
};

#[multiversx_sc::module]
pub trait UnDelegateUtilsModule:
    crate::storage::StorageModule
    + crate::config::ConfigModule
    + crate::utils::generic::UtilsModule
    + crate::events::EventsModule
    + crate::score::ScoreModule
    + crate::selection::SelectionModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    fn process_un_delegation(
        &self,
        storage_cache: &mut StorageCache<Self>,
        egld_from_pending_used: &BigUint,
        egld_to_remove_liquidity: &BigUint,
    ) {
        let caller = self.blockchain().get_caller();

        self.process_instant_redemption(storage_cache, &caller, &egld_from_pending_used);

        self.undelegate_amount(storage_cache, &egld_to_remove_liquidity, &caller);

        self.emit_remove_liquidity_event(
            storage_cache,
            &(egld_to_remove_liquidity + egld_from_pending_used),
        );
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
        if *egld_to_unstake > BigUint::zero() {
            storage_cache.pending_egld_for_unstake += egld_to_unstake;

            require!(
                storage_cache.pending_egld_for_unstake >= BigUint::from(MIN_EGLD_TO_DELEGATE)
                    || storage_cache.pending_egld_for_unstake == BigUint::zero(),
                ERROR_INSUFFICIENT_UNSTAKE_PENDING_EGLD
            );

            let current_epoch = self.blockchain().get_block_epoch();
            let unbond_epoch = current_epoch + self.unbond_period().get();

            let virtual_position = UnstakeTokenAttributes {
                unstake_epoch: current_epoch,
                unbond_epoch,
            };

            let user_payment = self.mint_unstake_tokens(
                &virtual_position,
                egld_to_unstake,
                unbond_epoch,
                current_epoch,
            );

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
}
