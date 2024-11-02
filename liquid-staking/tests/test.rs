mod contract_interactions;
mod contract_setup;
mod utils;

use contract_setup::*;

use liquid_staking::{
    errors::{
        ERROR_BAD_DELEGATION_ADDRESS, ERROR_INSUFFICIENT_REWARDS, ERROR_NOT_ACTIVE,
        ERROR_NO_DELEGATION_CONTRACTS,
    },
    structs::UnstakeTokenAttributes,
};
use multiversx_sc::imports::OptionalValue;
use multiversx_sc_scenario::{
    num_bigint::{self},
    DebugApi,
};
use utils::exp18;

#[test]
fn init_test() {
    let _ = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);
}

#[test]
fn liquid_staking_claim_rewards_and_withdraw_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let first_user = sc_setup.setup_new_user(100u64);

    sc_setup.add_liquidity(&first_user, 100u64);
    sc_setup.check_delegation_contract_values(&delegation_contract, 0u64, 0u64);
    sc_setup.check_contract_storage(100, 100, 0, 0, 100, 0);

    sc_setup.b_mock.set_block_round(14000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    sc_setup.check_delegation_contract_values(&delegation_contract, 100u64, 0u64);
    sc_setup.check_contract_storage(100, 100, 0, 0, 0, 0);

    sc_setup.b_mock.set_block_epoch(50u64);

    sc_setup.claim_rewards(&sc_setup.owner_address.clone());

    sc_setup.check_contract_rewards_storage_denominated(1369863013698630136);

    sc_setup.delegate_rewards(&sc_setup.owner_address.clone());

    sc_setup.check_contract_rewards_storage_denominated(0);

    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 90u64);
    sc_setup.check_pending_ls_for_unstake_denominated(91183561643835616437u128);
    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.check_pending_ls_for_unstake(0);

    sc_setup.check_delegation_contract_unstaked_value_denominated(
        &delegation_contract,
        91183561643835616437u128,
    );

    sc_setup.check_user_nft_balance_denominated(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        num_bigint::BigUint::from(91183561643835616437u128),
        Some(&UnstakeTokenAttributes::new(50, 60)),
    );

    sc_setup.b_mock.set_block_epoch(60u64);

    sc_setup.withdraw_pending(&sc_setup.owner_address.clone(), &delegation_contract);

    sc_setup.withdraw(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        num_bigint::BigUint::from(91183561643835616437u128),
    );

    sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, 10u64);
    sc_setup.check_user_egld_balance_denominated(&first_user, 91183561643835616437u128);
}

