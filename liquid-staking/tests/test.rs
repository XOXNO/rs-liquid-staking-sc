mod contract_interactions;
mod contract_setup;
mod utils;

use contract_setup::*;

use liquid_staking::{errors::ERROR_NO_DELEGATION_CONTRACTS, structs::UnstakeTokenAttributes};
use multiversx_sc::{imports::OptionalValue, types::TestAddress};
use multiversx_sc_scenario::DebugApi;
use utils::{exp, exp18};

#[test]
fn init_test() {
    let _ = LiquidStakingContractSetup::new(400);
}

#[test]
fn liquid_staking_claim_rewards_and_withdraw_test() {
    DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(400);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&OWNER_ADDRESS.to_address(), 1000, 1000, 1500, 0, 0);

    let first_user = sc_setup.setup_new_user(TestAddress::new("first_user"), 100u64);

    sc_setup.add_liquidity(&first_user, exp18(100u64), OptionalValue::None);
    sc_setup.check_delegation_contract_values(&delegation_contract, exp18(0u64), exp18(0u64));
    sc_setup.check_contract_storage(100, 100, 0, 0, 100, 0);

    sc_setup.b_mock.current_block().block_round(14000u64);
    sc_setup.delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    sc_setup.check_delegation_contract_values(&delegation_contract, exp18(100u64), exp18(0u64));
    sc_setup.check_contract_storage(100, 100, 0, 0, 0, 0);

    sc_setup.b_mock.current_block().block_epoch(50u64);

    sc_setup.claim_rewards(&OWNER_ADDRESS.to_address());

    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, exp18(90u64));
    sc_setup.check_pending_ls_for_unstake_denominated(90000000000000000000u128);
    sc_setup.un_delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);
    sc_setup.check_pending_ls_for_unstake(0);

    sc_setup.check_delegation_contract_unstaked_value_denominated(
        &delegation_contract,
        90000000000000000000u128,
    );
    sc_setup.check_user_nft_balance_denominated(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        exp(90000000000000000000u128),
        Some(UnstakeTokenAttributes::new(50, 60)),
    );

    sc_setup.b_mock.current_block().block_epoch(60u64);

    sc_setup.withdraw_pending(&OWNER_ADDRESS.to_address(), &delegation_contract);

    sc_setup.withdraw(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        exp(90000000000000000000u128),
    );

    sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, exp18(10u64));
    sc_setup.check_user_egld_balance(&first_user, exp(90000000000000000000u128));
}

#[test]
fn liquid_staking_multiple_operations() {
    DebugApi::dummy();

    let mut sc_setup = LiquidStakingContractSetup::new(400);

    sc_setup.deploy_staking_contract(&OWNER_ADDRESS.to_address(), 1000, 1000, 1100, 15, 7_000u64);

    let delegation_contract2 = sc_setup.deploy_staking_contract(
        &OWNER_ADDRESS.to_address(),
        1000,
        1000,
        1100,
        30,
        6_300u64,
    );

    let delegation_contract3 = sc_setup.deploy_staking_contract(
        &OWNER_ADDRESS.to_address(),
        1000,
        1000,
        1100,
        50,
        6_600u64,
    );

    let delegation_contract4 =
        sc_setup.deploy_staking_contract(&OWNER_ADDRESS.to_address(), 1000, 1000, 0, 3, 11_000u64);

    let delegation_contract5 =
        sc_setup.deploy_staking_contract(&OWNER_ADDRESS.to_address(), 1000, 1000, 0, 3, 11_000u64);

    sc_setup.update_staking_contract_params(
        &OWNER_ADDRESS.to_address(),
        &delegation_contract5,
        1000,
        1000,
        3,
        11_000u64,
        false,
    );

    let first_user = sc_setup.setup_new_user(TestAddress::new("first_user"), 1000u64);
    let second_user = sc_setup.setup_new_user(TestAddress::new("second_user"), 1000u64);
    let third_user = sc_setup.setup_new_user(TestAddress::new("third_user"), 1000u64);
    sc_setup.add_liquidity(&first_user, exp18(100u64), OptionalValue::None);

    sc_setup.b_mock.current_block().block_round(14000u64);
    sc_setup.delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    sc_setup.add_liquidity(&first_user, exp18(200u64), OptionalValue::None);
    sc_setup.delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    sc_setup.add_liquidity(&second_user, exp18(500u64), OptionalValue::None);
    sc_setup.delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    sc_setup.update_staking_contract_params(
        &OWNER_ADDRESS.to_address(),
        &delegation_contract2,
        1080,
        0,
        6,
        13_000u64,
        true,
    );

    sc_setup.add_liquidity(&third_user, exp18(600u64), OptionalValue::None);
    sc_setup.delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    sc_setup.check_delegation_contract_values(
        &delegation_contract3,
        exp(274999422660963819554u128),
        exp18(0u64),
    );
    sc_setup
        .check_delegation_contract_values_denominated(&delegation_contract4, 424999794451021658340);

    sc_setup.update_staking_contract_params(
        &OWNER_ADDRESS.to_address(),
        &delegation_contract2,
        1080,
        0,
        3,
        8_000u64,
        true,
    );
    sc_setup.update_staking_contract_params(
        &OWNER_ADDRESS.to_address(),
        &delegation_contract3,
        1030,
        1100,
        3,
        9_000u64,
        true,
    );

    sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, exp18(300u64));
    sc_setup.check_user_balance(&second_user, LS_TOKEN_ID, exp18(500u64));
    sc_setup.check_user_balance(&third_user, LS_TOKEN_ID, exp18(600u64));

    sc_setup.b_mock.current_block().block_epoch(10u64);
    sc_setup.claim_rewards(&OWNER_ADDRESS.to_address());

    // sc_setup.check_user_egld_balance(&sc_setup.sc_wrapper.to_address(), exp(3849315068493150683));
}

