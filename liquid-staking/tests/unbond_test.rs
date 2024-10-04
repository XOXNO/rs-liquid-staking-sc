mod contract_interactions;
mod contract_setup;
mod utils;

use contract_setup::*;
use liquid_staking::structs::UnstakeTokenAttributes;
use utils::*;

use multiversx_sc_scenario::DebugApi;

#[test]
fn liquid_staking_unbond_success_test() {
    let _ = DebugApi::dummy();
    let mut sc_setup = LiquidStakingContractSetup::new(liquid_staking::contract_obj);

    let delegation_contract =
        sc_setup.deploy_staking_contract(&sc_setup.owner_address.clone(), 1000, 1000, 1500, 0, 0);

    let user = sc_setup.setup_new_user(100u64);

    // Add liquidity
    sc_setup.add_liquidity(&user, 100u64);

    // Delegate pending tokens
    sc_setup.delegate_pending(&user);

    // Set block epoch to 50
    sc_setup.b_mock.set_block_epoch(50u64);

    // Remove liquidity
    sc_setup.remove_liquidity(&user, LS_TOKEN_ID, 90u64);

    // Check user's NFT balance after removing liquidity
    sc_setup.check_user_nft_balance_denominated(
        &user,
        UNSTAKE_TOKEN_ID,
        1,
        1,
        Some(&UnstakeTokenAttributes::new(
            50,
            to_managed_biguint(exp18(90)),
            60,
        )),
    );

    sc_setup.check_contract_storage(100, 100, 0, 0, 0, 90);

    sc_setup.un_delegate_pending(&user);

    sc_setup.check_contract_storage(10, 10, 0, 0, 0, 0);

    sc_setup.check_delegation_contract_values(&delegation_contract, 10, 90);

    // // Set block epoch to 60 (after unstake deadline)
    sc_setup.b_mock.set_block_epoch(61u64);

    sc_setup.withdraw_pending(&user, &delegation_contract);

    sc_setup.check_delegation_contract_values(&delegation_contract, 10, 0);

    // // Check contract storage after withdraw unbond
    sc_setup.check_contract_storage(10, 10, 0, 90, 0, 0);

    // // Perform unbond operation
    sc_setup.withdraw(&user, UNSTAKE_TOKEN_ID, 1);

    // // Check user's EGLD balance after unbond
    sc_setup.check_user_egld_balance(&user, 90u64);

    // // Check user's NFT balance after unbond (should be 0)
    sc_setup.check_user_nft_balance_denominated(&user, UNSTAKE_TOKEN_ID, 1, 0, None);

    // // Check contract storage after unbond
    sc_setup.check_contract_storage(10, 10, 0, 0, 0, 0);
}