#[test]
fn liquid_staking_multiple_operations() {
    let _ = DebugApi::dummy();

    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    let delegation_contract1 = sc_setup.deploy_staking_contract(
        &sc_setup.owner_address.clone(),
        1000,
        1000,
        1100,
        15,
        7_000u64,
    );

    let delegation_contract2 = sc_setup.deploy_staking_contract(
        &sc_setup.owner_address.clone(),
        1000,
        1000,
        1100,
        30,
        6_300u64,
    );

    let delegation_contract3 = sc_setup.deploy_staking_contract(
        &sc_setup.owner_address.clone(),
        1000,
        1000,
        1100,
        50,
        6_600u64,
    );

    let delegation_contract4 = sc_setup.deploy_staking_contract(
        &sc_setup.owner_address.clone(),
        1000,
        1000,
        0,
        3,
        11_000u64,
    );

    let first_user = sc_setup.setup_new_user(1000u64);
    let second_user = sc_setup.setup_new_user(1000u64);
    let third_user = sc_setup.setup_new_user(1000u64);
    sc_setup.add_liquidity(&first_user, 100u64);

    sc_setup.b_mock.set_block_round(14000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    sc_setup.check_delegation_contract_values(&delegation_contract1, 25u64, 0u64);
    sc_setup.check_delegation_contract_values(&delegation_contract2, 25u64, 0u64);
    sc_setup.check_delegation_contract_values(&delegation_contract3, 25u64, 0u64);
    sc_setup.check_delegation_contract_values(&delegation_contract4, 25u64, 0u64);

    sc_setup.add_liquidity(&first_user, 200u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    // sc_setup.check_delegation_contract_values(&delegation_contract1, 75u64, 0u64);
    // sc_setup.check_delegation_contract_values(&delegation_contract2, 75u64, 0u64);
    // sc_setup.check_delegation_contract_values(&delegation_contract3, 75u64, 0u64);
    // sc_setup.check_delegation_contract_values(&delegation_contract4, 75u64, 0u64);

    sc_setup.add_liquidity(&second_user, 500u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // sc_setup.check_delegation_contract_values(&delegation_contract1, 175u64, 0u64);
    // sc_setup.check_delegation_contract_values(&delegation_contract2, 175u64, 0u64);
    // sc_setup.check_delegation_contract_values(&delegation_contract3, 175u64, 0u64);
    // There was a remaining balance during the delegation and was added to the last contract as others have cap
    // sc_setup.check_delegation_contract_values(&delegation_contract4, 275u64, 0u64);

    sc_setup.update_staking_contract_params(
        &sc_setup.owner_address.clone(),
        &delegation_contract2,
        1080,
        0,
        6,
        13_000u64,
    );

    sc_setup.add_liquidity(&third_user, 600u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // sc_setup.check_delegation_contract_values(&delegation_contract1, 275u64, 0u64);
    sc_setup.check_delegation_contract_values_denominated(
        &delegation_contract2,
        396644526461469443760u128,
    );

    // sc_setup.check_delegation_contract_values(&delegation_contract3, 275u64, 0u64);
    sc_setup.check_delegation_contract_values_denominated(
        &delegation_contract4,
        865205349705419487464u128,
    );

    sc_setup.update_staking_contract_params(
        &sc_setup.owner_address.clone(),
        &delegation_contract2,
        1080,
        0,
        3,
        8_000u64,
    );
    sc_setup.update_staking_contract_params(
        &sc_setup.owner_address.clone(),
        &delegation_contract3,
        1030,
        1100,
        3,
        9_000u64,
    );

    sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, 300u64);
    sc_setup.check_user_balance(&second_user, LS_TOKEN_ID, 500u64);
    sc_setup.check_user_balance(&third_user, LS_TOKEN_ID, 600u64);

    sc_setup.b_mock.set_block_epoch(10u64);
    sc_setup.claim_rewards(&sc_setup.owner_address.clone());

    sc_setup.check_user_egld_balance_denominated(
        sc_setup.sc_wrapper.address_ref(),
        3835616438356164381u128,
    );

    sc_setup.check_contract_rewards_storage_denominated(3835616438356164381u128);
}

#[test]
fn liquid_staking_multiple_withdraw_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let first_user = sc_setup.setup_new_user(100u64);
    let second_user = sc_setup.setup_new_user(100u64);
    let third_user = sc_setup.setup_new_user(100u64);

    sc_setup.add_liquidity(&first_user, 50u64);
    sc_setup.add_liquidity(&second_user, 40u64);
    sc_setup.add_liquidity(&third_user, 40u64);
    sc_setup.b_mock.set_block_round(14000u64);
    sc_setup.check_contract_storage(130, 130, 0, 0, 130, 0);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.b_mock.set_block_epoch(50u64);
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 20u64);
    sc_setup.check_user_nft_balance_denominated(&first_user, UNSTAKE_TOKEN_ID, 1, exp18(20), None);
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 20u64);
    sc_setup.check_user_nft_balance_denominated(&first_user, UNSTAKE_TOKEN_ID, 1, exp18(40), None);
    sc_setup.remove_liquidity(&second_user, LS_TOKEN_ID, 20u64);
    sc_setup.remove_liquidity(&third_user, LS_TOKEN_ID, 20u64);

    sc_setup.check_contract_storage(50, 50, 0, 0, 0, 80);

    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.b_mock.set_block_epoch(60u64);

    sc_setup.withdraw_pending(&sc_setup.owner_address.clone(), &delegation_contract);

    sc_setup.check_contract_storage(50, 50, 0, 80, 0, 0);

    sc_setup.withdraw(&first_user, UNSTAKE_TOKEN_ID, 1, exp18(20));
    sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, 10u64);
    sc_setup.check_user_egld_balance(&first_user, 70);
    sc_setup.check_user_balance(&second_user, LS_TOKEN_ID, 20u64);
    sc_setup.check_user_egld_balance(&second_user, 60);
    sc_setup.check_user_balance(&third_user, LS_TOKEN_ID, 20u64);
    sc_setup.check_user_egld_balance(&third_user, 60);
    sc_setup.check_contract_storage(50, 50, 0, 60, 0, 0); // 20 + 20 (second_user + third_user)
}

