use crate::contract_setup::LiquidStakingContractSetup;
use crate::{utils::*, DELEGATION_DEPLOY_CODE, OWNER_ADDRESS};
use delegation_manager_mock::proxy_delegation::{self, DelegationMockProxy};
use liquid_staking::proxy::proxy_liquid_staking;
use liquid_staking::structs::UnstakeTokenAttributes;
use multiversx_sc::types::{
    BigUint, ManagedVec, ReturnsNewManagedAddress, ReturnsResult, TestAddress, TestTokenIdentifier,
};
use multiversx_sc::{
    imports::OptionalValue,
    types::{Address, ManagedAddress},
};
use multiversx_sc_scenario::api::StaticApi;
use multiversx_sc_scenario::{ExpectMessage, ScenarioTxRun};

impl LiquidStakingContractSetup {
    pub fn setup_new_user(&mut self, user: TestAddress, egld_token_amount: u64) -> Address {
        self.b_mock
            .account(user)
            .nonce(0)
            .balance(exp18(egld_token_amount));

        user.to_address()
    }

    pub fn deploy_staking_contract(
        &mut self,
        owner_address: &Address,
        egld_balance: u64,
        total_staked: u64,
        delegation_contract_cap: u64,
        nr_nodes: u64,
        apy: u64,
    ) -> Address {
        let rust_one_egld = exp18(1);
        let egld_balance_biguint = &exp18(egld_balance);
        let total_staked_biguint = exp18(total_staked);
        let delegation_contract_cap_biguint = exp18(delegation_contract_cap);

        self.b_mock
            .set_egld_balance(owner_address, &(egld_balance_biguint + &rust_one_egld));

        let delegation_contract = self
            .b_mock
            .tx()
            .from(OWNER_ADDRESS)
            .typed(DelegationMockProxy)
            .init()
            .code(DELEGATION_DEPLOY_CODE)
            .returns(ReturnsNewManagedAddress)
            .run();

        self.b_mock
            .tx()
            .from(owner_address)
            .to(&delegation_contract)
            .typed(proxy_delegation::DelegationMockProxy)
            .deposit_egld()
            .egld(egld_balance_biguint)
            .run();

        self.b_mock
            .tx()
            .from(owner_address)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .whitelist_delegation_contract(
                &delegation_contract,
                owner_address,
                total_staked_biguint,
                delegation_contract_cap_biguint,
                nr_nodes,
                apy,
            )
            .egld(rust_one_egld)
            .run();

        delegation_contract.to_address()
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
        let total_staked_biguint = exp18(total_staked);
        let delegation_contract_cap_biguint = exp18(delegation_contract_cap);

        self.b_mock
            .tx()
            .from(owner_address)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .change_delegation_contract_params(
                contract_address,
                total_staked_biguint,
                delegation_contract_cap_biguint,
                nr_nodes,
                apy,
                is_eligible,
            )
            .run()
    }

    pub fn set_inactive_state(&mut self, caller: &Address) {
        self.b_mock
            .tx()
            .from(caller)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .set_state_inactive()
            .run()
    }

    pub fn add_liquidity(
        &mut self,
        caller: &Address,
        payment_amount: BigUint<StaticApi>,
        to: OptionalValue<ManagedAddress<StaticApi>>,
    ) {
        self.b_mock
            .tx()
            .from(caller)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .delegate(to)
            .egld(payment_amount)
            .run();
    }

    pub fn add_liquidity_error(
        &mut self,
        caller: &Address,
        payment_amount: BigUint<StaticApi>,
        error: &[u8],
        to: OptionalValue<ManagedAddress<StaticApi>>,
    ) {
        self.b_mock
            .tx()
            .from(caller)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .delegate(to)
            .egld(&payment_amount)
            .returns(ExpectMessage(core::str::from_utf8(error).unwrap()))
            .run();
    }

    pub fn add_liquidity_provider(&mut self, providers: Address) {
        self.b_mock
            .tx()
            .from(OWNER_ADDRESS)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .add_liquidity_provider(providers)
            .run();
    }

    pub fn remove_liquidity_provider(&mut self, provider: Address) {
        self.b_mock
            .tx()
            .from(OWNER_ADDRESS)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .remove_liquidity_provider(provider)
            .run();
    }

