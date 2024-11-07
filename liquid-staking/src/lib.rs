#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const MIN_GAS_FOR_ASYNC_CALL: u64 = 12_000_000;
pub const MIN_GAS_FOR_ASYNC_CALL_CLAIM_REWARDS: u64 = 2_500_000;
pub const MIN_GAS_FOR_CALLBACK: u64 = 6_000_000;
pub const MIN_GAS_FOR_WHITELIST_CALLBACK: u64 = 20_000_000;
pub const MIN_EGLD_TO_DELEGATE: u64 = 1_000_000_000_000_000_000;

pub mod callback;
pub mod config;
pub mod delegation;
pub mod errors;
pub mod manage;
pub mod proxy_accumulator;
pub mod proxy_delegation;
pub mod proxy_delegation_manager;
pub mod storage;
pub mod structs;
pub mod utils;
pub mod utils_delegation;
pub mod utils_un_delegation;
pub mod views;

pub mod migrate;

mod contexts;
mod events;
mod liquidity_pool;

use crate::errors::*;

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
    + migrate::MigrateModule
    + views::ViewsModule
    + utils_delegation::DelegateUtilsModule
    + utils_un_delegation::UnDelegateUtilsModule
    + delegation::DelegationModule
    + callback::CallbackModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[upgrade]
    fn upgrade(&self) {}

    #[init]
    fn init(
        &self,
        accumulator_contract: ManagedAddress,
        fees: BigUint,
        rounds_per_epoch: u64,
        minimum_rounds: u64,
        max_selected_providers: BigUint,
        max_delegation_addresses: usize,
        unbond_period: u64,
    ) {
        self.state().set(State::Inactive);

        require!(
            max_selected_providers >= BigUint::from(1u64),
            ERROR_MAX_SELECTED_PROVIDERS
        );

        require!(
            max_delegation_addresses >= 1,
            ERROR_MAX_CHANGED_DELEGATION_ADDRESSES
        );

        self.unbond_period().set(unbond_period);
        self.max_delegation_addresses()
            .set(max_delegation_addresses);
        self.max_selected_providers().set(max_selected_providers);

        self.accumulator_contract().set(accumulator_contract);
        self.fees().set(fees);
        self.rounds_per_epoch().set(rounds_per_epoch);
        self.minimum_rounds().set(minimum_rounds);
    }

    #[payable("EGLD")]
    #[endpoint(delegate)]
    fn delegate(&self) {
        let mut storage_cache = StorageCache::new(self);

        let payment = self.call_value().egld_value().clone_value();

        self.validate_delegate_conditions(&mut storage_cache, &payment);

        let (pending, extra) = self.get_delegate_amount(&mut storage_cache, &payment);

        self.process_delegation(&mut storage_cache, &pending, &extra);
    }

    #[payable("*")]
    #[endpoint(unDelegate)]
    fn un_delegate(&self) {
        let mut storage_cache = StorageCache::new(self);
        let payment = self.call_value().single_esdt();
        self.validate_undelegate_conditions(&mut storage_cache, &payment);

        let caller = self.blockchain().get_caller();

        let unstaked_egld = self.pool_remove_liquidity(&payment.amount, &mut storage_cache);
        self.burn_ls_token(&payment.amount);

        let (instant, undelegate) = self.get_undelegate_amount(&mut storage_cache, &unstaked_egld);

        self.process_instant_redemption(&mut storage_cache, &caller, &instant);

        self.undelegate_amount(&mut storage_cache, &undelegate, &caller);

        self.emit_remove_liquidity_event(&storage_cache, &unstaked_egld);
    }

    #[payable("*")]
    #[endpoint(withdraw)]
    fn withdraw(&self) {
        let mut storage_cache = StorageCache::new(self);
        self.is_state_active(storage_cache.contract_state);

        let caller = self.blockchain().get_caller();
        let payments = self.call_value().all_esdt_transfers();
        let unstake_token_id = self.unstake_token().get_token_id();
        let current_epoch = self.blockchain().get_block_epoch();

        let mut to_send = BigUint::zero();

        for payment in payments.iter() {
            require!(
                payment.token_identifier == unstake_token_id,
                ERROR_BAD_PAYMENT_TOKEN
            );

            require!(payment.amount > BigUint::zero(), ERROR_BAD_PAYMENT_AMOUNT);

            let unstake_token_attributes: UnstakeTokenAttributes = self
                .unstake_token()
                .get_token_attributes(payment.token_nonce);

            require!(
                current_epoch >= unstake_token_attributes.unbond_epoch,
                ERROR_UNSTAKE_PERIOD_NOT_PASSED
            );

            if storage_cache.total_withdrawn_egld >= payment.amount {
                self.burn_unstake_tokens(payment.token_nonce, &payment.amount);

                storage_cache.total_withdrawn_egld -= &payment.amount;
                to_send += payment.amount;
            } else {
                if storage_cache.total_withdrawn_egld > BigUint::zero() {
                    // In this case the required amount of the MetaESDT is higher than the available amount
                    // This case can happen only when the amount from the providers didn't arrive yet in the protocol
                    // In this case we partially give to the user the available amount and return the un claimed MetaESDT to the user
                    self.burn_unstake_tokens(
                        payment.token_nonce,
                        &storage_cache.total_withdrawn_egld,
                    );

                    let remaining_amount = payment.amount - &storage_cache.total_withdrawn_egld;

                    // Send the remaining amount to the user
                    self.tx()
                        .to(&caller)
                        .single_esdt(
                            &payment.token_identifier,
                            payment.token_nonce,
                            &remaining_amount,
                        )
                        .transfer();

                    // Send the amount to the user
                    to_send += storage_cache.total_withdrawn_egld.clone();

                    // Reset the total withdrawn amount to 0
                    storage_cache.total_withdrawn_egld = BigUint::zero();
                } else {
                    sc_panic!(ERROR_INSUFFICIENT_UNBONDED_AMOUNT);
                }
            }
        }

        if to_send > BigUint::zero() {
            self.tx().to(&caller).egld(&to_send).transfer();
            self.emit_general_liquidity_event(&storage_cache);
        }
    }
}