#[test]
fn liquid_staking_multiple_withdraw_test() {
    DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(400);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&OWNER_ADDRESS.to_address(), 1000, 1000, 1500, 0, 0);

    let first_user = sc_setup.setup_new_user(TestAddress::new("first_user"), 100u64);
    let second_user = sc_setup.setup_new_user(TestAddress::new("second_user"), 100u64);
    let third_user = sc_setup.setup_new_user(TestAddress::new("third_user"), 100u64);

    sc_setup.add_liquidity(&first_user, exp18(50u64), OptionalValue::None);
    sc_setup.add_liquidity(&second_user, exp18(40u64), OptionalValue::None);
    sc_setup.add_liquidity(&third_user, exp18(40u64), OptionalValue::None);
    sc_setup.b_mock.current_block().block_round(14000u64);
    sc_setup.check_contract_storage(130, 130, 0, 0, 130, 0);
    sc_setup.delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);
    sc_setup.b_mock.current_block().block_epoch(50u64);
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, exp18(20u64));
    sc_setup.check_user_nft_balance_denominated(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        exp18(20),
        Some(UnstakeTokenAttributes {
            unbond_epoch: 60,
            unstake_epoch: 50,
        }),
    );
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, exp18(20u64));
    sc_setup.check_user_nft_balance_denominated(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        exp18(40),
        Some(UnstakeTokenAttributes {
            unbond_epoch: 60,
            unstake_epoch: 50,
        }),
    );
    sc_setup.remove_liquidity(&second_user, LS_TOKEN_ID, exp18(20u64));
    sc_setup.remove_liquidity(&third_user, LS_TOKEN_ID, exp18(20u64));

    sc_setup.check_contract_storage(50, 50, 0, 0, 0, 80);

    sc_setup.un_delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);
    sc_setup.b_mock.current_block().block_epoch(60u64);

    sc_setup.withdraw_pending(&OWNER_ADDRESS.to_address(), &delegation_contract);

    sc_setup.check_contract_storage(50, 50, 0, 80, 0, 0);

    sc_setup.withdraw(&first_user, UNSTAKE_TOKEN_ID, 1, exp18(20));
    sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, exp18(10u64));
    sc_setup.check_user_egld_balance(&first_user, exp18(70));
    sc_setup.check_user_balance(&second_user, LS_TOKEN_ID, exp18(20u64));
    sc_setup.check_user_egld_balance(&second_user, exp18(60));
    sc_setup.check_user_balance(&third_user, LS_TOKEN_ID, exp18(20u64));
    sc_setup.check_user_egld_balance(&third_user, exp18(60));
    sc_setup.check_contract_storage(50, 50, 0, 60, 0, 0); // 20 + 20 (second_user + third_user)
}

// #[test]
// fn full_flow_test() {
//     let _ = DebugApi::dummy();
//     let mut sc_setup = LiquidStakingContractSetup::new( 400);

