use crate::StorageCache;

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
        // TODO: Implement once new hooks are available in the VM
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
        let ls_token_supply = self.ls_token_supply().get();
        let virtual_egld_reserve = self.virtual_egld_reserve().get();
        // 1 EGLD = 10^18 atomic units
        const INITIAL_EXCHANGE_RATE: u64 = 1_000_000_000_000_000_000;

        // When no liquidity, 1 LS token = 1 EGLD
        if ls_token_supply == BigUint::zero() {
            return BigUint::from(INITIAL_EXCHANGE_RATE);
        }

        // Exchange Rate = (Total EGLD in protocol / Total LS Supply) * PRECISION
        // This gives us how many atomic units of EGLD you get for 1 LS token
        // Example: If rate = 1.1 * 10^18, it means 1 LS token = 1.1 EGLD
        &virtual_egld_reserve * &BigUint::from(INITIAL_EXCHANGE_RATE) / &ls_token_supply
    }

    #[view(getDelegationContractStakedAmount)]
    fn get_delegation_contract_staked_amount(&self, delegation_address: &ManagedAddress) -> BigUint {
        let delegation_contract_data = self.delegation_contract_data(delegation_address).get();
        delegation_contract_data.total_staked_from_ls_contract
    }

    #[view(getDelegationContractUnstakedAmount)]
    fn get_delegation_contract_unstaked_amount(
        &self,
        delegation_address: &ManagedAddress,
    ) -> BigUint {
        let delegation_contract_data = self.delegation_contract_data(delegation_address).get();
        delegation_contract_data.total_unstaked_from_ls_contract
    }
}