    pub fn remove_liquidity(
        &mut self,
        caller: &Address,
        payment_token: TestTokenIdentifier,
        payment_amount: BigUint<StaticApi>,
    ) {
        self.b_mock
            .tx()
            .from(caller)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .un_delegate()
            .single_esdt(&payment_token.to_token_identifier(), 0, &payment_amount)
            .run();
    }

    pub fn remove_liquidity_error(
        &mut self,
        caller: &Address,
        payment_token: TestTokenIdentifier,
        payment_amount: BigUint<StaticApi>,
        error: &[u8],
    ) {
        self.b_mock
            .tx()
            .from(caller)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .un_delegate()
            .single_esdt(&payment_token.to_token_identifier(), 0, &payment_amount)
            .returns(ExpectMessage(core::str::from_utf8(error).unwrap()))
            .run();
    }

    pub fn claim_rewards(&mut self, caller: &Address) {
        self.b_mock
            .tx()
            .from(caller)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .claim_rewards()
            .run();
    }

    pub fn claim_rewards_error(&mut self, caller: &Address, error: &[u8]) {
        self.b_mock
            .tx()
            .from(caller)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .claim_rewards()
            .returns(ExpectMessage(core::str::from_utf8(error).unwrap()))
            .run();
    }

    pub fn delegate_pending(
        &mut self,
        caller: &Address,
        amount: OptionalValue<BigUint<StaticApi>>,
    ) {
        self.b_mock
            .tx()
            .from(caller)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .delegate_pending(amount)
            .run();
    }

    pub fn delegate_pending_error(
        &mut self,
        caller: &Address,
        amount: OptionalValue<BigUint<StaticApi>>,
        error: &[u8],
    ) {
        self.b_mock
            .tx()
            .from(caller)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .delegate_pending(amount)
            .returns(ExpectMessage(core::str::from_utf8(error).unwrap()))
            .run();
    }

    pub fn un_delegate_pending(
        &mut self,
        caller: &Address,
        amount: OptionalValue<BigUint<StaticApi>>,
    ) {
        self.b_mock
            .tx()
            .from(caller)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .un_delegate_pending(
                amount,
                OptionalValue::<ManagedVec<StaticApi, ManagedAddress<StaticApi>>>::None,
            )
            .run();
    }

    pub fn un_delegate_pending_provider(
        &mut self,
        caller: &Address,
        amount: OptionalValue<BigUint<StaticApi>>,
        provider: ManagedAddress<StaticApi>,
    ) {
        self.b_mock
            .tx()
            .from(caller)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .un_delegate_pending(
                amount,
                OptionalValue::Some(ManagedVec::from_iter(vec![provider])),
            )
            .run();
    }

    pub fn un_delegate_pending_error(
        &mut self,
        caller: &Address,
        amount: OptionalValue<BigUint<StaticApi>>,
        error: &[u8],
    ) {
        self.b_mock
            .tx()
            .from(caller)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .un_delegate_pending(
                amount,
                OptionalValue::<ManagedVec<StaticApi, ManagedAddress<StaticApi>>>::None,
            )
            .returns(ExpectMessage(core::str::from_utf8(error).unwrap()))
            .run();
    }

    pub fn withdraw_pending(&mut self, caller: &Address, contracts: &Address) {
        self.b_mock
            .tx()
            .from(caller)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .withdraw_pending(contracts)
            .run();
    }

    pub fn withdraw_pending_error(&mut self, caller: &Address, contracts: &Address, error: &[u8]) {
        self.b_mock
            .tx()
            .from(caller)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .withdraw_pending(contracts)
            .returns(ExpectMessage(core::str::from_utf8(error).unwrap()))
            .run();
    }

    pub fn withdraw(
        &mut self,
        caller: &Address,
        payment_token: TestTokenIdentifier,
        token_nonce: u64,
        amount: BigUint<StaticApi>,
    ) {
        self.b_mock
            .tx()
            .from(caller)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .withdraw()
            .single_esdt(&payment_token.to_token_identifier(), token_nonce, &amount)
            .run();
    }

