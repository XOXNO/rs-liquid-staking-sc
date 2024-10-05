mod contract_interactions;
mod contract_setup;
mod utils;

use contract_setup::*;

use multiversx_sc_scenario::DebugApi;
use utils::exp18;

#[test]
fn init_test() {
    let _ = LiquidStakingContractSetup::new(liquid_staking::contract_obj);
}

#[test]
fn liquid_staking_claim_rewards_and_withdraw_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let first_user = sc_setup.setup_new_user(100u64);

    sc_setup.add_liquidity(&first_user, 100u64);
    sc_setup.check_delegation_contract_values(&delegation_contract, 0u64, 0u64);
    sc_setup.check_contract_storage(100, 100, 0, 0, 100, 0);

    sc_setup.b_mock.set_block_round(14000u64);
    sc_setup.delegate_pending(&first_user);

    sc_setup.check_delegation_contract_values(&delegation_contract, 100u64, 0u64);
    sc_setup.check_contract_storage(100, 100, 0, 0, 0, 0);

    sc_setup.b_mock.set_block_epoch(50u64);

    sc_setup.claim_rewards(&first_user);

    sc_setup.check_contract_rewards_storage_denominated(1369863013698630136);

    sc_setup.delegate_rewards(&first_user);
    return;
    sc_setup.check_contract_rewards_storage_denominated(0);

    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 90u64);
    sc_setup.check_pending_ls_for_unstake(90);

    sc_setup.un_delegate_pending(&first_user);
    sc_setup.check_pending_ls_for_unstake(0);
    sc_setup.check_delegation_contract_unstaked_value_denominated(
        &delegation_contract,
        91232876712328767122u128,
    );

    sc_setup.b_mock.set_block_epoch(60u64);

    sc_setup.withdraw_pending(&first_user, &delegation_contract);

    sc_setup.withdraw(&first_user, UNSTAKE_TOKEN_ID, 1, exp18(70));

    sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, 10u64);
    sc_setup.check_user_egld_balance_denominated(&first_user, 91232876712328767122u128);
}

#[test]
fn liquid_staking_multiple_operations() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj);

    let delegation_contract1 = sc_setup.deploy_staking_contract(
        &sc_setup.owner_address.clone(),
        1000,
        1000,
        1100,
        3,
        10_000u64,
    );

    let delegation_contract2 = sc_setup.deploy_staking_contract(
        &sc_setup.owner_address.clone(),
        1000,
        1000,
        1100,
        3,
        13_000u64,
    );

    let delegation_contract3 = sc_setup.deploy_staking_contract(
        &sc_setup.owner_address.clone(),
        1000,
        1000,
        1100,
        3,
        11_000u64,
    );

    let delegation_contract4 = sc_setup.deploy_staking_contract(
        &sc_setup.owner_address.clone(),
        1000,
        1000,
        0,
        3,
        11_000u64,
    );

    let manager = sc_setup.setup_new_user(100u64);
    let first_user = sc_setup.setup_new_user(1000u64);
    let second_user = sc_setup.setup_new_user(1000u64);
    let third_user = sc_setup.setup_new_user(1000u64);
    sc_setup.add_liquidity(&first_user, 100u64);
    
    sc_setup.b_mock.set_block_round(14000u64);
    sc_setup.delegate_pending(&manager);

    sc_setup.check_delegation_contract_values(&delegation_contract1, 25u64, 0u64);
    sc_setup.check_delegation_contract_values(&delegation_contract2, 25u64, 0u64);
    sc_setup.check_delegation_contract_values(&delegation_contract3, 25u64, 0u64);
    sc_setup.check_delegation_contract_values(&delegation_contract4, 25u64, 0u64);

    sc_setup.add_liquidity(&first_user, 200u64);
    sc_setup.delegate_pending(&manager);
    sc_setup.check_delegation_contract_values(&delegation_contract1, 75u64, 0u64);
    sc_setup.check_delegation_contract_values(&delegation_contract2, 75u64, 0u64);
    sc_setup.check_delegation_contract_values(&delegation_contract3, 75u64, 0u64);
    sc_setup.check_delegation_contract_values(&delegation_contract4, 75u64, 0u64);

    sc_setup.add_liquidity(&second_user, 500u64);
    sc_setup.delegate_pending(&manager);

    sc_setup.check_delegation_contract_values(&delegation_contract1, 175u64, 0u64);
    sc_setup.check_delegation_contract_values(&delegation_contract2, 175u64, 0u64);
    sc_setup.check_delegation_contract_values(&delegation_contract3, 175u64, 0u64);
    // There was a remaining balance during the delegation and was added to the last contract as others have cap
    sc_setup.check_delegation_contract_values(&delegation_contract4, 275u64, 0u64);

    sc_setup.update_staking_contract_params(
        &sc_setup.owner_address.clone(),
        &delegation_contract2,
        1080,
        0,
        6,
        13_000u64,
    );

    sc_setup.add_liquidity(&third_user, 600u64);
    sc_setup.delegate_pending(&manager);
    sc_setup.check_delegation_contract_values(&delegation_contract1, 275u64, 0u64);
    sc_setup.check_delegation_contract_values_denominated(
        &delegation_contract2,
        443750000000000000000u128,
    );
    sc_setup.check_delegation_contract_values(&delegation_contract3, 275u64, 0u64);
    sc_setup.check_delegation_contract_values_denominated(
        &delegation_contract4,
        406250000000000000000u128,
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
    sc_setup.claim_rewards(&manager);

    sc_setup.check_user_egld_balance_denominated(
        sc_setup.sc_wrapper.address_ref(),
        3835616438356164382u128,
    );

    sc_setup.check_contract_rewards_storage_denominated(3835616438356164382u128);
}

#[test]
fn liquid_staking_multiple_withdraw_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj);

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
    sc_setup.delegate_pending(&first_user);
    sc_setup.b_mock.set_block_epoch(50u64);
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 20u64);
    sc_setup.check_user_nft_balance_denominated(&first_user, UNSTAKE_TOKEN_ID, 1, exp18(20), None);
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 20u64);
    sc_setup.check_user_nft_balance_denominated(&first_user, UNSTAKE_TOKEN_ID, 1, exp18(40), None);
    sc_setup.remove_liquidity(&second_user, LS_TOKEN_ID, 20u64);
    sc_setup.remove_liquidity(&third_user, LS_TOKEN_ID, 20u64);

    sc_setup.check_contract_storage(130, 130, 0, 0, 0, 80);

    // return;
    sc_setup.un_delegate_pending(&first_user);
    sc_setup.b_mock.set_block_epoch(60u64);

    sc_setup.withdraw_pending(&first_user, &delegation_contract);

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
