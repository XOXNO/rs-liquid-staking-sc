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
use multiversx_sc::{
    imports::OptionalValue,
    types::{TestAddress, TestTokenIdentifier},
};
use utils::*;

use multiversx_sc_scenario::DebugApi;

pub static BAD_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("BAD-123456");

#[test]
fn liquid_staking_unbond_success_test() {
    DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(400);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&OWNER_ADDRESS.to_address(), 1000, 1000, 1500, 0, 0);

    let user = sc_setup.setup_new_user(TestAddress::new("user"), 100u64);

    // Add liquidity
    sc_setup.add_liquidity(&user, exp18(100u64), OptionalValue::None);

    sc_setup.b_mock.current_block().block_round(14000u64);
    // Delegate pending tokens
    sc_setup.delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    // Set block epoch to 50
    sc_setup.b_mock.current_block().block_epoch(50u64);

    // Remove liquidity
    sc_setup.remove_liquidity(&user, LS_TOKEN_ID, exp18(90u64));

    // Check user's NFT balance after removing liquidity
    sc_setup.check_user_nft_balance_denominated(
        &user,
        UNSTAKE_TOKEN_ID,
        1,
        exp18(90),
        Some(UnstakeTokenAttributes::new(50, 60)),
    );

    sc_setup.check_contract_storage(10, 10, 0, 0, 0, 90);

    sc_setup.un_delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    sc_setup.check_contract_storage(10, 10, 0, 0, 0, 0);

    sc_setup.check_delegation_contract_values(&delegation_contract, exp18(10), exp18(90));

    // // Set block epoch to 60 (after unstake deadline)
    sc_setup.b_mock.current_block().block_epoch(61u64);

    sc_setup.withdraw_pending(&OWNER_ADDRESS.to_address(), &delegation_contract);

    sc_setup.check_delegation_contract_values(&delegation_contract, exp18(10), exp18(0));

    // // Check contract storage after withdraw unbond
    sc_setup.check_contract_storage(10, 10, 0, 90, 0, 0);

    // // Perform unbond operation
    sc_setup.withdraw(&user, UNSTAKE_TOKEN_ID, 1, exp18(90));

    // // Check user's EGLD balance after unbond
    sc_setup.check_user_egld_balance(&user, exp18(90u64));

    // // Check contract storage after unbond
    sc_setup.check_contract_storage(10, 10, 0, 0, 0, 0);
}

#[test]
fn liquid_staking_unbond_error_epoch_too_soon_test() {
    DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(400);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&OWNER_ADDRESS.to_address(), 1000, 1000, 1500, 0, 0);

    let user = sc_setup.setup_new_user(TestAddress::new("user"), 100u64);

    // Add liquidity
    sc_setup.add_liquidity(&user, exp18(100u64), OptionalValue::None);

    sc_setup.b_mock.current_block().block_round(14000u64);
    // Delegate pending tokens
    sc_setup.delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    // Set block epoch to 50
    sc_setup.b_mock.current_block().block_epoch(50u64);

    // Remove liquidity
    sc_setup.remove_liquidity(&user, LS_TOKEN_ID, exp18(90u64));

    sc_setup.un_delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    // // Set block epoch to 60 (after unstake deadline)
    sc_setup.b_mock.current_block().block_epoch(55u64);

    sc_setup.withdraw_pending(&OWNER_ADDRESS.to_address(), &delegation_contract);

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
    DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(400);

    sc_setup.deploy_staking_contract(&OWNER_ADDRESS.to_address(), 1000, 1000, 1500, 0, 0);

    let user = sc_setup.setup_new_user(TestAddress::new("user"), 100u64);

    // Add liquidity
    sc_setup.add_liquidity(&user, exp18(100u64), OptionalValue::None);

    sc_setup.b_mock.current_block().block_round(14000u64);
    // Delegate pending tokens
    sc_setup.delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    // Set block epoch to 50
    sc_setup.b_mock.current_block().block_epoch(50u64);

    // Remove liquidity
    sc_setup.remove_liquidity(&user, LS_TOKEN_ID, exp18(90u64));

    sc_setup.un_delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    // // Set block epoch to 60 (after unstake deadline)
    sc_setup.b_mock.current_block().block_epoch(60u64);

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
fn liquid_staking_unbond_partial_withdraw_pending_test() {
    DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(400);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&OWNER_ADDRESS.to_address(), 1000, 1000, 1500, 0, 0);

    let user = sc_setup.setup_new_user(TestAddress::new("user"), 40u64);
    let second_user = sc_setup.setup_new_user(TestAddress::new("second_user"), 100u64);

    // Add liquidity
    sc_setup.add_liquidity(&user, exp18(40u64), OptionalValue::None);
    // Add liquidity
    sc_setup.add_liquidity(&second_user, exp18(100u64), OptionalValue::None);

    sc_setup.b_mock.current_block().block_round(14000u64);
    // Delegate pending tokens
    sc_setup.delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    // Set block epoch to 50
    sc_setup.b_mock.current_block().block_epoch(50u64);

    // Remove liquidity
    sc_setup.remove_liquidity(&user, LS_TOKEN_ID, exp18(40u64));

    sc_setup.un_delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    // Set block epoch to 51
    sc_setup.b_mock.current_block().block_epoch(51u64);

    // Remove liquidity
    sc_setup.remove_liquidity(&second_user, LS_TOKEN_ID, exp18(90u64));

    // // Set block epoch to 60 (after unstake deadline)
    sc_setup.b_mock.current_block().block_epoch(60u64);

    sc_setup.withdraw_pending(&OWNER_ADDRESS.to_address(), &delegation_contract);

    // // Set block epoch to 60 (after unstake deadline)
    sc_setup.b_mock.current_block().block_epoch(61u64);

    sc_setup.withdraw(&second_user, UNSTAKE_TOKEN_ID, 2, exp18(90));

    sc_setup.check_user_nft_balance_denominated(
        &second_user,
        UNSTAKE_TOKEN_ID,
        2,
        exp18(50),
        Some(UnstakeTokenAttributes::new(51, 61)),
    );

    // Perform unbond operation
    sc_setup.withdraw_error(
        &user,
        UNSTAKE_TOKEN_ID,
        1,
        exp18(40),
        ERROR_INSUFFICIENT_UNBONDED_AMOUNT,
    );
}

