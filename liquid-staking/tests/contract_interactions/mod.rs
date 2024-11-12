use crate::contract_setup::LiquidStakingContractSetup;
use crate::utils::*;
use liquid_staking::config::ConfigModule;
use liquid_staking::manage::ManageModule;
use liquid_staking::storage::StorageModule;
use liquid_staking::structs::UnstakeTokenAttributes;
use liquid_staking::views::ViewsModule;
use liquid_staking::LiquidStaking;
use multiversx_sc::{imports::OptionalValue, types::Address};
use multiversx_sc_scenario::{managed_address, num_bigint, rust_biguint, DebugApi};

use delegation_mock::*;
use liquid_staking::delegation::DelegationModule;

impl<LiquidStakingContractObjBuilder> LiquidStakingContractSetup<LiquidStakingContractObjBuilder>
where
    LiquidStakingContractObjBuilder: 'static + Copy + Fn() -> liquid_staking::ContractObj<DebugApi>,
{
    pub fn deploy_staking_contract(
        &mut self,
        owner_address: &Address,
        egld_balance: u64,
        total_staked: u64,
        delegation_contract_cap: u64,
        nr_nodes: u64,
        apy: u64,
    ) -> Address {
        let rust_zero = rust_biguint!(0u64);
        let rust_one_egld = exp18(1);
        let egld_balance_biguint = &exp18(egld_balance);
        let total_staked_biguint = exp18(total_staked);
        let delegation_contract_cap_biguint = exp18(delegation_contract_cap);

        self.b_mock
            .set_egld_balance(owner_address, &(egld_balance_biguint + &rust_one_egld));

        let delegation_wrapper = self.b_mock.create_sc_account(
            &rust_zero,
            Some(owner_address),
            delegation_mock::contract_obj,
            "delegation-mock.wasm",
        );

        self.b_mock
            .execute_tx(owner_address, &delegation_wrapper, &rust_zero, |sc| {
                sc.init();
            })
            .assert_ok();

        self.b_mock
            .execute_tx(
                owner_address,
                &delegation_wrapper,
                egld_balance_biguint,
                |sc| {
                    sc.deposit_egld();
                },
            )
            .assert_ok();

        self.b_mock
            .execute_tx(owner_address, &self.sc_wrapper, &rust_one_egld, |sc| {
                sc.whitelist_delegation_contract(
                    managed_address!(delegation_wrapper.address_ref()),
                    managed_address!(owner_address),
                    to_managed_biguint(total_staked_biguint),
                    to_managed_biguint(delegation_contract_cap_biguint),
                    nr_nodes,
                    apy,
                );
            })
            .assert_ok();

        delegation_wrapper.address_ref().clone()
    }

    pub fn set_inactive_state(&mut self, caller: &Address) {
        let rust_zero = rust_biguint!(0u64);
        self.b_mock
            .execute_tx(caller, &self.sc_wrapper, &rust_zero, |sc| {
                sc.set_state_inactive();
            })
            .assert_ok();
    }

    pub fn update_staking_contract_params(
        &mut self,
        owner_address: &Address,
        contract_address: &Address,
        total_staked: u64,
        delegation_contract_cap: u64,
        nr_nodes: u64,
        apy: u64,
        is_eligible: bool,
    ) {
        let rust_zero = rust_biguint!(0u64);
        let total_staked_biguint = exp18(total_staked);
        let delegation_contract_cap_biguint = exp18(delegation_contract_cap);

        self.b_mock
            .execute_tx(owner_address, &self.sc_wrapper, &rust_zero, |sc| {
                sc.change_delegation_contract_params(
                    managed_address!(contract_address),
                    to_managed_biguint(total_staked_biguint),
                    to_managed_biguint(delegation_contract_cap_biguint),
                    nr_nodes,
                    apy,
                    is_eligible,
                );
            })
            .assert_ok();
    }

    pub fn add_liquidity(&mut self, caller: &Address, payment_amount: u64) {
        self.b_mock
            .execute_tx(caller, &self.sc_wrapper, &exp18(payment_amount), |sc| {
                sc.delegate();
            })
            .assert_ok();
    }

    pub fn add_liquidity_error(&mut self, caller: &Address, payment_amount: u64, error: &[u8]) {
        self.b_mock
            .execute_tx(caller, &self.sc_wrapper, &exp18(payment_amount), |sc| {
                sc.delegate();
            })
            .assert_error(4, bytes_to_str(error));
    }

    pub fn add_liquidity_exp17(&mut self, caller: &Address, payment_amount: u64) {
        self.b_mock
            .execute_tx(caller, &self.sc_wrapper, &exp17(payment_amount), |sc| {
                sc.delegate();
            })
            .assert_ok();
    }

    pub fn add_liquidity_exp17_error(
        &mut self,
        caller: &Address,
        payment_amount: u64,
        error: &[u8],
    ) {
        self.b_mock
            .execute_tx(caller, &self.sc_wrapper, &exp17(payment_amount), |sc| {
                sc.delegate();
            })
            .assert_error(4, bytes_to_str(error));
    }

    pub fn remove_liquidity(
        &mut self,
        caller: &Address,
        payment_token: &[u8],
        payment_amount: u64,
    ) {
        self.b_mock
            .execute_esdt_transfer(
                caller,
                &self.sc_wrapper,
                payment_token,
                0,
                &exp18(payment_amount),
                |sc| {
                    sc.un_delegate();
                },
            )
            .assert_ok();
    }

    pub fn remove_liquidity_exp17(
        &mut self,
        caller: &Address,
        payment_token: &[u8],
        payment_amount: u64,
    ) {
        self.b_mock
            .execute_esdt_transfer(
                caller,
                &self.sc_wrapper,
                payment_token,
                0,
                &exp17(payment_amount),
                |sc| {
                    sc.un_delegate();
                },
            )
            .assert_ok();
    }

    pub fn remove_liquidity_error(
        &mut self,
        caller: &Address,
        payment_token: &[u8],
        payment_amount: u64,
        error: &[u8],
    ) {
        self.b_mock
            .execute_esdt_transfer(
                caller,
                &self.sc_wrapper,
                payment_token,
                0,
                &exp18(payment_amount),
                |sc| {
                    sc.un_delegate();
                },
            )
            .assert_error(4, bytes_to_str(error));
    }

    pub fn remove_liquidity_exp17_error(
        &mut self,
        caller: &Address,
        payment_token: &[u8],
        payment_amount: u64,
        error: &[u8],
    ) {
        self.b_mock
            .execute_esdt_transfer(
                caller,
                &self.sc_wrapper,
                payment_token,
                0,
                &exp17(payment_amount),
                |sc| {
                    sc.un_delegate();
                },
            )
            .assert_error(4, bytes_to_str(error));
    }

    pub fn claim_rewards(&mut self, caller: &Address) {
        let rust_zero = rust_biguint!(0u64);
        self.b_mock
            .execute_tx(caller, &self.sc_wrapper, &rust_zero, |sc| {
                sc.claim_rewards();
            })
            .assert_ok();
    }

    pub fn claim_rewards_error(&mut self, caller: &Address, error: &[u8]) {
        let rust_zero = rust_biguint!(0u64);
        self.b_mock
            .execute_tx(caller, &self.sc_wrapper, &rust_zero, |sc| {
                sc.claim_rewards();
            })
            .assert_error(4, bytes_to_str(error));
    }

    pub fn delegate_rewards(&mut self, caller: &Address) {
        let rust_zero = rust_biguint!(0u64);
        self.b_mock
            .execute_tx(caller, &self.sc_wrapper, &rust_zero, |sc| {
                sc.delegate_rewards();
            })
            .assert_ok();
    }

    pub fn delegate_rewards_error(&mut self, caller: &Address, error: &[u8]) {
        let rust_zero = rust_biguint!(0u64);
        self.b_mock
            .execute_tx(caller, &self.sc_wrapper, &rust_zero, |sc| {
                sc.delegate_rewards();
            })
            .assert_error(4, bytes_to_str(error));
    }

    pub fn delegate_pending(&mut self, caller: &Address, amount: OptionalValue<u64>) {
        let rust_zero = rust_biguint!(0u64);
        self.b_mock
            .execute_tx(caller, &self.sc_wrapper, &rust_zero, |sc| {
                sc.delegate_pending(match amount {
                    OptionalValue::Some(amount) => multiversx_sc::imports::OptionalValue::Some(
                        to_managed_biguint(exp18(amount)),
                    ),
                    OptionalValue::None => multiversx_sc::imports::OptionalValue::None,
                });
            })
            .assert_ok();
    }

    pub fn delegate_pending_error(
        &mut self,
        caller: &Address,
        amount: OptionalValue<u64>,
        error: &[u8],
    ) {
        let rust_zero = rust_biguint!(0u64);
        self.b_mock
            .execute_tx(caller, &self.sc_wrapper, &rust_zero, |sc| {
                sc.delegate_pending(match amount {
                    OptionalValue::Some(amount) => multiversx_sc::imports::OptionalValue::Some(
                        to_managed_biguint(exp17(amount)),
                    ),
                    OptionalValue::None => multiversx_sc::imports::OptionalValue::None,
                });
            })
            .assert_error(4, bytes_to_str(error));
    }

    pub fn un_delegate_pending(&mut self, caller: &Address, amount: OptionalValue<u64>) {
        let rust_zero = rust_biguint!(0u64);
        self.b_mock
            .execute_tx(caller, &self.sc_wrapper, &rust_zero, |sc| {
                sc.un_delegate_pending(match amount {
                    OptionalValue::Some(amount) => multiversx_sc::imports::OptionalValue::Some(
                        to_managed_biguint(exp18(amount)),
                    ),
                    OptionalValue::None => multiversx_sc::imports::OptionalValue::None,
                });
            })
            .assert_ok();
    }

    pub fn un_delegate_pending_error(
        &mut self,
        caller: &Address,
        amount: OptionalValue<u64>,
        error: &[u8],
    ) {
        let rust_zero = rust_biguint!(0u64);
        self.b_mock
            .execute_tx(caller, &self.sc_wrapper, &rust_zero, |sc| {
                sc.un_delegate_pending(match amount {
                    OptionalValue::Some(amount) => multiversx_sc::imports::OptionalValue::Some(
                        to_managed_biguint(exp17(amount)),
                    ),
                    OptionalValue::None => multiversx_sc::imports::OptionalValue::None,
                });
            })
            .assert_error(4, bytes_to_str(error));
    }

    pub fn withdraw_pending(&mut self, caller: &Address, contracts: &Address) {
        let rust_zero = rust_biguint!(0u64);

        self.b_mock
            .execute_tx(caller, &self.sc_wrapper, &rust_zero, |sc| {
                sc.withdraw_pending(managed_address!(contracts));
            })
            .assert_ok();
    }

    pub fn withdraw_pending_error(&mut self, caller: &Address, contracts: &Address, error: &[u8]) {
        let rust_zero = rust_biguint!(0u64);

        self.b_mock
            .execute_tx(caller, &self.sc_wrapper, &rust_zero, |sc| {
                sc.withdraw_pending(managed_address!(contracts));
            })
            .assert_error(4, bytes_to_str(error));
    }

    pub fn withdraw(
        &mut self,
        caller: &Address,
        payment_token: &[u8],
        token_nonce: u64,
        amount: num_bigint::BigUint,
    ) {
        self.b_mock
            .execute_esdt_transfer(
                caller,
                &self.sc_wrapper,
                payment_token,
                token_nonce,
                &amount,
                |sc| {
                    sc.withdraw();
                },
            )
            .assert_ok();
    }

    pub fn withdraw_error(
        &mut self,
        caller: &Address,
        payment_token: &[u8],
        token_nonce: u64,
        amount: num_bigint::BigUint,
        error: &[u8],
    ) {
        self.b_mock
            .execute_esdt_transfer(
                caller,
                &self.sc_wrapper,
                payment_token,
                token_nonce,
                &amount,
                |sc| {
                    sc.withdraw();
                },
            )
            .assert_error(4, bytes_to_str(error));
    }

    pub fn setup_new_user(&mut self, egld_token_amount: u64) -> Address {
        let rust_zero = rust_biguint!(0);

        let new_user = self.b_mock.create_user_account(&rust_zero);
        self.b_mock
            .set_egld_balance(&new_user, &exp18(egld_token_amount));
        new_user
    }

    pub fn check_user_balance(&self, address: &Address, token_id: &[u8], token_balance: u64) {
        self.b_mock
            .check_esdt_balance(address, token_id, &exp18(token_balance));
    }

    pub fn check_user_balance_exp17(&self, address: &Address, token_id: &[u8], token_balance: u64) {
        self.b_mock
            .check_esdt_balance(address, token_id, &exp17(token_balance));
    }

    pub fn check_user_balance_denominated(
        &self,
        address: &Address,
        token_id: &[u8],
        token_balance: u128,
    ) {
        self.b_mock.check_esdt_balance(
            address,
            token_id,
            &num_bigint::BigUint::from(token_balance),
        );
    }

    pub fn check_user_egld_balance(&self, address: &Address, token_balance: u64) {
        self.b_mock
            .check_egld_balance(address, &exp18(token_balance));
    }
    pub fn check_user_egld_balance_exp17(&self, address: &Address, token_balance: u64) {
        self.b_mock
            .check_egld_balance(address, &exp17(token_balance));
    }
    pub fn check_user_egld_balance_denominated(&self, address: &Address, token_balance: u128) {
        self.b_mock
            .check_egld_balance(address, &num_bigint::BigUint::from(token_balance));
    }

    pub fn debug_providers(&mut self) {
        self.b_mock
            .execute_query(&self.sc_wrapper, |sc| {
                let providers = sc.delegation_addresses_list();
                for provider in providers.iter() {
                    let delegation_contract_data = sc.delegation_contract_data(&provider).get();
                    println!("provider: {:?}", provider);
                    println!("delegation_contract_data: {:?}", delegation_contract_data);
                    let staked_amount = delegation_contract_data.total_staked_from_ls_contract;
                    println!("staked_amount: {:?}",staked_amount);
                    let unstaked_amount = delegation_contract_data.total_unstaked_from_ls_contract;
                    if unstaked_amount > 0 {
                        println!("unstaked_amount: {:?}", unstaked_amount);
                    }
                }
            })
            .assert_ok();
    }

    pub fn check_contract_storage(
        &mut self,
        ls_token_supply: u64,
        virtual_egld_reserve: u64,
        rewards_reserve: u64,
        withdrawn_egld: u64,
        pending_egld: u64,
        pending_ls_for_unstake: u64,
    ) {
        self.b_mock
            .execute_query(&self.sc_wrapper, |sc| {
                assert_eq!(
                    sc.ls_token_supply().get(),
                    to_managed_biguint(exp18(ls_token_supply))
                );
                assert_eq!(
                    sc.virtual_egld_reserve().get(),
                    to_managed_biguint(exp18(virtual_egld_reserve))
                );
                assert_eq!(
                    sc.rewards_reserve().get(),
                    to_managed_biguint(exp18(rewards_reserve))
                );
                assert_eq!(
                    sc.total_withdrawn_egld().get(),
                    to_managed_biguint(exp18(withdrawn_egld))
                );
                assert_eq!(
                    sc.pending_egld().get(),
                    to_managed_biguint(exp18(pending_egld))
                );
                assert_eq!(
                    sc.pending_egld_for_unstake().get(),
                    to_managed_biguint(exp18(pending_ls_for_unstake))
                );
            })
            .assert_ok();
    }

    pub fn check_pending_egld_exp17(&mut self, pending_egld: u64) {
        self.b_mock
            .execute_query(&self.sc_wrapper, |sc| {
                assert_eq!(
                    sc.pending_egld().get(),
                    to_managed_biguint(exp17(pending_egld))
                );
            })
            .assert_ok();
    }

    pub fn check_pending_ls_for_unstake(&mut self, pending_ls_for_unstake: u64) {
        self.b_mock
            .execute_query(&self.sc_wrapper, |sc| {
                assert_eq!(
                    sc.pending_egld_for_unstake().get(),
                    to_managed_biguint(exp18(pending_ls_for_unstake))
                );
            })
            .assert_ok();
    }
    pub fn check_pending_ls_for_unstake_exp17(&mut self, pending_ls_for_unstake: u64) {
        self.b_mock
            .execute_query(&self.sc_wrapper, |sc| {
                assert_eq!(
                    sc.pending_egld_for_unstake().get(),
                    to_managed_biguint(exp17(pending_ls_for_unstake))
                );
            })
            .assert_ok();
    }
    pub fn check_pending_ls_for_unstake_denominated(&mut self, pending_ls_for_unstake: u128) {
        self.b_mock
            .execute_query(&self.sc_wrapper, |sc| {
                assert_eq!(
                    sc.pending_egld_for_unstake().get(),
                    to_managed_biguint(num_bigint::BigUint::from(pending_ls_for_unstake))
                );
            })
            .assert_ok();
    }

    pub fn check_total_withdrawn_egld_denominated(&mut self, total_withdrawn_egld: u128) {
        self.b_mock
            .execute_query(&self.sc_wrapper, |sc| {
                assert_eq!(
                    sc.total_withdrawn_egld().get(),
                    to_managed_biguint(num_bigint::BigUint::from(total_withdrawn_egld))
                );
            })
            .assert_ok();
    }

    pub fn check_total_withdrawn_egld_exp17(&mut self, total_withdrawn_egld: u64) {
        self.b_mock
            .execute_query(&self.sc_wrapper, |sc| {
                assert_eq!(
                    sc.total_withdrawn_egld().get(),
                    to_managed_biguint(exp17(total_withdrawn_egld))
                );
            })
            .assert_ok();
    }

    pub fn check_contract_rewards_storage_denominated(&mut self, rewards_reserve: u128) {
        self.b_mock
            .execute_query(&self.sc_wrapper, |sc| {
                assert_eq!(
                    sc.rewards_reserve().get(),
                    to_managed_biguint(num_bigint::BigUint::from(rewards_reserve))
                );
            })
            .assert_ok();
    }

    pub fn check_delegation_contract_values(
        &mut self,
        delegation_contract: &Address,
        total_staked: u64,
        total_unstaked: u64,
    ) {
        self.b_mock
            .execute_query(&self.sc_wrapper, |sc| {
                assert_eq!(
                    sc.delegation_contract_data(&managed_address!(delegation_contract))
                        .get()
                        .total_staked_from_ls_contract,
                    to_managed_biguint(exp18(total_staked))
                );
                assert_eq!(
                    sc.delegation_contract_data(&managed_address!(delegation_contract))
                        .get()
                        .total_unstaked_from_ls_contract,
                    to_managed_biguint(exp18(total_unstaked))
                );
            })
            .assert_ok();
    }

    pub fn get_ls_value_for_position(&mut self, token_amount: u64) -> u128 {
        let mut ls_value = 0u64;
        self.b_mock
            .execute_query(&self.sc_wrapper, |sc| {
                let ls_value_biguint =
                    sc.get_ls_value_for_position(to_managed_biguint(exp18(token_amount)));
                println!("ls_value {:?}", ls_value_biguint);
                ls_value = ls_value_biguint.to_u64().unwrap();
            })
            .assert_ok();

        u128::from(ls_value)
    }

    pub fn get_pending_rewards(&mut self) -> u128 {
        let mut rewards_value_biguint = 0u64;
        self.b_mock
            .execute_query(&self.sc_wrapper, |sc| {
                rewards_value_biguint = sc.rewards_reserve().get().to_u64().unwrap();
            })
            .assert_ok();

        u128::from(rewards_value_biguint)
    }

    pub fn print_pending_egld(&mut self) {
        self.b_mock
            .execute_query(&self.sc_wrapper, |sc| {
                let pending_egld_value_biguint = sc.pending_egld().get().to_display();
                println!(
                    "pending_egld_value_biguint {:?}",
                    pending_egld_value_biguint
                );
            })
            .assert_ok();
    }

    pub fn check_delegation_contract_values_denominated(
        &mut self,
        delegation_contract: &Address,
        total_staked: u128,
    ) {
        self.b_mock
            .execute_query(&self.sc_wrapper, |sc| {
                assert_eq!(
                    sc.delegation_contract_data(&managed_address!(delegation_contract))
                        .get()
                        .total_staked_from_ls_contract,
                    to_managed_biguint(num_bigint::BigUint::from(total_staked))
                );
            })
            .assert_ok();
    }

    pub fn check_delegation_contract_unstaked_value_denominated(
        &mut self,
        delegation_contract: &Address,
        total_un_staked: u128,
    ) {
        self.b_mock
            .execute_query(&self.sc_wrapper, |sc| {
                assert_eq!(
                    sc.delegation_contract_data(&managed_address!(delegation_contract))
                        .get()
                        .total_unstaked_from_ls_contract,
                    to_managed_biguint(num_bigint::BigUint::from(total_un_staked))
                );
            })
            .assert_ok();
    }

    pub fn check_user_nft_balance_denominated(
        &self,
        address: &Address,
        token_id: &[u8],
        token_nonce: u64,
        token_balance: num_bigint::BigUint,
        expected_attributes: Option<&UnstakeTokenAttributes>,
    ) {
        self.b_mock.check_nft_balance::<UnstakeTokenAttributes>(
            address,
            token_id,
            token_nonce,
            &token_balance,
            expected_attributes,
        );
    }
}