    pub fn withdraw_error(
        &mut self,
        caller: &Address,
        payment_token: TestTokenIdentifier,
        token_nonce: u64,
        amount: BigUint<StaticApi>,
        error: &[u8],
    ) {
        self.b_mock
            .tx()
            .from(caller)
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .withdraw()
            .single_esdt(&payment_token.to_token_identifier(), token_nonce, &amount)
            .returns(ExpectMessage(core::str::from_utf8(error).unwrap()))
            .run();
    }

    pub fn check_user_balance(
        &mut self,
        address: &Address,
        token_id: TestTokenIdentifier,
        token_balance: BigUint<StaticApi>,
    ) {
        self.b_mock
            .check_account(address)
            .esdt_balance(token_id, token_balance);
    }

    pub fn check_user_egld_balance(
        &mut self,
        address: &Address,
        token_balance: BigUint<StaticApi>,
    ) {
        self.b_mock.check_account(address).balance(token_balance);
    }

    pub fn debug_providers(&mut self) {
        let providers = self
            .b_mock
            .query()
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .delegation_addresses_list()
            .returns(ReturnsResult)
            .run();
        for provider in providers {
            let delegation_contract_data = self
                .b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .delegation_contract_data(&provider)
                .returns(ReturnsResult)
                .run();
            println!("provider: {:?}", provider);
            println!("delegation_contract_data: {:?}", delegation_contract_data);
            let staked_amount = delegation_contract_data.total_staked_from_ls_contract;
            println!("staked_amount: {:?}", staked_amount);
            let unstaked_amount = delegation_contract_data.total_unstaked_from_ls_contract;
            if unstaked_amount > 0 {
                println!("unstaked_amount: {:?}", unstaked_amount);
            }
        }
    }

    pub fn check_contract_storage(
        &mut self,
        ls_token_supply: u64,
        virtual_egld_reserve: u64,
        fees_reserve: u64,
        withdrawn_egld: u64,
        pending_egld: u64,
        pending_ls_for_unstake: u64,
    ) {
        assert_eq!(
            self.b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .ls_token_supply()
                .returns(ReturnsResult)
                .run(),
            exp18(ls_token_supply)
        );
        assert_eq!(
            self.b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .virtual_egld_reserve()
                .returns(ReturnsResult)
                .run(),
            exp18(virtual_egld_reserve)
        );

        assert_eq!(
            self.b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .fees_reserve()
                .returns(ReturnsResult)
                .run(),
            exp18(fees_reserve)
        );

        assert_eq!(
            self.b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .total_withdrawn_egld()
                .returns(ReturnsResult)
                .run(),
            exp18(withdrawn_egld)
        );

        assert_eq!(
            self.b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .pending_egld()
                .returns(ReturnsResult)
                .run(),
            exp18(pending_egld)
        );

        assert_eq!(
            self.b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .pending_egld_for_unstake()
                .returns(ReturnsResult)
                .run(),
            exp18(pending_ls_for_unstake)
        );
    }

    pub fn check_pending_egld_exp17(&mut self, pending_egld: u64) {
        assert_eq!(
            self.b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .pending_egld()
                .returns(ReturnsResult)
                .run(),
            exp17(pending_egld)
        );
    }

    pub fn check_pending_ls_for_unstake(&mut self, pending_ls_for_unstake: u64) {
        assert_eq!(
            self.b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .pending_egld_for_unstake()
                .returns(ReturnsResult)
                .run(),
            exp18(pending_ls_for_unstake)
        );
    }

    pub fn check_pending_ls_for_unstake_exp17(&mut self, pending_ls_for_unstake: u64) {
        assert_eq!(
            self.b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .pending_egld_for_unstake()
                .returns(ReturnsResult)
                .run(),
            exp17(pending_ls_for_unstake)
        );
    }

    pub fn check_pending_ls_for_unstake_denominated(&mut self, pending_ls_for_unstake: u128) {
        assert_eq!(
            self.b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .pending_egld_for_unstake()
                .returns(ReturnsResult)
                .run(),
            exp(pending_ls_for_unstake)
        );
    }

    pub fn check_total_withdrawn_egld_denominated(&mut self, total_withdrawn_egld: u128) {
        assert_eq!(
            self.b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .total_withdrawn_egld()
                .returns(ReturnsResult)
                .run(),
            exp(total_withdrawn_egld)
        );
    }

