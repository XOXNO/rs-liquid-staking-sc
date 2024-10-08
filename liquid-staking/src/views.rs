use crate::{structs::ClaimStatusType, StorageCache};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]

pub trait ViewsModule:
    crate::storage::StorageModule
    + crate::config::ConfigModule
    + crate::liquidity_pool::LiquidityPoolModule
{
    #[view(canExecutePendingTasks)]
    fn can_execute_pending_tasks(&self) -> bool {
        // let block_round = self.blockchain().get_block_round();
        // let rounds_per_epoch = self.rounds_per_epoch().get();
        // let minimum_rounds = self.minimum_rounds().get();
        // let current_epoch = self.blockchain().get_block_epoch();
        // let change_epoch_round = (current_epoch + 1) * rounds_per_epoch;
        // change_epoch_round - block_round <= minimum_rounds
        true
    }

    #[view(getLsValueForPosition)]
    fn get_ls_value_for_position(&self, ls_token_amount: BigUint) -> BigUint {
        let storage_cache = StorageCache::new(self);
        self.get_egld_amount(&ls_token_amount, &storage_cache)
    }

    #[view(getEgldPositionValue)]
    fn get_egld_position_value(&self, egld_amount: BigUint) -> BigUint {
        let storage_cache = StorageCache::new(self);
        self.get_ls_amount(&egld_amount, &storage_cache)
    }

    #[view(getExchangeRate)]
    fn get_exchange_rate(&self) -> BigUint {
        let storage_cache = StorageCache::new(self);
        const INITIAL_EXCHANGE_RATE: u64 = 1_000_000_000_000_000_000;

        // The initial exchange rate between XOXNO and LXOXNO is fixed to one
        if storage_cache.ls_token_supply.clone() == BigUint::zero() {
            return BigUint::from(INITIAL_EXCHANGE_RATE);
        }

        storage_cache.virtual_egld_reserve.clone() * BigUint::from(INITIAL_EXCHANGE_RATE)
            / storage_cache.ls_token_supply.clone()
    }

    #[view(getDelegationStatus)]
    fn get_delegation_status(&self) -> ClaimStatusType {
        let claim_status = self.delegation_claim_status().get();
        claim_status.status
    }

    #[view(getDelegationContractStakedAmount)]
    fn get_delegation_contract_staked_amount(&self, delegation_address: ManagedAddress) -> BigUint {
        let delegation_contract_data = self.delegation_contract_data(&delegation_address).get();
        delegation_contract_data.total_staked_from_ls_contract
    }

    #[view(getDelegationContractUnstakedAmount)]
    fn get_delegation_contract_unstaked_amount(
        &self,
        delegation_address: ManagedAddress,
    ) -> BigUint {
        let delegation_contract_data = self.delegation_contract_data(&delegation_address).get();
        delegation_contract_data.total_unstaked_from_ls_contract
    }
}
