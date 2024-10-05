use multiversx_sc::{
    storage::mappers::StorageTokenWrapper,
    types::{Address, EsdtLocalRole},
};

use multiversx_sc_scenario::{
    imports::{BlockchainStateWrapper, ContractObjWrapper},
    managed_address, managed_biguint, managed_token_id, rust_biguint, DebugApi,
};

use liquid_staking::config::ConfigModule;
use liquid_staking::*;

extern crate accumulator;

pub const LIQUID_STAKING_WASM_PATH: &str = "liquid-staking/output/liquid-staking.wasm";
pub const ACCUMULATOR_WASM_PATH: &str = "liquid-staking/tests/accumulator.wasm";

pub static LS_TOKEN_ID: &[u8] = b"LSTOKEN-123456";
pub static UNSTAKE_TOKEN_ID: &[u8] = b"UNSTAKE-123456";

pub static ESDT_ROLES: &[EsdtLocalRole] = &[
    EsdtLocalRole::Mint,
    EsdtLocalRole::Burn,
    EsdtLocalRole::Transfer,
];

pub static SFT_ROLES: &[EsdtLocalRole] = &[
    EsdtLocalRole::NftCreate,
    EsdtLocalRole::NftAddQuantity,
    EsdtLocalRole::NftBurn,
];

pub struct LiquidStakingContractSetup<LiquidStakingContractObjBuilder>
where
    LiquidStakingContractObjBuilder: 'static + Copy + Fn() -> liquid_staking::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub owner_address: Address,
    pub sc_wrapper:
        ContractObjWrapper<liquid_staking::ContractObj<DebugApi>, LiquidStakingContractObjBuilder>,
}

impl<LiquidStakingContractObjBuilder> LiquidStakingContractSetup<LiquidStakingContractObjBuilder>
where
    LiquidStakingContractObjBuilder: 'static + Copy + Fn() -> liquid_staking::ContractObj<DebugApi>,
{
    pub fn new(sc_builder: LiquidStakingContractObjBuilder) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let mut b_mock = BlockchainStateWrapper::new();
        let owner_address = b_mock.create_user_account(&rust_zero);

        let sc_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(&owner_address),
            sc_builder,
            LIQUID_STAKING_WASM_PATH,
        );
        let accumulator_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(&owner_address),
            accumulator::contract_obj,
            ACCUMULATOR_WASM_PATH,
        );

        b_mock
            .execute_tx(&owner_address, &sc_wrapper, &rust_zero, |sc| {
                sc.init(
                    managed_address!(accumulator_wrapper.address_ref()),
                    managed_biguint!(400),
                    14400,
                    1400,
                );
            })
            .assert_ok();

        b_mock
            .execute_tx(&owner_address, &sc_wrapper, &rust_zero, |sc| {
                sc.ls_token().set_token_id(managed_token_id!(LS_TOKEN_ID));
            })
            .assert_ok();

        b_mock
            .execute_tx(&owner_address, &sc_wrapper, &rust_zero, |sc| {
                sc.unstake_token()
                    .set_token_id(managed_token_id!(UNSTAKE_TOKEN_ID));
            })
            .assert_ok();

        b_mock.set_esdt_local_roles(sc_wrapper.address_ref(), LS_TOKEN_ID, ESDT_ROLES);
        b_mock.set_esdt_local_roles(sc_wrapper.address_ref(), UNSTAKE_TOKEN_ID, SFT_ROLES);

        b_mock
            .execute_tx(&owner_address, &sc_wrapper, &rust_zero, |sc| {
                sc.set_state_active();
            })
            .assert_ok();

        LiquidStakingContractSetup {
            b_mock,
            owner_address,
            sc_wrapper,
        }
    }
}
