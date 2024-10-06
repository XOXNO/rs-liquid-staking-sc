#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const MIN_GAS_FOR_ASYNC_CALL: u64 = 12_000_000;
pub const MIN_GAS_FOR_CALLBACK: u64 = 6_000_000;
pub const MIN_EGLD_TO_DELEGATE: u64 = 1_000_000_000_000_000_000;
pub const MAX_DELEGATION_ADDRESSES: usize = 50;

pub mod accumulator;
pub mod callback;
pub mod config;
pub mod delegate_utils;
pub mod delegation;
pub mod delegation_proxy;
pub mod errors;
pub mod manage;
pub mod storage;
pub mod structs;
pub mod un_delegate_utils;
pub mod utils;
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
    + delegate_utils::DelegateUtilsModule
    + un_delegate_utils::UnDelegateUtilsModule
    + delegation::DelegationModule
    + callback::CallbackModule
    + multiversx_sc_modules::ongoing_operation::OngoingOperationModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[init]
    fn init(
        &self,
        accumulator_contract: ManagedAddress,
        fees: BigUint,
        rounds_per_epoch: u64,
        minimum_rounds: u64,
    ) {
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
        self.rounds_per_epoch().set(rounds_per_epoch);
        self.minimum_rounds().set(minimum_rounds);
    }

    #[payable("EGLD")]
    #[endpoint(delegate)]
    fn delegate(&self) {
        let mut storage_cache = StorageCache::new(self);

        let payment = self.call_value().egld_value().clone_value();

        self.validate_delegate_conditions(&mut storage_cache, &payment);

        let ls_amount = self.get_ls_amount(&payment, &mut storage_cache);

        let min_xegld_amount =
            self.get_ls_amount(&BigUint::from(MIN_EGLD_TO_DELEGATE), &mut storage_cache);

        let (xegld_from_pending, instant_unbond_balance, egld_to_add_liquidity) = self
            .determine_delegate_amounts(
                &mut storage_cache,
                &payment,
                &ls_amount,
                &min_xegld_amount,
            );

        self.process_redemption_and_staking(
            &mut storage_cache,
            xegld_from_pending,
            instant_unbond_balance,
            egld_to_add_liquidity,
        );
    }

    #[payable("*")]
    #[endpoint(unDelegate)]
    fn un_delegate(&self) {
        let mut storage_cache = StorageCache::new(self);
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();

        self.validate_undelegate_conditions(&mut storage_cache, &payment);

        let total_egld = self.get_egld_amount(&payment.amount, &mut storage_cache);
        let min_egld_amount = BigUint::from(MIN_EGLD_TO_DELEGATE);

        let (instant_amount, undelegate_amount) =
            self.determine_undelegate_amounts(&mut storage_cache, &total_egld, &min_egld_amount);

        self.process_instant_redemption(
            &mut storage_cache,
            &caller,
            &payment,
            &total_egld,
            &instant_amount,
        );

        if undelegate_amount > BigUint::from(0u64) {
            self.undelegate_amount(&undelegate_amount, &caller);
        }

        self.store_remaining_xegld(&mut storage_cache, &payment, &instant_amount);

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
    }
}
