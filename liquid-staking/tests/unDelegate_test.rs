mod contract_interactions;
mod contract_setup;
mod utils;
use contract_setup::*;
use multiversx_sc::imports::OptionalValue;
use rand::Rng;
use utils::*;

use liquid_staking::{
    errors::{ERROR_INSUFFICIENT_PENDING_EGLD, ERROR_INSUFFICIENT_UNSTAKE_PENDING_EGLD},
    structs::UnstakeTokenAttributes,
};
use multiversx_sc_scenario::DebugApi;

// Test: liquid_staking_remove_liquidity_instant_test
// Summary: This test verifies the instant removal of liquidity from the contract when the contract has enough available EGLD.
// It confirms that the user's LS token balance is reduced, their EGLD balance is increased by the correct amount,
// and the contract's storage is updated to reflect the removed liquidity.
#[test]
fn undelegate_can_fully_instant_redeem() {
    // Create a dummy debug API instance
    let _ = DebugApi::dummy();
    // Set up the liquid staking contract
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    // Deploy the staking contract with the specified parameters
    sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    // Set up a new user with an initial balance of 100 tokens
    let first_user = sc_setup.setup_new_user(100u64);

    // Add liquidity of 100 tokens from the user to the contract
    sc_setup.add_liquidity(&first_user, 100u64);
    // Check the contract storage to ensure the liquidity is added correctly
    sc_setup.check_contract_storage(101, 101, 0, 0, 100, 0);

    // Remove liquidity of 90 tokens from the user
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 90u64);
    // Check the contract storage to ensure the liquidity is removed correctly
    sc_setup.check_contract_storage(11, 11, 0, 0, 10, 0);

    // Check the user's balance of LS tokens to ensure they have 10 tokens remaining
    sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, 10u64);
    // Check the user's EGLD balance to ensure they received 90 EGLD back
    sc_setup.check_user_egld_balance(&first_user, 90u64);
}

// Test: liquid_staking_remove_liquidity_not_instant_test
// Summary: This test verifies the non-instant removal of liquidity from the contract when the contract does not have enough available EGLD.
// It confirms that the user receives an NFT representing their unstaked tokens with the correct attributes,
// their LS token balance is reduced, and the contract's storage is updated to reflect the pending unstake.
#[test]
fn undelegate_partially_instant_test() {
    // Create a dummy debug API instance
    let _ = DebugApi::dummy();
    // Set up the liquid staking contract
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    // Deploy the staking contract with the specified parameters
    sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    // Set up a new user with an initial balance of 100 tokens
    let first_user = sc_setup.setup_new_user(100u64);
    let second_user = sc_setup.setup_new_user(200u64);

    // Add liquidity of 100 tokens from the user to the contract
    sc_setup.add_liquidity(&first_user, 100u64);

    // Set the block epoch to 50
    sc_setup.b_mock.set_block_epoch(50u64);

    // Check the contract storage to ensure the liquidity is added correctly
    sc_setup.check_contract_storage(101, 101, 0, 0, 100, 0);

    // Delegate the pending tokens
    sc_setup.b_mock.set_block_round(14000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // Add liquidity of 90.5 tokens from the second user to the contract
    sc_setup.add_liquidity_exp17(&second_user, 905u64);

    // Remove liquidity of 90 tokens from the user
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 90u64);

    sc_setup.check_pending_egld_exp17(15u64);
    sc_setup.check_pending_ls_for_unstake(1);

    // Check the user's balance of LS tokens to ensure they have 10 tokens remaining
    sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, 10u64);

    // Check the user's NFT balance to ensure they received an NFT representing their unstaked tokens
    sc_setup.check_user_nft_balance_denominated(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        exp18(1),
        Some(&UnstakeTokenAttributes::new(50, 60)),
    );

    // Check the user's EGLD balance to ensure they received some instant EGLD back the maximum possible
    sc_setup.check_user_egld_balance(&first_user, 89);
}

