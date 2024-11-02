mod contract_interactions;
mod contract_setup;
mod utils;

use contract_setup::*;
use liquid_staking::{
    errors::{
        ERROR_BAD_PAYMENT_AMOUNT, ERROR_BAD_PAYMENT_TOKEN, ERROR_INSUFFICIENT_UNBONDED_AMOUNT,
        ERROR_NOT_ACTIVE, ERROR_UNSTAKE_PERIOD_NOT_PASSED,
    },
    structs::UnstakeTokenAttributes,
};
use multiversx_sc::imports::OptionalValue;
use utils::*;

use multiversx_sc_scenario::DebugApi;

pub static BAD_TOKEN_ID: &[u8] = b"BAD-123456";

#[test]
fn liquid_staking_unbond_success_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let user = sc_setup.setup_new_user(100u64);

    // Add liquidity
    sc_setup.add_liquidity(&user, 100u64);

    sc_setup.b_mock.set_block_round(14000u64);
    // Delegate pending tokens
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // Set block epoch to 50
    sc_setup.b_mock.set_block_epoch(50u64);

    // Remove liquidity
    sc_setup.remove_liquidity(&user, LS_TOKEN_ID, 90u64);

    // Check user's NFT balance after removing liquidity
    sc_setup.check_user_nft_balance_denominated(
        &user,
        UNSTAKE_TOKEN_ID,
        1,
        exp18(90),
        Some(&UnstakeTokenAttributes::new(50, 60)),
    );

    sc_setup.check_contract_storage(10, 10, 0, 0, 0, 90);

    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    sc_setup.check_contract_storage(10, 10, 0, 0, 0, 0);

    sc_setup.check_delegation_contract_values(&delegation_contract, 10, 90);

    // // Set block epoch to 60 (after unstake deadline)
    sc_setup.b_mock.set_block_epoch(61u64);

    sc_setup.withdraw_pending(&sc_setup.owner_address.clone(), &delegation_contract);

    sc_setup.check_delegation_contract_values(&delegation_contract, 10, 0);

    // // Check contract storage after withdraw unbond
    sc_setup.check_contract_storage(10, 10, 0, 90, 0, 0);

    // // Perform unbond operation
    sc_setup.withdraw(&user, UNSTAKE_TOKEN_ID, 1, exp18(90));

    // // Check user's EGLD balance after unbond
    sc_setup.check_user_egld_balance(&user, 90u64);

    // // Check user's NFT balance after unbond (should be 0)
    sc_setup.check_user_nft_balance_denominated(&user, UNSTAKE_TOKEN_ID, 1, exp18(0), None);

    // // Check contract storage after unbond
    sc_setup.check_contract_storage(10, 10, 0, 0, 0, 0);
}

#[test]
fn liquid_staking_unbond_error_epoch_too_soon_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let user = sc_setup.setup_new_user(100u64);

    // Add liquidity
    sc_setup.add_liquidity(&user, 100u64);

    sc_setup.b_mock.set_block_round(14000u64);
    // Delegate pending tokens
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // Set block epoch to 50
    sc_setup.b_mock.set_block_epoch(50u64);

    // Remove liquidity
    sc_setup.remove_liquidity(&user, LS_TOKEN_ID, 90u64);

    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // // Set block epoch to 60 (after unstake deadline)
    sc_setup.b_mock.set_block_epoch(55u64);

    sc_setup.withdraw_pending(&sc_setup.owner_address.clone(), &delegation_contract);

    // // Perform unbond operation
    sc_setup.withdraw_error(
        &user,
        UNSTAKE_TOKEN_ID,
        1,
        exp18(90),
        ERROR_UNSTAKE_PERIOD_NOT_PASSED,
    );
}

#[test]
fn liquid_staking_unbond_error_epoch_no_withdraw_pending_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let user = sc_setup.setup_new_user(100u64);

    // Add liquidity
    sc_setup.add_liquidity(&user, 100u64);

    sc_setup.b_mock.set_block_round(14000u64);
    // Delegate pending tokens
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // Set block epoch to 50
    sc_setup.b_mock.set_block_epoch(50u64);

    // Remove liquidity
    sc_setup.remove_liquidity(&user, LS_TOKEN_ID, 90u64);

    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // // Set block epoch to 60 (after unstake deadline)
    sc_setup.b_mock.set_block_epoch(60u64);

    // // Perform unbond operation
    sc_setup.withdraw_error(
        &user,
        UNSTAKE_TOKEN_ID,
        1,
        exp18(90),
        ERROR_INSUFFICIENT_UNBONDED_AMOUNT,
    );
}

#[test]
fn liquid_staking_unbond_error_not_active_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let user = sc_setup.setup_new_user(100u64);

    // Add liquidity
    sc_setup.add_liquidity(&user, 100u64);

    sc_setup.b_mock.set_block_round(14000u64);
    // Delegate pending tokens
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // Set block epoch to 50
    sc_setup.b_mock.set_block_epoch(50u64);

    // Remove liquidity
    sc_setup.remove_liquidity(&user, LS_TOKEN_ID, 90u64);

    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // // Set block epoch to 60 (after unstake deadline)
    sc_setup.b_mock.set_block_epoch(60u64);

    sc_setup.withdraw_pending(&sc_setup.owner_address.clone(), &delegation_contract);

    sc_setup.set_inactive_state(&sc_setup.owner_address.clone());

    // // Perform unbond operation
    sc_setup.withdraw_error(&user, UNSTAKE_TOKEN_ID, 1, exp18(90), ERROR_NOT_ACTIVE);
}

#[test]
fn liquid_staking_unbond_error_not_amount_sent_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let user = sc_setup.setup_new_user(100u64);

    // Add liquidity
    sc_setup.add_liquidity(&user, 100u64);

    sc_setup.b_mock.set_block_round(14000u64);
    // Delegate pending tokens
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // Set block epoch to 50
    sc_setup.b_mock.set_block_epoch(50u64);

    // Remove liquidity
    sc_setup.remove_liquidity(&user, LS_TOKEN_ID, 90u64);

    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // // Set block epoch to 60 (after unstake deadline)
    sc_setup.b_mock.set_block_epoch(60u64);

    sc_setup.withdraw_pending(&sc_setup.owner_address.clone(), &delegation_contract);

    // // Perform unbond operation
    sc_setup.withdraw_error(
        &user,
        UNSTAKE_TOKEN_ID,
        1,
        exp18(0),
        ERROR_BAD_PAYMENT_AMOUNT,
    );
}

#[test]
fn liquid_staking_unbond_error_bad_token_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj, 400);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let user = sc_setup.setup_new_user(100u64);

    // Add liquidity
    sc_setup.add_liquidity(&user, 100u64);

    sc_setup.b_mock.set_block_round(14000u64);
    // Delegate pending tokens
    sc_setup.delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // Set block epoch to 50
    sc_setup.b_mock.set_block_epoch(50u64);

    // Remove liquidity
    sc_setup.remove_liquidity(&user, LS_TOKEN_ID, 90u64);

    sc_setup.un_delegate_pending(&sc_setup.owner_address.clone(), OptionalValue::None);

    // // Set block epoch to 60 (after unstake deadline)
    sc_setup.b_mock.set_block_epoch(60u64);

    sc_setup.withdraw_pending(&sc_setup.owner_address.clone(), &delegation_contract);

    // // Perform unbond operation with bad token

    sc_setup
        .b_mock
        .set_esdt_balance(&user, BAD_TOKEN_ID, &exp18(100));
    sc_setup.withdraw_error(&user, BAD_TOKEN_ID, 0, exp18(100), ERROR_BAD_PAYMENT_TOKEN);
}