#[test]
fn full_flow_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 0, 0, 0, 0, 0);

    let first_user = sc_setup.setup_new_user(50u64);
    let second_user = sc_setup.setup_new_user(20u64);
    let third_user = sc_setup.setup_new_user(20u64);

    sc_setup.check_user_egld_balance_denominated(&delegation_contract, 0);

    sc_setup.add_liquidity(&first_user, 50u64);
    sc_setup.add_liquidity(&second_user, 20u64);
    sc_setup.add_liquidity(&third_user, 20u64);

    sc_setup.b_mock.set_block_round(14000u64);

    sc_setup.check_user_egld_balance(&sc_setup.sc_wrapper.address_ref(), 90);

    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    sc_setup.check_user_egld_balance(&delegation_contract, 90);
    sc_setup.check_user_egld_balance(&sc_setup.sc_wrapper.address_ref(), 0);

    sc_setup.b_mock.set_block_epoch(50u64);
    sc_setup.claim_rewards(&sc_setup.owner_address.clone());

    let pending_rewards = sc_setup.get_pending_rewards();

    // From the 90 EGLD the mock SC send rewards to the liquid staking contract
    sc_setup.check_user_egld_balance_denominated(&delegation_contract, 88767123287671232877u128);

    sc_setup.b_mock.set_egld_balance(
        &delegation_contract,
        &(sc_setup.b_mock.get_egld_balance(&delegation_contract)
            + num_bigint::BigUint::from(pending_rewards)),
    );

    sc_setup.check_user_egld_balance(&delegation_contract, 90);

    // The liquid staking contract should have received the rewards
    sc_setup.check_user_egld_balance_denominated(
        &sc_setup.sc_wrapper.address_ref(),
        1232876712328767123u128,
    );

    // Due to the fees, less rewards are sent back to the delegation contract
    sc_setup.delegate_rewards(&sc_setup.owner_address.clone());

    // The liquid staking contract should have delegated the rewards to the delegation contract
    sc_setup.check_user_egld_balance_denominated(&sc_setup.sc_wrapper.address_ref(), 0);

    // Rewards are sent back to the delegation contract - the protocol fee is deducted
    sc_setup.check_user_egld_balance_denominated(&delegation_contract, 91183561643835616439u128);

    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 25u64);

    sc_setup.check_user_nft_balance_denominated(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        num_bigint::BigUint::from(25328767123287671233u128),
        Some(&UnstakeTokenAttributes::new(50, 60)),
    );
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 25u64);
    sc_setup.check_user_nft_balance_denominated(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        num_bigint::BigUint::from(50657534246575342466u128),
        Some(&UnstakeTokenAttributes::new(50, 60)),
    );
    sc_setup.remove_liquidity(&second_user, LS_TOKEN_ID, 20u64);
    sc_setup.check_user_nft_balance_denominated(
        &second_user,
        UNSTAKE_TOKEN_ID,
        1,
        num_bigint::BigUint::from(20263013698630136986u128),
        Some(&UnstakeTokenAttributes::new(50, 60)),
    );
    sc_setup.remove_liquidity(&third_user, LS_TOKEN_ID, 20u64);
    sc_setup.check_user_nft_balance_denominated(
        &third_user,
        UNSTAKE_TOKEN_ID,
        1,
        num_bigint::BigUint::from(20263013698630136987u128),
        Some(&UnstakeTokenAttributes::new(50, 60)),
    );

    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    sc_setup.b_mock.set_block_epoch(60u64);

    sc_setup.check_user_egld_balance_denominated(&sc_setup.sc_wrapper.address_ref(), 0);

    sc_setup.withdraw_pending(&sc_setup.owner_address.clone(), &delegation_contract);
    // The unstaked EGLD is sent back to the main liquid staking contract
    sc_setup.check_user_egld_balance_denominated(
        &sc_setup.sc_wrapper.address_ref(),
        91183561643835616439u128,
    );

    sc_setup.check_delegation_contract_values(&delegation_contract, 0u64, 0u64);

    sc_setup.check_total_withdrawn_egld_denominated(91183561643835616439u128);

    sc_setup.check_user_balance(&sc_setup.sc_wrapper.address_ref(), LS_TOKEN_ID, 0u64);

    sc_setup.check_user_egld_balance_denominated(
        &sc_setup.sc_wrapper.address_ref(),
        91183561643835616439u128,
    );
    sc_setup.withdraw(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        num_bigint::BigUint::from(50657534246575342466u128),
    );
    sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, 0u64);
    sc_setup.check_user_egld_balance_denominated(&first_user, 50657534246575342466u128);

    sc_setup.withdraw(
        &second_user,
        UNSTAKE_TOKEN_ID,
        1,
        num_bigint::BigUint::from(20263013698630136986u128),
    );
    sc_setup.check_user_balance(&second_user, LS_TOKEN_ID, 0u64);
    sc_setup.check_user_egld_balance_denominated(&second_user, 20263013698630136986u128);

    sc_setup.withdraw(
        &third_user,
        UNSTAKE_TOKEN_ID,
        1,
        num_bigint::BigUint::from(20263013698630136987u128),
    );
    sc_setup.check_user_balance(&third_user, LS_TOKEN_ID, 0u64);
    sc_setup.check_user_egld_balance_denominated(&third_user, 20263013698630136987u128);

    // The main delegation contract should have 0 EGLD left as the initial deposit (or a small amount due to rounding)
    sc_setup.check_user_egld_balance_denominated(&sc_setup.sc_wrapper.address_ref(), 0);
}

