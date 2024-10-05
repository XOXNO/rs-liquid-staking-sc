mod contract_interactions;
mod contract_setup;
mod utils;

use contract_setup::*;

use utils::*;

use liquid_staking::structs::UnstakeTokenAttributes;
use multiversx_sc_scenario::DebugApi;

#[test]
fn init_test() {
    let _ = LiquidStakingContractSetup::new(liquid_staking::contract_obj);
}

#[test]
fn liquid_staking_add_liquidity_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj);

    sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let first_user = sc_setup.setup_new_user(100u64);

    // Action: First user adds 100 EGLD as liquidity to the contract
    sc_setup.add_liquidity(&first_user, 100u64);

    // Check the contract storage to ensure the values are as expected after liquidity addition
    // 100: total_egld_staked (increased by 100 due to liquidity addition)
    // 100: total_ls_token_supply (increased by 100 due to liquidity addition)
    // 0: total_egld_withdrawn
    // 0: total_pending_egld
    // 100: total_pending_ls_token (increased by 100 due to liquidity addition)
    // 0: total_unstaked_egld
    sc_setup.check_contract_storage(100, 100, 0, 0, 100, 0);

    // Check the first user's balance of LS tokens after liquidity addition
    // Expected balance: 100 LS tokens (equal to the amount of EGLD added as liquidity)
    sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, 100u64);
}

#[test]
fn liquid_staking_add_liquidity_instant_test() {
    // Create a dummy instance of DebugApi for testing purposes
    let _ = DebugApi::dummy();

    // Set up a new instance of the LiquidStakingContractSetup with the liquid_staking contract object
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj);

    // Deploy the staking contract with the specified parameters
    // owner_address: the address of the contract owner
    // 1000: the initial total_egld_staked value
    // 1000: the initial total_ls_token_supply value
    // 1500: the initial total_egld_withdrawn value
    // 0: the initial total_pending_egld value
    // 0: the initial total_pending_ls_token value
    sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    // Set up two new users, each with an initial balance of 100 EGLD
    let first_user = sc_setup.setup_new_user(100u64);
    let second_user = sc_setup.setup_new_user(100u64);

    // First user adds 100 EGLD as liquidity to the contract
    sc_setup.add_liquidity(&first_user, 100u64);

    // Set the current block epoch to 50
    sc_setup.b_mock.set_block_epoch(50u64);

    // Check the contract storage to ensure the values are as expected
    // 100: total_egld_staked
    // 100: total_ls_token_supply
    // 0: total_egld_withdrawn
    // 0: total_pending_egld
    // 100: total_pending_ls_token
    // 0: total_unstaked_egld
    sc_setup.check_contract_storage(100, 100, 0, 0, 100, 0);

    sc_setup.b_mock.set_block_round(14000u64);

    // Delegate the pending EGLD for the first user
    sc_setup.delegate_pending(&first_user);

    // Check the contract storage again to ensure the values are updated
    // 100: total_egld_staked
    // 100: total_ls_token_supply
    // 0: total_egld_withdrawn
    // 0: total_pending_egld
    // 0: total_pending_ls_token (decreased by 100 after delegation)
    // 0: total_unstaked_egld
    sc_setup.check_contract_storage(100, 100, 0, 0, 0, 0);

    // First user removes 90 LS tokens as liquidity from the contract
    sc_setup.remove_liquidity(&first_user, LS_TOKEN_ID, 90u64);

    // Check the contract storage to ensure the values are updated
    // 100: total_egld_staked
    // 100: total_ls_token_supply
    // 0: total_egld_withdrawn
    // 0: total_pending_egld
    // 0: total_pending_ls_token
    // 90: total_unstaked_egld (increased by 90 after liquidity removal)
    sc_setup.check_contract_storage(100, 100, 0, 0, 0, 90);

    // Check the first user's balance of LS tokens
    // Expected balance: 10 LS tokens (100 - 90 removed)
    sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, 10u64);


    // Check the first user's balance of unstake tokens (NFTs)
    // Expected balance: 1 unstake token with the specified attributes
    // 50: unstake_epoch
    // 90 * 10^18: unstake_amount
    // 60: unstake_deadline_epoch
    sc_setup.check_user_nft_balance_denominated(
        &first_user,
        UNSTAKE_TOKEN_ID,
        1,
        exp18(90),
        Some(&UnstakeTokenAttributes::new(
            50,
            60,
        )),
    );

    // Check the first user's EGLD balance
    // Expected balance: 0 EGLD (all EGLD was added as liquidity)
    sc_setup.check_user_egld_balance(&first_user, 0u64);

    // Second user adds 100 EGLD as liquidity to the contract
    sc_setup.add_liquidity(&second_user, 100u64);

    // Check the contract storage to ensure the values are updated
    // 110: total_egld_staked (increased by 10 due to the 90 EGLD in pending unstake not being executed)
    // 110: total_ls_token_supply (increased by 10 due to the 90 EGLD in pending unstake not being executed)
    // 0: total_egld_withdrawn
    // 90: total_pending_egld (the 90 EGLD from the first user's liquidity removal)
    // 10: total_pending_ls_token (increased by 10 due to the second user's liquidity addition)
    // 0: total_unstaked_egld
    sc_setup.check_contract_storage(110, 110, 0, 90, 10, 0);
}

#[test]
fn liquid_staking_add_liquidity_exp17_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj);

    sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let first_user = sc_setup.setup_new_user(10u64);
    let second_user = sc_setup.setup_new_user(1u64);

    // Action: First user adds 1 EGLD as liquidity to the contract
    sc_setup.add_liquidity(&first_user, 1u64);

    // Action: Second user adds 5 * 10^17 (0.5 EGLD) as liquidity to the contract using exp17
    // This simulates adding liquidity with decimal values
    sc_setup.add_liquidity_exp17(&second_user, 5u64);

    // Check the pending EGLD in the contract
    // Expected value: 15 * 10^17 (1.5 EGLD)
    // This is because the first user added 1 EGLD and the second user added 0.5 EGLD
    sc_setup.check_pending_egld_exp17(15u64);

    // Check the first user's balance of LS tokens
    // Expected balance: 1 LS token
    sc_setup.check_user_balance(&first_user, LS_TOKEN_ID, 1u64);

    // Check the second user's balance of LS tokens using exp17
    // Expected balance: 5 * 10^17 (0.5 LS tokens)
    // This is because the second user added 0.5 EGLD as liquidity
    sc_setup.check_user_balance_exp17(&second_user, LS_TOKEN_ID, 5u64);
}