#[test]
fn clean_old_unbond_epochs_test() {
    // Create a dummy debug API instance
    let _ = DebugApi::dummy();
    // Set up the liquid staking contract
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    // Deploy the staking contract with the specified parameters
    sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    // Set up a new user with an initial balance of 100 tokens
    let first_user = sc_setup.setup_new_user(100u64);
    let second_user = sc_setup.setup_new_user(200u64);

    // Add liquidity of 100 tokens from the user to the contract
    sc_setup.add_liquidity(&first_user, 100u64);

    // Set the block epoch to 50
    sc_setup.b_mock.set_block_epoch(50u64);

    // Delegate the pending tokens
    sc_setup.b_mock.set_block_round(14000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // Add liquidity of 90.5 tokens from the second user to the contract
    sc_setup.add_liquidity_exp17(&second_user, 905u64);

    // Remove liquidity of 90 tokens from the user
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 90u64);

    sc_setup.check_pending_egld_exp17(15u64);
    sc_setup.check_pending_ls_for_unstake(1);

    // Check the user's balance of LS tokens to ensure they have 10 tokens remaining
    sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, 10u64);

    // Check the user's NFT balance to ensure they received an NFT representing their unstaked tokens
    sc_setup.check_user_nft_balance_denominated(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        exp18(1),
        Some(&UnstakeTokenAttributes::new(50, 60)),
    );

    // Check the user's EGLD balance to ensure they received some instant EGLD back the maximum possible
    sc_setup.check_user_egld_balance(&first_user, 89);

    sc_setup.b_mock.set_block_epoch(51u64);

    sc_setup.remove_liquidity_exp17(&second_user, LS_TOKEN_ID, 905u64);
    // Check the user's NFT balance to ensure they received an NFT representing their unstaked tokens
    sc_setup.check_user_nft_balance_denominated(
        &second_user,
        UNSTAKE_TOKEN_ID,
        2,
        exp17(890),
        Some(&UnstakeTokenAttributes::new(51, 61)),
    );
}

// Test: liquid_staking_remove_liquidity_not_partially_instant_test
// Summary: This test verifies the removal of liquidity from the contract when the remaining amount is less than 1 EGLD.
// It confirms that the liquidity is removed correctly, the user receives an NFT representing their unstaked tokens with the correct attributes,
// their LS token balance is reduced, and the contract's storage is updated to reflect the pending unstake and pending EGLD balance.
#[test]
fn calculate_partial_undelegate_fallback_test() {
    // Create a dummy debug API instance
    let _ = DebugApi::dummy();
    // Set up the liquid staking contract
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    // Deploy the staking contract with the specified parameters
    sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    // Set up a new user with an initial balance of 100 tokens
    let first_user = sc_setup.setup_new_user(100u64);

    // Add liquidity of 100 tokens from the user to the contract
    sc_setup.add_liquidity(&first_user, 100u64);

    sc_setup.b_mock.set_block_round(14000u64);
    // Set the block epoch to 50
    sc_setup.b_mock.set_block_epoch(50u64);

    // Check the contract storage to ensure the liquidity is added correctly
    sc_setup.check_contract_storage(101, 101, 0, 0, 100, 0);

    // Delegate the pending tokens
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    // Check the contract storage to ensure the pending tokens are delegated
    sc_setup.check_contract_storage(101, 101, 0, 0, 0, 0);

    // Set up a second user with an initial balance of 2 tokens
    let second_user = sc_setup.setup_new_user(2u64);

    // Add liquidity of 1.5 tokens (with 17 decimals) from the second user to the contract
    sc_setup.add_liquidity_exp17(&second_user, 15u64);
    // Check the pending EGLD balance to ensure it is updated correctly
    sc_setup.check_pending_egld_exp17(15u64);

    // Remove liquidity of 2 tokens from the first user
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 2u64);

    // Check the pending EGLD balance to ensure it remains unchanged
    sc_setup.check_pending_egld_exp17(10u64);

    // Check the pending LS tokens for unstake to ensure they are updated correctly
    sc_setup.check_pending_ls_for_unstake_exp17(15);

    // Check the user's balance of LS tokens to ensure they have 98 tokens remaining
    sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, 98u64);

    // Check the user's NFT balance to ensure they received an NFT representing their unstaked tokens
    sc_setup.check_user_nft_balance_denominated(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        exp17(15),
        Some(&UnstakeTokenAttributes::new(50, 60)),
    );
    // Check the user's EGLD balance to ensure they didn't receive any EGLD back
    sc_setup.check_user_egld_balance_exp17(&first_user, 5);
}

