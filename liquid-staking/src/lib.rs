#![no_std]

multiversx_sc::derive_imports!();
multiversx_sc::imports!();

pub mod callback;
pub mod config;
pub mod constants;
pub mod contexts;
pub mod delegation;
pub mod errors;
pub mod events;
pub mod liquidity_pool;
pub mod manage;
pub mod migrate;
pub mod proxy;
pub mod score;
pub mod selection;
pub mod storage;
pub mod structs;
pub mod utils;
pub mod views;

use constants::*;
use contexts::base::*;
use errors::*;
use structs::{State, UnstakeTokenAttributes};

#[multiversx_sc::contract]
pub trait LiquidStaking<ContractReader>:
    score::ScoreModule
    + views::ViewsModule
    + config::ConfigModule
    + events::EventsModule
    + manage::ManageModule
    + storage::StorageModule
    + migrate::MigrateModule
    + callback::CallbackModule
    + selection::SelectionModule
    + utils::generic::UtilsModule
    + delegation::DelegationModule
    + liquidity_pool::LiquidityPoolModule
    + utils::delegate::DelegateUtilsModule
    + utils::un_delegation::UnDelegateUtilsModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[upgrade]
    fn upgrade(&self) {}

    /// Initializes the Liquid Staking contract with essential parameters, setting up
    /// the structure for fair delegation distribution across multiple providers.
    ///
    /// Arguments:
    /// - `accumulator_contract`: Address of the accumulator contract used for tracking the total EGLD stake.
    /// - `fees`: Fee structure applicable on staking activities.
    /// - `rounds_per_epoch`: Defines the staking epoch's duration in rounds.
    /// - `minimum_rounds`: Minimum required rounds for staking cycle.
    /// - `max_selected_providers`: Maximum number of staking providers chosen daily.
    /// - `max_delegation_addresses`: Sets a cap on the number of delegation addresses.
    /// - `unbond_period`: Duration, in epochs, required for unbonding of stakes.
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

    /// Delegates EGLD to the staking pool by minting xEGLD tokens for the user,
    /// while pending delegation funds accumulate for batch processing.
    ///
    /// Note: No immediate delegation occurs; instead, funds are held and distributed
    /// at set intervals across providers for efficient decentralization.
    /// Transaction value is used as the staked amount.
    #[payable("EGLD")]
    #[endpoint(delegate)]
    fn delegate(&self) {
        let mut storage_cache = StorageCache::new(self);

        let payment = self.call_value().egld_value().clone_value();

        self.validate_delegate_conditions(&mut storage_cache, &payment);

        let (pending, extra) =
            self.get_action_amount(&storage_cache.pending_egld_for_unstake, &payment);

        self.process_delegation(&mut storage_cache, &pending, &extra);
    }

    /// Initiates the un-delegation process, enabling users to withdraw their stake.
    /// Depending on the available pending EGLD in the contract, users can receive
    /// an instant return without fees or enter a 10-day unbonding period.
    #[payable("*")]
    #[endpoint(unDelegate)]
    fn un_delegate(&self) {
        let mut storage_cache = StorageCache::new(self);

        let payment = self.call_value().single_esdt();

        self.validate_undelegate_conditions(&mut storage_cache, &payment);

        let unstaked_egld = self.pool_remove_liquidity(&payment.amount, &mut storage_cache);
        self.burn_ls_token(&payment.amount);

        let (instant, undelegate) =
            self.get_action_amount(&storage_cache.pending_egld, &unstaked_egld);

        self.process_un_delegation(&mut storage_cache, &instant, &undelegate);
    }

    /// Withdraws funds once the un-delegation process is complete. If the unbonding period
    /// has passed, users can claim their EGLD. This endpoint ensures all conditions for
    /// unbonding are met before allowing withdrawals.
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
                    // In this case we partially give to the user the available amount and return the remaining MetaESDT to the user
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

        self.tx().to(&caller).egld(&to_send).transfer();
        self.emit_general_liquidity_event(&storage_cache);
    }
}
