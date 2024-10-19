mod contract_interactions;
mod contract_setup;
mod utils;

use contract_setup::*;
use utils::*;

use liquid_staking::{
    errors::ERROR_INSUFFICIENT_UNSTAKE_PENDING_EGLD, structs::UnstakeTokenAttributes,
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
    sc_setup.check_contract_storage(100, 100, 0, 0, 100, 0);

    // Remove liquidity of 90 tokens from the user
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 90u64);
    // Check the contract storage to ensure the liquidity is removed correctly
    sc_setup.check_contract_storage(10, 10, 0, 0, 10, 0);

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
    sc_setup.check_contract_storage(100, 100, 0, 0, 100, 0);

    // Delegate the pending tokens
    sc_setup.b_mock.set_block_round(14000u64);
    sc_setup.delegate_pending(&sc_setup.owner_address.clone());

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
    sc_setup.check_contract_storage(100, 100, 0, 0, 100, 0);

    // Delegate the pending tokens
    sc_setup.delegate_pending(&sc_setup.owner_address.clone());
    // Check the contract storage to ensure the pending tokens are delegated
    sc_setup.check_contract_storage(100, 100, 0, 0, 0, 0);

    // Set up a second user with an initial balance of 2 tokens
    let second_user = sc_setup.setup_new_user(2u64);

    // Add liquidity of 1.5 tokens (with 17 decimals) from the second user to the contract
    sc_setup.add_liquidity_exp17(&second_user, 15u64);
    // Check the pending EGLD balance to ensure it is updated correctly
    sc_setup.check_pending_egld_exp17(15u64);

    // Remove liquidity of 2 tokens from the first user
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 2u64);

    // Check the pending EGLD balance to ensure it remains unchanged
    sc_setup.check_pending_egld_exp17(15u64);

    // Check the pending LS tokens for unstake to ensure they are updated correctly
    sc_setup.check_pending_ls_for_unstake(2);

    // Check the user's balance of LS tokens to ensure they have 98 tokens remaining
    sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, 98u64);

    // Check the user's NFT balance to ensure they received an NFT representing their unstaked tokens
    sc_setup.check_user_nft_balance_denominated(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        exp18(2),
        Some(&UnstakeTokenAttributes::new(50, 60)),
    );
    // Check the user's EGLD balance to ensure they didn't receive any EGLD back
    sc_setup.check_user_egld_balance(&first_user, 0u64);
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
    sc_setup.delegate_pending(&sc_setup.owner_address.clone());

    // Check the values of the first delegation contract
    sc_setup.check_delegation_contract_values(&delegation_contract1, 50u64, 0u64);
    // Check the values of the second delegation contract
    sc_setup.check_delegation_contract_values(&delegation_contract2, 50u64, 0u64);

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
    sc_setup.delegate_pending(&sc_setup.owner_address.clone());

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