// Test: liquid_staking_remove_liquidity_partially_instant_test
// Summary: This test verifies the partial instant removal of liquidity from the contract when the contract has enough available EGLD for a portion of the unstake.
// It confirms that the user receives a portion of their unstaked tokens instantly, the remaining as an NFT with the correct attributes,
// their LS token balance is reduced, their EGLD balance is increased by the correct amount, and the contract's storage is updated to reflect the removed liquidity and pending unstake.
#[test]
fn undelegate_can_fully_pending_redeem() {
    // Create a dummy debug API instance
    let _ = DebugApi::dummy();
    // Set up the liquid staking contract
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    // Deploy the first staking contract with the specified parameters
    let delegation_contract1 =
        sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);
    // Deploy the second staking contract with the specified parameters
    let delegation_contract2 =
        sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    // Set up the first user with an initial balance of 100 tokens
    let first_user = sc_setup.setup_new_user(100u64);
    // Set up the second user with an initial balance of 30 tokens
    let second_user = sc_setup.setup_new_user(30u64);

    // Add liquidity of 100 tokens from the first user to the contract
    sc_setup.add_liquidity(&first_user, 100u64);

    sc_setup.b_mock.set_block_round(14000u64);
    // Set the block epoch to 50
    sc_setup.b_mock.set_block_epoch(50u64);
    // Delegate the pending tokens
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // Check the values of the first delegation contract
    sc_setup.check_delegation_contract_values(&delegation_contract1, 51u64, 0u64);
    // Check the values of the second delegation contract
    sc_setup.check_delegation_contract_values(&delegation_contract2, 51u64, 0u64);

    // Add liquidity of 30 tokens from the second user to the contract
    sc_setup.add_liquidity(&second_user, 30u64);
    // Remove liquidity of 90 tokens from the first user
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 90u64);

    // Check the user's balance of LS tokens to ensure they have 10 tokens remaining
    sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, 10u64);
    // Check the user's NFT balance to ensure they received an NFT representing their unstaked tokens
    sc_setup.check_user_nft_balance_denominated(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        exp18(60),
        Some(&UnstakeTokenAttributes::new(50, 60)),
    );
    // Check the user's EGLD balance to ensure they received 30 EGLD back instantly
    sc_setup.check_user_egld_balance(&first_user, 30u64);
}

// #[test]
// fn liquid_staking_un_delegate_pending_rounds_error_test() {
//     let _ = DebugApi::dummy();
//     let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

//     let delegation_contract =
//         sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

//     let first_user = sc_setup.setup_new_user(100u64);

//     sc_setup.add_liquidity(&first_user, 100u64);

//     sc_setup.check_delegation_contract_values(&delegation_contract, 0u64, 0u64);
//     sc_setup.check_contract_storage(100, 100, 0, 0, 100, 0);

//     sc_setup.b_mock.set_block_round(14000u64);
//     sc_setup.delegate_pending(&first_user);

//     sc_setup.check_delegation_contract_values(&delegation_contract, 100u64, 0u64);
//     sc_setup.check_contract_storage(100, 100, 0, 0, 0, 0);

//     sc_setup.b_mock.set_block_epoch(50u64);

//     sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 90u64);

//     sc_setup.b_mock.set_block_round(140u64);
//     sc_setup.un_delegate_pending_error(&first_user, ERROR_MINIMUM_ROUNDS_NOT_PASSED);
// }

#[test]
fn undelegate_small_amount_error_test() {
    // Create a dummy debug API instance
    let _ = DebugApi::dummy();
    // Set up the liquid staking contract
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    // Deploy the staking contract with the specified parameters
    sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    // Set up a new user with an initial balance of 100 tokens
    let first_user = sc_setup.setup_new_user(2u64);
    let second_user = sc_setup.setup_new_user(2u64);

    // Add liquidity of 100 tokens from the user to the contract
    sc_setup.add_liquidity(&first_user, 2u64);

    // Set the block epoch to 50
    sc_setup.b_mock.set_block_epoch(50u64);

    // Delegate the pending tokens
    sc_setup.b_mock.set_block_round(14000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // Add liquidity of 1.2 tokens from the second user to the contract
    sc_setup.add_liquidity_exp17(&second_user, 12u64);

    // Remove liquidity of 0.3 tokens from the user
    sc_setup.remove_liquidity_exp17_error(
        &first_user,
        LS_TOKEN_ID,
        3u64,
        ERROR_INSUFFICIENT_UNSTAKE_PENDING_EGLD,
    );
}

#[test]
fn liquid_staking_un_delegate_custom_amount_under_min_pending_error_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let first_user = sc_setup.setup_new_user(10u64);

    // Action: First user adds 3 EGLD as liquidity to the contract
    sc_setup.add_liquidity(&first_user, 3u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 3u64);

    sc_setup.un_delegate_pending_error(
        &sc_setup.owner_address.clone(),
        OptionalValue::Some(5),
        ERROR_INSUFFICIENT_PENDING_EGLD,
    );
}