#[test]
fn liquid_staking_unbond_error_not_active_test() {
    DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(400);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&OWNER_ADDRESS.to_address(), 1000, 1000, 1500, 0, 0);

    let user = sc_setup.setup_new_user(TestAddress::new("user"), 100u64);

    // Add liquidity
    sc_setup.add_liquidity(&user, exp18(100u64), OptionalValue::None);

    sc_setup.b_mock.current_block().block_round(14000u64);
    // Delegate pending tokens
    sc_setup.delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    // Set block epoch to 50
    sc_setup.b_mock.current_block().block_epoch(50u64);

    // Remove liquidity
    sc_setup.remove_liquidity(&user, LS_TOKEN_ID, exp18(90u64));

    sc_setup.un_delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    // // Set block epoch to 60 (after unstake deadline)
    sc_setup.b_mock.current_block().block_epoch(60u64);

    sc_setup.withdraw_pending(&OWNER_ADDRESS.to_address(), &delegation_contract);

    sc_setup.set_inactive_state(&OWNER_ADDRESS.to_address());

    // // Perform unbond operation
    sc_setup.withdraw_error(&user, UNSTAKE_TOKEN_ID, 1, exp18(90), ERROR_NOT_ACTIVE);
}

#[test]
fn liquid_staking_unbond_error_not_amount_sent_test() {
    DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(400);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&OWNER_ADDRESS.to_address(), 1000, 1000, 1500, 0, 0);

    let user = sc_setup.setup_new_user(TestAddress::new("user"), 100u64);

    // Add liquidity
    sc_setup.add_liquidity(&user, exp18(100u64), OptionalValue::None);

    sc_setup.b_mock.current_block().block_round(14000u64);
    // Delegate pending tokens
    sc_setup.delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    // Set block epoch to 50
    sc_setup.b_mock.current_block().block_epoch(50u64);

    // Remove liquidity
    sc_setup.remove_liquidity(&user, LS_TOKEN_ID, exp18(90u64));

    sc_setup.un_delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    // // Set block epoch to 60 (after unstake deadline)
    sc_setup.b_mock.current_block().block_epoch(60u64);

    sc_setup.withdraw_pending(&OWNER_ADDRESS.to_address(), &delegation_contract);

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
    DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(400);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&OWNER_ADDRESS.to_address(), 1000, 1000, 1500, 0, 0);

    let user = sc_setup.setup_new_user(TestAddress::new("user"), 100u64);

    // Add liquidity
    sc_setup.add_liquidity(&user, exp18(100u64), OptionalValue::None);

    sc_setup.b_mock.current_block().block_round(14000u64);
    // Delegate pending tokens
    sc_setup.delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    // Set block epoch to 50
    sc_setup.b_mock.current_block().block_epoch(50u64);

    // Remove liquidity
    sc_setup.remove_liquidity(&user, LS_TOKEN_ID, exp18(90u64));

    sc_setup.un_delegate_pending(&OWNER_ADDRESS.to_address(), OptionalValue::None);

    // // Set block epoch to 60 (after unstake deadline)
    sc_setup.b_mock.current_block().block_epoch(60u64);

    sc_setup.withdraw_pending(&OWNER_ADDRESS.to_address(), &delegation_contract);

    // Perform unbond operation with bad token

    sc_setup
        .b_mock
        .set_esdt_balance(&user, BAD_TOKEN_ID.as_bytes(), exp18(100));
    sc_setup.withdraw_error(&user, BAD_TOKEN_ID, 0, exp18(100), ERROR_BAD_PAYMENT_TOKEN);
}