//     let delegation_contract =
//         sc_setup.deploy_staking_contract(&OWNER_ADDRESS.to_address(), 0, 0, 0, 0, 0);

//     let first_user = sc_setup.setup_new_user(TestAddress::new("first_user"),50u64);
//     let second_user = sc_setup.setup_new_user(TestAddress::new("second_user"),20u64);
//     let third_user = sc_setup.setup_new_user(20u64);

//     sc_setup.check_user_egld_balance(&delegation_contract, 1);

//     sc_setup.add_liquidity(&first_user, 50u64);
//     sc_setup.add_liquidity(&second_user, 20u64);
//     sc_setup.add_liquidity(&third_user, 20u64);

//     sc_setup.b_mock.set_block_round(14000u64);

//     sc_setup.check_user_egld_balance(&sc_setup.sc_wrapper.address_ref(), 90);

//     sc_setup.delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

//     sc_setup.check_user_egld_balance(&delegation_contract, 91);
//     sc_setup.check_user_egld_balance(&sc_setup.sc_wrapper.address_ref(), 0);

//     sc_setup.b_mock.current_block().block_epoch(50u64);
//     sc_setup.claim_rewards(&sc_setup.owner_address.clone());

//     // let pending_rewards = sc_setup.get_pending_egld();

//     // From the 90 EGLD the mock SC send rewards to the liquid staking contract
//     sc_setup.check_user_egld_balance_denominated(&delegation_contract, 89753424657534246576u128);

//     // sc_setup.b_mock.set_egld_balance(
//     //     &delegation_contract,
//     //     &(sc_setup.b_mock.get_egld_balance(&delegation_contract)
//     //         + num_bigint::BigUint::from(pending_rewards)),
//     // );

//     sc_setup.check_user_egld_balance_denominated(&delegation_contract, 89753424657534246576u128);

//     // The liquid staking contract should have received the rewards
//     sc_setup.check_user_egld_balance_denominated(
//         &sc_setup.sc_wrapper.address_ref(),
//         1246575342465753424u128,
//     );
//     // The liquid staking contract should have delegated the rewards to the delegation contract
//     sc_setup.check_user_egld_balance_denominated(&sc_setup.sc_wrapper.address_ref(), 1246575342465753424u128);

//     // Rewards are sent back to the delegation contract - the protocol fee is deducted
//     sc_setup.check_user_egld_balance_denominated(&delegation_contract, 89753424657534246576u128);

//     sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 25u64);

//     sc_setup.check_user_nft_balance_denominated(
//         &first_user,
//         UNSTAKE_TOKEN_ID,
//         1,
//         num_bigint::BigUint::from(24132054794520547944u128),
//         Some(UnstakeTokenAttributes::new(50, 60)),
//     );
//     sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 25u64);
//     sc_setup.check_user_nft_balance_denominated(
//         &first_user,
//         UNSTAKE_TOKEN_ID,
//         1,
//         num_bigint::BigUint::from(49460821917808219177u128),
//         Some(UnstakeTokenAttributes::new(50, 60)),
//     );
//     sc_setup.remove_liquidity(&second_user, LS_TOKEN_ID, 20u64);
//     sc_setup.check_user_nft_balance_denominated(
//         &second_user,
//         UNSTAKE_TOKEN_ID,
//         1,
//         num_bigint::BigUint::from(20263013698630136986u128),
//         Some(UnstakeTokenAttributes::new(50, 60)),
//     );
//     sc_setup.remove_liquidity(&third_user, LS_TOKEN_ID, 20u64);
//     sc_setup.check_user_nft_balance_denominated(
//         &third_user,
//         UNSTAKE_TOKEN_ID,
//         1,
//         num_bigint::BigUint::from(20263013698630136987u128),
//         Some(UnstakeTokenAttributes::new(50, 60)),
//     );

//     sc_setup.un_delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

//     sc_setup.b_mock.current_block().block_epoch(60u64);

//     sc_setup.check_user_egld_balance_denominated(&sc_setup.sc_wrapper.address_ref(), 49863013698630136u128);

//     sc_setup.withdraw_pending(&OWNER_ADDRESS.to_address(), &delegation_contract);
//     // The unstaked EGLD is sent back to the main liquid staking contract
//     sc_setup.check_user_egld_balance_denominated(
//         &sc_setup.sc_wrapper.address_ref(),
//         49863013698630136u128,
//     );