#[test]
fn liquid_staking_un_delegate_custom_amount_pending_error_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let first_user = sc_setup.setup_new_user(10u64);

    // Action: First user adds 3 EGLD as liquidity to the contract
    sc_setup.add_liquidity(&first_user, 3u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 3u64);

    sc_setup.un_delegate_pending_error(
        &sc_setup.owner_address.clone(),
        OptionalValue::Some(40),
        ERROR_INSUFFICIENT_PENDING_EGLD,
    );
}

#[test]
fn liquid_staking_un_delegate_custom_amount_left_over_pending_error_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let first_user = sc_setup.setup_new_user(10u64);

    // Action: First user adds 3 EGLD as liquidity to the contract
    sc_setup.add_liquidity(&first_user, 3u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 3u64);
    sc_setup.un_delegate_pending_error(
        &sc_setup.owner_address.clone(),
        OptionalValue::Some(25),
        ERROR_INSUFFICIENT_PENDING_EGLD,
    );
}

#[test]
fn liquid_staking_un_delegate_custom_amount_full_pending_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let first_user = sc_setup.setup_new_user(10u64);

    // Action: First user adds 3 EGLD as liquidity to the   contract
    sc_setup.add_liquidity(&first_user, 3u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 3u64);
    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::Some(3));
}

#[test]
fn undelegate_remaining_amount_distribution_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    // Deploy providers with specific caps to trigger our edge case
    sc_setup.deploy_staking_contract(
        &sc_setup.owner_address.clone(),
        1000,
        1000,
        1200, // Higher cap
        4,
        8_000u64,
    );

    sc_setup.deploy_staking_contract(
        &sc_setup.owner_address.clone(),
        1000,
        1000,
        1050, // Lower cap
        5,
        9_000u64,
    );

    // Setup user and initial delegation
    let first_user = sc_setup.setup_new_user(100u64);
    sc_setup.add_liquidity(&first_user, 100u64);

    sc_setup.b_mock.set_block_round(14000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // Remove liquidity in a way that would leave less than MIN_EGLD in contract2
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 98u64);

    // This should trigger the remaining amount redistribution
    sc_setup.b_mock.set_block_epoch(50u64);
    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // Verify NFT was issued correctly
    sc_setup.check_user_nft_balance_denominated(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        exp18(98),
        Some(&UnstakeTokenAttributes::new(0, 10)),
    );
}

#[test]
fn undelegate_left_over_amount_condition_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    // Deploy three providers to ensure redistribution logic is properly tested
    let delegation_contract1 = sc_setup.deploy_staking_contract(
        &sc_setup.owner_address.clone(),
        1000,
        1000,
        1200,
        4,
        8_000u64,
    );

    let delegation_contract2 = sc_setup.deploy_staking_contract(
        &sc_setup.owner_address.clone(),
        1000,
        1000,
        1200,
        5,
        9_000u64,
    );

    let delegation_contract3 = sc_setup.deploy_staking_contract(
        &sc_setup.owner_address.clone(),
        1000,
        1000,
        1200,
        6,
        10_000u64,
    );

    // Setup user with initial amount that will divide unevenly
    let first_user = sc_setup.setup_new_user(153u64);
    sc_setup.add_liquidity(&first_user, 153u64);

    sc_setup.b_mock.set_block_round(14000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // Initial distribution should be roughly equal
    // sc_setup.check_delegation_contract_values_denominated(&delegation_contract1, 52007624458065480641u128);
    // sc_setup.check_delegation_contract_values_denominated(&delegation_contract2, 51992435479086324532u128);
    // sc_setup.check_delegation_contract_values_denominated(&delegation_contract3, 51992435479086324532u128);

    // Remove liquidity to trigger redistribution with left_over amounts
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 150u64);

    sc_setup.b_mock.set_block_epoch(50u64);
    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // Contract2 should be skipped due to left_over_amount condition
    // Remaining amount should be split between contract1 and contract3
    // sc_setup.check_delegation_contract_values_denominated(&delegation_contract1, 2100745480183515319u128);
    // sc_setup.check_delegation_contract_values_denominated(&delegation_contract2, 1992435479086324532u128);  // Skipped
    // sc_setup.check_delegation_contract_values_denominated(&delegation_contract3, 1906819040730160149u128);

    // Verify NFT was issued correctly
    sc_setup.check_user_nft_balance_denominated(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        exp18(150),
        Some(&UnstakeTokenAttributes::new(0, 10)),
    );
}

