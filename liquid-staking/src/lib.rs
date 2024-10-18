#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const MIN_GAS_FOR_ASYNC_CALL: u64 = 12_000_000;
pub const MIN_GAS_FOR_CALLBACK: u64 = 6_000_000;
pub const MIN_EGLD_TO_DELEGATE: u64 = 1_000_000_000_000_000_000;

pub mod accumulator;
pub mod callback;
pub mod config;
pub mod delegation;
pub mod delegation_proxy;
pub mod errors;
pub mod manage;
pub mod storage;
pub mod structs;
pub mod utils;
pub mod utils_delegation;
pub mod utils_un_delegation;
pub mod views;

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
    + utils_delegation::DelegateUtilsModule
    + utils_un_delegation::UnDelegateUtilsModule
    + delegation::DelegationModule
    + callback::CallbackModule
    + multiversx_sc_modules::ongoing_operation::OngoingOperationModule
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

        self.max_delegation_addresses()
            .set(max_delegation_addresses);
        self.max_selected_providers().set(max_selected_providers);

        let current_epoch = self.blockchain().get_block_epoch();
        let claim_status = ClaimStatus {
            status: ClaimStatusType::Insufficent,
            last_claim_epoch: current_epoch,
            current_node: 0,
        };

        self.delegation_claim_status().set(claim_status);
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

        let (egld_from_pending_used, egld_to_add_liquidity) =
            self.determine_delegate_amounts(&mut storage_cache, &payment);

        self.process_redemption_and_staking(
            &mut storage_cache,
            &egld_from_pending_used,
            &egld_to_add_liquidity,
        );
    }

    #[payable("*")]
    #[endpoint(unDelegate)]
    fn un_delegate(&self) {
        let mut storage_cache = StorageCache::new(self);
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();

        self.validate_undelegate_conditions(&mut storage_cache, &payment);

        let unstaked_egld = self.pool_remove_liquidity(&payment.amount, &mut storage_cache);
        self.burn_ls_token(&payment.amount);

        let (instant_amount, to_undelegate_amount) =
            self.determine_undelegate_amounts(&mut storage_cache, &unstaked_egld);

        self.process_instant_redemption(&mut storage_cache, &caller, &instant_amount);

        self.undelegate_amount(&mut storage_cache, &to_undelegate_amount, &caller);

        self.emit_remove_liquidity_event(&storage_cache, &unstaked_egld);
    }

    #[payable("*")]
    #[endpoint(withdraw)]
    fn withdraw(&self) {
        let mut storage_cache = StorageCache::new(self);
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();

        self.is_state_active(storage_cache.contract_state);

        require!(
            payment.token_identifier == self.unstake_token().get_token_id(),
            ERROR_BAD_PAYMENT_TOKEN
        );

        require!(payment.amount > BigUint::zero(), ERROR_BAD_PAYMENT_AMOUNT);

        let unstake_token_attributes: UnstakeTokenAttributes = self
            .unstake_token()
            .get_token_attributes(payment.token_nonce);

        let current_epoch = self.blockchain().get_block_epoch();

        require!(
            current_epoch >= unstake_token_attributes.unbond_epoch,
            ERROR_UNSTAKE_PERIOD_NOT_PASSED
        );

        require!(
            storage_cache.total_withdrawn_egld >= payment.amount,
            ERROR_INSUFFICIENT_UNBONDED_AMOUNT
        );

        self.burn_unstake_tokens(payment.token_nonce, &payment.amount);

        storage_cache.total_withdrawn_egld -= &payment.amount;

        self.tx().to(&caller).egld(&payment.amount).transfer();

        self.emit_general_liquidity_event(&storage_cache);
    }
}
