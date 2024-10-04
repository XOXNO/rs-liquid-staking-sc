#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const DEFAULT_GAS_TO_CLAIM_REWARDS: u64 = 6_000_000;
pub const MIN_GAS_FOR_ASYNC_CALL: u64 = 12_000_000;
pub const MIN_GAS_FOR_CALLBACK: u64 = 12_000_000;
pub const MIN_EGLD_TO_DELEGATE: u64 = 1_000_000_000_000_000_000;
pub const MAX_DELEGATION_ADDRESSES: usize = 50;

pub mod accumulator;
pub mod config;
pub mod delegation;
pub mod delegation_proxy;
pub mod errors;
pub mod manage;
pub mod storage;
pub mod structs;
pub mod utils;
pub mod views;
pub mod callback;

mod contexts;
mod events;
mod liquidity_pool;

use crate::{
    errors::*,
    structs::{ClaimStatus, ClaimStatusType},
};

use contexts::base::*;
use structs::{State, UnstakeTokenAttributes};

#[multiversx_sc::contract]
pub trait LiquidStaking<ContractReader>:
    liquidity_pool::LiquidityPoolModule
    + config::ConfigModule
    + events::EventsModule
    + utils::UtilsModule
    + storage::StorageModule
    + manage::ManageModule
    + views::ViewsModule
    + delegation::DelegationModule
    + callback::CallbackModule
    + multiversx_sc_modules::ongoing_operation::OngoingOperationModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[init]
    fn init(&self, accumulator_contract: ManagedAddress, fees: BigUint) {
        self.state().set(State::Inactive);
        self.max_delegation_addresses()
            .set(MAX_DELEGATION_ADDRESSES);

        let current_epoch = self.blockchain().get_block_epoch();
        let claim_status = ClaimStatus {
            status: ClaimStatusType::Insufficent,
            last_claim_epoch: current_epoch,

            current_node: 0,
        };

        self.delegation_claim_status().set(claim_status);
        self.accumulator_contract().set(accumulator_contract);
        self.fees().set(fees);
    }

    #[payable("EGLD")]
    #[endpoint(delegate)]
    fn delegate(&self) {
        let mut storage_cache = StorageCache::new(self);
        let caller = self.blockchain().get_caller();

        let payment = self.call_value().egld_value().clone_value();
        require!(
            self.is_state_active(storage_cache.contract_state),
            ERROR_NOT_ACTIVE
        );

        let ls_amount = self.get_ls_amount(&payment, &mut storage_cache);
        let min_xegld_amount =
            self.get_ls_amount(&BigUint::from(MIN_EGLD_TO_DELEGATE), &mut storage_cache);

        let mut instant_unbound_balance = BigUint::zero();
        let mut xegld_from_pending = BigUint::zero();
        let mut egld_to_add_liquidity = BigUint::zero();

        if &storage_cache.pending_ls_for_unstake >= &min_xegld_amount {
            if ls_amount == storage_cache.pending_ls_for_unstake
                || ls_amount <= &storage_cache.pending_ls_for_unstake - &min_xegld_amount
            {
                // Case 1: Full instant staking
                xegld_from_pending = ls_amount.clone();
                instant_unbound_balance = payment.clone();
            } else {
                // Case 2: Partial instant staking or full normal staking
                // Here ls_amount is always greater than pending_ls_for_unstake with at least min_xegld_amount
                let difference = &ls_amount - &storage_cache.pending_ls_for_unstake;

                if difference >= min_xegld_amount {
                    // Case 2: Full pending redemption + normal staking
                    xegld_from_pending = storage_cache.pending_ls_for_unstake.clone();
                    instant_unbound_balance =
                        self.get_egld_amount(&xegld_from_pending, &storage_cache);
                    egld_to_add_liquidity = &payment - &instant_unbound_balance;
                } else {
                    // Case 3: Attempt partial pending redemption + normal staking
                    let possible_instant_amount = self.calculate_instant_amount(
                        &ls_amount,
                        &storage_cache.pending_ls_for_unstake,
                        &min_xegld_amount,
                    );

                    if possible_instant_amount >= min_xegld_amount
                        && (&ls_amount - &possible_instant_amount) >= min_xegld_amount
                    {
                        // We can do partial redemption
                        xegld_from_pending = possible_instant_amount;
                        instant_unbound_balance =
                            self.get_egld_amount(&xegld_from_pending, &storage_cache);
                        egld_to_add_liquidity = &payment - &instant_unbound_balance;
                    } else {
                        // Fallback: full normal staking
                        egld_to_add_liquidity = payment.clone();
                    }
                }
            }
        } else {
            // Fallback: full normal staking
            egld_to_add_liquidity = payment.clone();
        }

        // Ensure the remaining pending EGLD is not less than 1 EGLD
        require!(
            &storage_cache.pending_egld + &egld_to_add_liquidity
                >= BigUint::from(MIN_EGLD_TO_DELEGATE),
            ERROR_INSUFFICIENT_PENDING_EGLD
        );

        // Process instant staking
        if xegld_from_pending > 0 {
            storage_cache.pending_ls_for_unstake -= &xegld_from_pending;
            storage_cache.total_withdrawn_egld += &instant_unbound_balance; // Ensure the remaining pending xEGLD is not less than min_xegld_amount or is zero
            require!(
                storage_cache.pending_ls_for_unstake >= min_xegld_amount
                    || storage_cache.pending_ls_for_unstake == BigUint::zero(),
                ERROR_INSUFFICIENT_PENDING_XEGLD
            );
            self.send()
                .direct_esdt(&caller, &storage_cache.ls_token_id, 0, &xegld_from_pending);
        }

        // Process normal staking
        if egld_to_add_liquidity > 0 {
            storage_cache.pending_egld += &egld_to_add_liquidity;
            let ls_amount = self.pool_add_liquidity(&egld_to_add_liquidity, &mut storage_cache);
            let user_payment = self.mint_ls_token(ls_amount);

            self.send().direct_esdt(
                &caller,
                &user_payment.token_identifier,
                user_payment.token_nonce,
                &user_payment.amount,
            );
        }

        self.emit_add_liquidity_event(&storage_cache, &caller, ls_amount);
    }

    #[payable("*")]
    #[endpoint(unDelegate)]
    fn un_delegate(&self) {
        let mut storage_cache = StorageCache::new(self);
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();

        require!(
            self.is_state_active(storage_cache.contract_state),
            ERROR_NOT_ACTIVE
        );

        require!(
            storage_cache.ls_token_id.is_valid_esdt_identifier(),
            ERROR_LS_TOKEN_NOT_ISSUED
        );

        require!(
            payment.token_identifier == storage_cache.ls_token_id,
            ERROR_BAD_PAYMENT_TOKEN
        );

        require!(payment.amount > 0, ERROR_BAD_PAYMENT_AMOUNT);

        let total_egld = self.get_egld_amount(&payment.amount, &mut storage_cache);
        let min_egld_amount = BigUint::from(MIN_EGLD_TO_DELEGATE);

        let mut instant_amount = BigUint::zero();
        let mut undelegate_amount = BigUint::zero();

        if &storage_cache.pending_egld >= &min_egld_amount {
            if total_egld == storage_cache.pending_egld
                || total_egld <= &storage_cache.pending_egld - &min_egld_amount
            {
                // Case 1: Full instant redemption
                instant_amount = total_egld.clone();
            } else {
                // Always total_egld is greater than storage_cache.pending_egld with at least MIN_EGLD_TO_DELEGATE

                let difference = &total_egld - &storage_cache.pending_egld;

                // Basically we can use all the pending EGLD to instant undelegate and the rest to undelegate normally as the difference is always >= MIN_EGLD_TO_DELEGATE
                if difference >= min_egld_amount {
                    // Case 2: Full pending redemption + undelegation
                    undelegate_amount = difference;
                    instant_amount = storage_cache.pending_egld.clone();
                } else {
                    // Case 3: Attempt partial pending redemption + undelegation
                    let possible_instant_amount = self.calculate_instant_amount(
                        &total_egld,
                        &storage_cache.pending_egld,
                        &min_egld_amount,
                    );

                    if possible_instant_amount >= min_egld_amount
                        && (&total_egld - &possible_instant_amount) >= min_egld_amount
                    {
                        // We can do partial redemption
                        instant_amount = possible_instant_amount.clone();
                        undelegate_amount = &total_egld - &instant_amount;
                        require!(
                            undelegate_amount >= min_egld_amount,
                            ERROR_INSUFFICIENT_UNSTAKE_AMOUNT
                        );
                    } else {
                        // Fallback: full undelegation
                        undelegate_amount = total_egld.clone();
                    }
                }
            }
        } else {
            // Fallback: full undelegation
            undelegate_amount = total_egld.clone();
        }

        // Process instant redemption
        let mut xegld_amount_to_burn = BigUint::from(0u64);
        if instant_amount > BigUint::from(0u64) {
            // Determine if we are doing a full instant redemption or partial (good for dust decimal handling)
            xegld_amount_to_burn = if &instant_amount == &total_egld {
                payment.amount.clone()
            } else {
                self.get_ls_amount(&instant_amount, &mut storage_cache)
            };

            self.send().direct_egld(&caller, &instant_amount);
            require!(
                &storage_cache.pending_egld >= &min_egld_amount
                    || storage_cache.pending_egld == BigUint::zero(),
                ERROR_INSUFFICIENT_PENDING_EGLD
            );
            self.pool_remove_liquidity(&xegld_amount_to_burn, &mut storage_cache);
            self.burn_ls_token(&xegld_amount_to_burn);
            storage_cache.pending_egld -= instant_amount.clone();
        }

        if undelegate_amount > BigUint::from(0u64) {
            self.undelegate_amount(&undelegate_amount, &caller);
        }

        // Store the remaining SEGLD for future redemption
        let remaining_xegld = if payment.amount >= xegld_amount_to_burn {
            &payment.amount - &xegld_amount_to_burn
        } else {
            BigUint::zero()
        };

        storage_cache.pending_ls_for_unstake += remaining_xegld;
        let min_xegld_amount = self.get_ls_amount(&min_egld_amount, &mut storage_cache);
        require!(
            storage_cache.pending_ls_for_unstake >= min_xegld_amount
                || storage_cache.pending_ls_for_unstake == BigUint::zero(),
            ERROR_INSUFFICIENT_PENDING_XEGLD
        );

        self.emit_remove_liquidity_event(&storage_cache, &payment.amount, &total_egld);
    }

    #[payable("*")]
    #[endpoint(withdraw)]
    fn withdraw(&self) {
        let mut storage_cache = StorageCache::new(self);
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();

        require!(
            self.is_state_active(storage_cache.contract_state),
            ERROR_NOT_ACTIVE
        );

        require!(
            payment.token_identifier == self.unstake_token().get_token_id(),
            ERROR_BAD_PAYMENT_TOKEN
        );

        require!(payment.amount > 0, ERROR_BAD_PAYMENT_AMOUNT);

        let unstake_token_attributes: UnstakeTokenAttributes<Self::Api> = self
            .unstake_token()
            .get_token_attributes(payment.token_nonce);

        let current_epoch = self.blockchain().get_block_epoch();

        require!(
            current_epoch >= unstake_token_attributes.unbond_epoch,
            ERROR_UNSTAKE_PERIOD_NOT_PASSED
        );

        require!(
            storage_cache.total_withdrawn_egld >= unstake_token_attributes.unstake_amount,
            ERROR_INSUFFICIENT_UNBONDED_AMOUNT
        );

        self.burn_unstake_tokens(payment.token_nonce);

        storage_cache.total_withdrawn_egld -= &unstake_token_attributes.unstake_amount;

        self.tx()
            .to(&caller)
            .egld(&unstake_token_attributes.unstake_amount)
            .transfer();
    }
}