    pub fn check_total_withdrawn_egld_exp17(&mut self, total_withdrawn_egld: u64) {
        assert_eq!(
            self.b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .total_withdrawn_egld()
                .returns(ReturnsResult)
                .run(),
            exp17(total_withdrawn_egld)
        );
    }

    pub fn check_contract_fees_storage_denominated(&mut self, fees_reserve: u128) {
        assert_eq!(
            self.b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .fees_reserve()
                .returns(ReturnsResult)
                .run(),
            exp(fees_reserve)
        );
    }

    pub fn check_delegation_contract_values(
        &mut self,
        delegation_contract: &Address,
        total_staked: BigUint<StaticApi>,
        total_unstaked: BigUint<StaticApi>,
    ) {
        assert_eq!(
            self.b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .delegation_contract_data(delegation_contract)
                .returns(ReturnsResult)
                .run()
                .total_staked_from_ls_contract,
            total_staked
        );
        assert_eq!(
            self.b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .delegation_contract_data(delegation_contract)
                .returns(ReturnsResult)
                .run()
                .total_unstaked_from_ls_contract,
            total_unstaked
        );
    }

    pub fn get_ls_value_for_position(&mut self, token_amount: u64) -> u128 {
        let ls_value_biguint = self
            .b_mock
            .query()
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .get_ls_value_for_position(exp18(token_amount))
            .returns(ReturnsResult)
            .run();
        println!("ls_value {:?}", ls_value_biguint);

        u128::from(ls_value_biguint.to_u64().unwrap())
    }

    pub fn get_fees_reserve(&mut self) -> u128 {
        let fees_value_biguint = self
            .b_mock
            .query()
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .fees_reserve()
            .returns(ReturnsResult)
            .run();

        u128::from(fees_value_biguint.to_u64().unwrap())
    }

    pub fn print_pending_egld(&mut self) {
        let pending_egld_value_biguint = self
            .b_mock
            .query()
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .pending_egld()
            .returns(ReturnsResult)
            .run();
        println!(
            "pending_egld_value_biguint {:?}",
            pending_egld_value_biguint.to_display()
        );
    }

    pub fn check_delegation_contract_values_denominated(
        &mut self,
        delegation_contract: &Address,
        total_staked: u128,
    ) {
        assert_eq!(
            self.b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .delegation_contract_data(delegation_contract)
                .returns(ReturnsResult)
                .run()
                .total_staked_from_ls_contract,
            exp(total_staked)
        );
    }

    pub fn get_total_staked_from_ls_contract(
        &mut self,
        delegation_contract: &Address,
    ) -> BigUint<StaticApi> {
        self.b_mock
            .query()
            .to(&self.sc_wrapper)
            .typed(proxy_liquid_staking::LiquidStakingProxy)
            .delegation_contract_data(delegation_contract)
            .returns(ReturnsResult)
            .run()
            .total_staked_from_ls_contract
    }

    pub fn check_delegation_contract_unstaked_value_denominated(
        &mut self,
        delegation_contract: &Address,
        total_un_staked: u128,
    ) {
        assert_eq!(
            self.b_mock
                .query()
                .to(&self.sc_wrapper)
                .typed(proxy_liquid_staking::LiquidStakingProxy)
                .delegation_contract_data(delegation_contract)
                .returns(ReturnsResult)
                .run()
                .total_unstaked_from_ls_contract,
            exp(total_un_staked)
        );
    }

    pub fn check_user_nft_balance_denominated(
        &mut self,
        address: &Address,
        token_id: TestTokenIdentifier,
        token_nonce: u64,
        token_balance: BigUint<StaticApi>,
        expected_attributes: Option<UnstakeTokenAttributes>,
    ) {
        if expected_attributes.is_some() {
            self.b_mock
                .check_account(address)
                .esdt_nft_balance_and_attributes(
                    token_id,
                    token_nonce,
                    token_balance,
                    expected_attributes.unwrap(),
                );
        } else {
            self.b_mock
                .check_account(address)
                .esdt_nft_balance_and_attributes(
                    token_id,
                    token_nonce,
                    token_balance,
                    UnstakeTokenAttributes {
                        unbond_epoch: 0,
                        unstake_epoch: 0,
                    },
                );
        }
    }
}