//     sc_setup.check_delegation_contract_values_denominated(
//         &delegation_contract,
//         0,
//     );
//     return;
//     sc_setup.check_total_withdrawn_egld_denominated(91183561643835616438u128);

//     sc_setup.check_user_balance(&sc_setup.sc_wrapper.address_ref(), LS_TOKEN_ID, 0u64);

//     sc_setup.check_user_egld_balance_denominated(
//         &sc_setup.sc_wrapper.address_ref(),
//         91183561643835616438u128,
//     );

//     sc_setup.check_user_nft_balance_denominated(
//         &first_user,
//         UNSTAKE_TOKEN_ID,
//         1,
//         num_bigint::BigUint::from(50657534246575342465u128),
//         Some(UnstakeTokenAttributes::new(50, 60)),
//     );
//     sc_setup.withdraw(
//         &first_user,
//         UNSTAKE_TOKEN_ID,
//         1,
//         num_bigint::BigUint::from(50657534246575342465u128),
//     );
//     sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, 0u64);
//     sc_setup.check_user_egld_balance_denominated(&first_user, 50657534246575342465u128);
//     sc_setup.check_user_nft_balance_denominated(
//         &second_user,
//         UNSTAKE_TOKEN_ID,
//         1,
//         num_bigint::BigUint::from(20263013698630136986u128),
//         Some(UnstakeTokenAttributes::new(50, 60)),
//     );
//     sc_setup.withdraw(
//         &second_user,
//         UNSTAKE_TOKEN_ID,
//         1,
//         num_bigint::BigUint::from(20263013698630136986u128),
//     );
//     sc_setup.check_user_balance(&second_user, LS_TOKEN_ID, 0u64);
//     sc_setup.check_user_egld_balance_denominated(&second_user, 20263013698630136986u128);

//     sc_setup.withdraw(
//         &third_user,
//         UNSTAKE_TOKEN_ID,
//         1,
//         num_bigint::BigUint::from(20263013698630136986u128),
//     );
//     sc_setup.check_user_balance(&third_user, LS_TOKEN_ID, 0u64);
//     sc_setup.check_user_egld_balance_denominated(&third_user, 20263013698630136986u128);

//     // The main delegation contract should have 0 EGLD left as the initial deposit (or a small amount due to rounding)
//     sc_setup.check_user_egld_balance_denominated(&sc_setup.sc_wrapper.address_ref(), 1);
// }

// #[test]
// fn claim_rewards_multiple_times_test() {
//     let _ = DebugApi::dummy();
//     let mut sc_setup = LiquidStakingContractSetup::new(400);

//     sc_setup.deploy_staking_contract(&OWNER_ADDRESS.to_address(), 1000, 1000, 1500, 0, 0);

//     let first_user = sc_setup.setup_new_user(TestAddress::new("first_user"), 100u64);

//     sc_setup.add_liquidity(&first_user, exp18(100u64), OptionalValue::None);
//     sc_setup.b_mock.current_block().block_round(14000u64);
//     sc_setup.delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);
//     sc_setup.b_mock.current_block().block_epoch(50u64);
//     sc_setup.claim_rewards(&OWNER_ADDRESS.to_address());
//     // sc_setup.delegate_rewards(&sc_setup.owner_address.clone());
//     // let pending_rewards = sc_setup.get_pending_rewards();
//     // assert_eq!(pending_rewards, 0, "pending_rewards should be 0");
//     sc_setup.b_mock.current_block().block_epoch(100u64);
//     sc_setup.claim_rewards(&OWNER_ADDRESS.to_address());
//     // let pending_rewards = sc_setup.get_pending_rewards();
//     // assert_eq!(
//     //     pending_rewards, 1401756427097016325u128,
//     //     "pending_rewards should be 1401756427097016325"
//     // );
//     // sc_setup.delegate_rewards(&sc_setup.owner_address.clone());
// }

#[test]
fn add_liquidity_no_valid_delegation_contract_error_test() {
    DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(400);

    let first_user = sc_setup.setup_new_user(TestAddress::new("first_user"), 100u64);

    sc_setup.add_liquidity(&first_user, exp18(100u64), OptionalValue::None);
    sc_setup.b_mock.current_block().block_round(14000u64);
    sc_setup.delegate_pending_error(
        &OWNER_ADDRESS.to_address(),
        OptionalValue::None,
        ERROR_NO_DELEGATION_CONTRACTS,
    );
}
