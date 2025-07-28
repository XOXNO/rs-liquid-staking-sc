use delegation_manager_mock::proxy_delegation::DelegationMockProxy;
use multiversx_sc::types::{
    EsdtLocalRole, ManagedAddress, ReturnsNewManagedAddress, TestAddress, TestTokenIdentifier,
};

use multiversx_sc_scenario::{
    api::StaticApi, imports::{ExecutorConfig, MxscPath}, managed_biguint, rust_biguint, ScenarioTxRun,
    ScenarioTxWhitebox, ScenarioWorld,
};

use liquid_staking::config::ConfigModule;
use liquid_staking::*;
use proxy::{proxy_accumulator::AccumulatorProxy, proxy_liquid_staking};
use storage::StorageModule;
use structs::ScoringConfig;

use crate::exp18;

extern crate accumulator;
extern crate delegation_manager_mock;
extern crate delegation_mock;
extern crate liquid_staking;

pub const XOXNO_TOKEN: TestTokenIdentifier = TestTokenIdentifier::new("XOXNO-abcdef");
pub const LXOXNO_TOKEN: TestTokenIdentifier = TestTokenIdentifier::new("LXOXNO-abcdef");
pub const LS_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("LSTOKEN-123456");
pub const UNSTAKE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("UNSTAKE-123456");

pub const LIQUID_STAKING_DEPLOY_CODE: MxscPath =
    MxscPath::new("liquid-staking/output/liquid-staking.mxsc.json");
pub const DELEGATION_DEPLOY_CODE: MxscPath =
    MxscPath::new("liquid-staking/tests/delegation-mock.mxsc.json");
pub const DELEGATION_MANAGER_DEPLOY_CODE: MxscPath =
    MxscPath::new("liquid-staking/tests/delegation-manager-mock.mxsc.json");
pub const ACCUMULATION_DEPLOY_CODE: MxscPath =
    MxscPath::new("liquid-staking/tests/accumulator.mxsc.json");

pub static ESDT_ROLES: &[EsdtLocalRole] = &[
    EsdtLocalRole::Mint,
    EsdtLocalRole::Burn,
];

pub static SFT_ROLES: &[EsdtLocalRole] = &[
    EsdtLocalRole::NftCreate,
    EsdtLocalRole::NftAddQuantity,
    EsdtLocalRole::NftBurn,
];

pub const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
pub const ACCUMULATOR_ADDRESS: TestAddress = TestAddress::new("accumulator");
pub const ASH_SWAP_ADDRESS: TestAddress = TestAddress::new("ashswap");

pub struct LiquidStakingContractSetup {
    pub b_mock: ScenarioWorld,
    pub sc_wrapper: ManagedAddress<StaticApi>,
}

fn setup_delegation_manager(world: &mut ScenarioWorld) -> ManagedAddress<StaticApi> {
    let sc = world
        .tx()
        .from(OWNER_ADDRESS)
        .typed(DelegationMockProxy)
        .init()
        .code(DELEGATION_MANAGER_DEPLOY_CODE)
        .returns(ReturnsNewManagedAddress)
        .run();

    sc
}

fn setup_accumulation(world: &mut ScenarioWorld) -> ManagedAddress<StaticApi> {
    world
        .tx()
        .from(OWNER_ADDRESS)
        .typed(AccumulatorProxy)
        .init(
            ACCUMULATOR_ADDRESS,
            rust_biguint!(1000),
            rust_biguint!(3000),
            XOXNO_TOKEN,
            LXOXNO_TOKEN,
            ASH_SWAP_ADDRESS,
        )
        .code(ACCUMULATION_DEPLOY_CODE)
        .returns(ReturnsNewManagedAddress)
        .run()
}

fn setup_liquid_staking_sc(world: &mut ScenarioWorld, fees: u64) -> ManagedAddress<StaticApi> {
    let accumulator = setup_accumulation(world);
    let sc = world
        .tx()
        .from(OWNER_ADDRESS)
        .typed(proxy_liquid_staking::LiquidStakingProxy)
        .init(
            accumulator,
            managed_biguint!(fees),
            managed_biguint!(25),
            100usize,
            10u64,
        )
        .code(LIQUID_STAKING_DEPLOY_CODE)
        .returns(ReturnsNewManagedAddress)
        .run();

    world
        .tx()
        .from(OWNER_ADDRESS)
        .to(&sc)
        .whitebox(liquid_staking::contract_obj, |sc| {
            sc.unstake_token()
                .set_token_id(UNSTAKE_TOKEN_ID.to_token_identifier());
            sc.ls_token()
                .set_token_id(LS_TOKEN_ID.to_token_identifier());
            sc.set_scoring_config(ScoringConfig::default());
            sc.set_state_active();
        });

    world.set_esdt_local_roles(&sc, LS_TOKEN_ID.as_bytes(), ESDT_ROLES);
    world.set_esdt_local_roles(&sc, UNSTAKE_TOKEN_ID.as_bytes(), SFT_ROLES);

    sc
}

impl LiquidStakingContractSetup {
    pub fn new(fees: u64) -> Self {
        let mut world = world();

        setup_delegation_manager(&mut world);
        let template_address_liquidity_pool = setup_liquid_staking_sc(&mut world, fees);
        world.current_block().block_round(14000u64);

        LiquidStakingContractSetup {
            b_mock: world,
            sc_wrapper: template_address_liquidity_pool,
        }
    }
}

pub fn world() -> ScenarioWorld {    
    let mut blockchain =
    ScenarioWorld::new().executor_config(ExecutorConfig::compiled_tests_if_else(
        ExecutorConfig::Experimental.then(ExecutorConfig::Experimental),
        ExecutorConfig::Debugger,
    ));

    blockchain.register_contract(LIQUID_STAKING_DEPLOY_CODE, liquid_staking::ContractBuilder);
    blockchain.register_contract(ACCUMULATION_DEPLOY_CODE, accumulator::ContractBuilder);
    blockchain.register_contract(DELEGATION_DEPLOY_CODE, delegation_mock::ContractBuilder);
    blockchain.register_contract(
        DELEGATION_MANAGER_DEPLOY_CODE,
        delegation_manager_mock::ContractBuilder,
    );

    setup_owner(&mut blockchain);
    blockchain
}

pub fn setup_owner(world: &mut ScenarioWorld) {
    world.account(OWNER_ADDRESS).nonce(1).balance(exp18(100));
}