#[test]
fn undelegate_left_over_min_egld_condition_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    // Deploy with same APY differences
    let delegation_contract1 = sc_setup.deploy_staking_contract(
        &sc_setup.owner_address.clone(),
        1000,
        1000,
        1200,
        4,
        12_000u64,
    );

    let delegation_contract2 = sc_setup.deploy_staking_contract(
        &sc_setup.owner_address.clone(),
        1000,
        1000,
        1200,
        5,
        8_000u64,
    );

    let delegation_contract3 = sc_setup.deploy_staking_contract(
        &sc_setup.owner_address.clone(),
        1000,
        1000,
        1200,
        6,
        10_000u64,
    );

    // Add 60 EGLD
    let first_user = sc_setup.setup_new_user(60u64);
    sc_setup.add_liquidity(&first_user, 60u64);

    sc_setup.b_mock.set_block_round(14000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // Verify actual distribution values
    // sc_setup.check_delegation_contract_values_denominated(&delegation_contract1, 21011864155420436008u128);
    // sc_setup.check_delegation_contract_values_denominated(&delegation_contract2, 20994067922289781996u128);
    // sc_setup.check_delegation_contract_values_denominated(&delegation_contract3, 20994067922289781996u128);

    // Now we need to remove the right amount to leave contract2 with ~1.1 EGLD
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 57u64);

    sc_setup.b_mock.set_block_epoch(50u64);
    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // Let's verify the actual values after undelegation
    // sc_setup.check_delegation_contract_values_denominated(&delegation_contract1, 1979329908845093542u128);
    // sc_setup.check_delegation_contract_values_denominated(&delegation_contract2, 2026602168865124462u128);
    // sc_setup.check_delegation_contract_values_denominated(&delegation_contract3, 1994067922289781996u128);
}