#[test]
fn claim_rewards_multiple_times_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let first_user = sc_setup.setup_new_user(100u64);

    sc_setup.add_liquidity(&first_user, 100u64);
    sc_setup.b_mock.set_block_round(14000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.b_mock.set_block_epoch(50u64);
    sc_setup.claim_rewards(&sc_setup.owner_address.clone());
    sc_setup.delegate_rewards(&sc_setup.owner_address.clone());
    let pending_rewards = sc_setup.get_pending_rewards();
    assert_eq!(pending_rewards, 0, "pending_rewards should be 0");
    sc_setup.b_mock.set_block_epoch(100u64);
    sc_setup.claim_rewards(&sc_setup.owner_address.clone());
    let pending_rewards = sc_setup.get_pending_rewards();
    assert_eq!(
        pending_rewards, 1387877650591105273u128,
        "pending_rewards should be 1387877650591105273"
    );
    sc_setup.delegate_rewards(&sc_setup.owner_address.clone());
}

#[test]
fn add_liquidity_no_valid_delegation_contract_error_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    let first_user = sc_setup.setup_new_user(100u64);

    sc_setup.add_liquidity(&first_user, 100u64);
    sc_setup.b_mock.set_block_round(14000u64);
    sc_setup.delegate_pending_error(
        &sc_setup.owner_address.clone(),
        OptionalValue::None,
        ERROR_NO_DELEGATION_CONTRACTS,
    );
}

#[test]
fn add_liquidity_no_available_delegation_contract_error_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);
    sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1000, 0, 0);
    let first_user = sc_setup.setup_new_user(100u64);

    sc_setup.add_liquidity(&first_user, 100u64);
    sc_setup.b_mock.set_block_round(14000u64);
    sc_setup.delegate_pending_error(
        &sc_setup.owner_address.clone(),
        OptionalValue::None,
        ERROR_BAD_DELEGATION_ADDRESS,
    );
}

#[test]
fn delegate_rewards_not_enough_egld_error_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let first_user = sc_setup.setup_new_user(100u64);

    sc_setup.add_liquidity(&first_user, 100u64);
    sc_setup.b_mock.set_block_round(14000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.b_mock.set_block_epoch(1u64);
    sc_setup.claim_rewards(&sc_setup.owner_address.clone());
    sc_setup.delegate_rewards_error(&sc_setup.owner_address.clone(), ERROR_INSUFFICIENT_REWARDS);
}

#[test]
fn delegate_rewards_inactive_state_error_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let first_user = sc_setup.setup_new_user(100u64);

    sc_setup.add_liquidity(&first_user, 100u64);
    sc_setup.b_mock.set_block_round(14000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.b_mock.set_block_epoch(1u64);
    sc_setup.claim_rewards(&sc_setup.owner_address.clone());
    sc_setup.set_inactive_state(&sc_setup.owner_address.clone());
    sc_setup.delegate_rewards_error(&sc_setup.owner_address.clone(), ERROR_NOT_ACTIVE);
}

// #[test]
// fn delegate_rewards_not_finished_claim_status_error_test() {
//     let _ = DebugApi::dummy();
//     let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

//     sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

//     let first_user = sc_setup.setup_new_user(100u64);

//     sc_setup.delegate_rewards_error(&first_user, ERROR_RECOMPUTE_RESERVES);
// }