#[test]
fn undelegate_remaining_amount_over_provider_limit_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    // Deploy 25 providers with varying APYs and caps
    let mut delegation_contracts = Vec::new();
    for i in 0..60 {
        let contract = sc_setup.deploy_staking_contract(
            &sc_setup.owner_address.clone(),
            1000,
            1000,
            0, // Different caps
            i as u64,
            8_000u64 + (i as u64 * 100), // Different APYs to get varied distribution
        );
        delegation_contracts.push(contract);
    }

    // Add significant liquidity that will exceed first 20 providers capacity
    let first_user = sc_setup.setup_new_user(200000u64);
    sc_setup.add_liquidity(&first_user, 20000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.add_liquidity(&first_user, 20000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    sc_setup.add_liquidity(&first_user, 20000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    sc_setup.add_liquidity(&first_user, 20000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    sc_setup.add_liquidity(&first_user, 20000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // Remove most liquidity to trigger large undelegation
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 50000u64);

    sc_setup.b_mock.set_block_epoch(50u64);
    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
}

#[test]
fn delegate_remaining_amount_over_provider_limit_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    // Deploy 25 providers with varying caps
    let mut delegation_contracts = Vec::new();
    for i in 0..60 {
        let contract = sc_setup.deploy_staking_contract(
            &sc_setup.owner_address.clone(),
            0,
            0,
            if i <= 50 { 100 } else { 10 }, // Different caps
            1,
            9_000u64,
        );
        delegation_contracts.push(contract);
    }

    // Add large amount of liquidity
    let first_user = sc_setup.setup_new_user(5000000u64);
    sc_setup.add_liquidity(&first_user, 2500000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
}

#[test]
fn full_un_delegate_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    // Deploy 60 providers with varying caps
    let mut delegation_contracts = Vec::new();
    for _ in 0..60 {
        let random_nodes = rand::thread_rng().gen_range(1..=100);
        let random_apy = rand::thread_rng().gen_range(500..=1000);
        let contract = sc_setup.deploy_staking_contract(
            &sc_setup.owner_address.clone(),
            0,
            0,
            0, // Different caps
            random_nodes,
            random_apy,
        );
        delegation_contracts.push(contract);
    }

    // Add large amount of liquidity
    let first_user = sc_setup.setup_new_user(60000u64);
    sc_setup.add_liquidity(&first_user, 20000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.add_liquidity(&first_user, 20000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.add_liquidity(&first_user, 20000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 60000u64);

    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    
    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    sc_setup.check_pending_ls_for_unstake_denominated(0);
}

#[test]
fn test_scoring_config_distribution() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    // Deploy 25 providers with varying caps
    let mut delegation_contracts = Vec::new();
    for _ in 0..60 {
        let random_nodes = rand::thread_rng().gen_range(1..=100);
        let random_apy = rand::thread_rng().gen_range(500..=1000);
        let contract = sc_setup.deploy_staking_contract(
            &sc_setup.owner_address.clone(),
            0,
            0,
            0, // Different caps
            random_nodes,
            random_apy,
        );
        delegation_contracts.push(contract);
    }

    // Add large amount of liquidity
    let first_user = sc_setup.setup_new_user(120000u64);
    sc_setup.add_liquidity(&first_user, 20000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.add_liquidity(&first_user, 20000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.add_liquidity(&first_user, 20000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    for _ in 0..10 {
        let random_nodes = rand::thread_rng().gen_range(1..=100);
        let random_apy = rand::thread_rng().gen_range(500..=1000);
        let contract = sc_setup.deploy_staking_contract(
            &sc_setup.owner_address.clone(),
            0,
            0,
            0, // Different caps
            random_nodes,
            random_apy,
        );
        delegation_contracts.push(contract);
    }
    sc_setup.add_liquidity(&first_user, 20000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.add_liquidity(&first_user, 20000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.add_liquidity(&first_user, 20000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.debug_providers();
}

#[test]
fn full_small_first_amount_un_delegate_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    // Deploy 25 providers with varying caps
    let mut delegation_contracts = Vec::new();
    for _ in 0..60 {
        let random_nodes = rand::thread_rng().gen_range(1..=100);
        let random_apy = rand::thread_rng().gen_range(500..=1000);
        let contract = sc_setup.deploy_staking_contract(
            &sc_setup.owner_address.clone(),
            0,
            0,
            0, // Different caps
            random_nodes,
            random_apy,
        );
        delegation_contracts.push(contract);
    }

    // Add large amount of liquidity
    let first_user = sc_setup.setup_new_user(20);
    sc_setup.add_liquidity_exp17(&first_user, 50);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.add_liquidity_exp17(&first_user, 10);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    sc_setup.remove_liquidity_exp17(&first_user, LS_TOKEN_ID, 17);
    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);


    sc_setup.remove_liquidity_exp17(&first_user, LS_TOKEN_ID, 10);
    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);


    sc_setup.remove_liquidity_exp17(&first_user, LS_TOKEN_ID, 10);
    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);
    sc_setup.check_pending_ls_for_unstake_denominated(0);
}

#[test]
fn full_over_2_first_amount_un_delegate_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    // Deploy 25 providers with varying caps
    let mut delegation_contracts = Vec::new();
    for _ in 0..60 {

        let random_nodes = rand::thread_rng().gen_range(1..=100);
        let random_apy = rand::thread_rng().gen_range(500..=1000);
        let contract = sc_setup.deploy_staking_contract(
            &sc_setup.owner_address.clone(),
            0,
            0,
            0, // Different caps
            random_nodes,
            random_apy,
        );
        delegation_contracts.push(contract);
    }

    // Add large amount of liquidity
    let first_user = sc_setup.setup_new_user(20);
    sc_setup.add_liquidity_exp17(&first_user, 21);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    sc_setup.remove_liquidity_exp17(&first_user, LS_TOKEN_ID, 21);
    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    sc_setup.check_user_nft_balance_denominated(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        exp17(21),
        Some(&UnstakeTokenAttributes::new(0, 10)),
    );

    sc_setup.check_pending_ls_for_unstake_denominated(0);
}
