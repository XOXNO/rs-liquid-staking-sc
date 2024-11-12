multiversx_sc::imports!();
use crate::{StorageCache, ERROR_MIGRATION_NOT_ALLOWED, ERROR_MIGRATION_SC_NOT_SET};


#[multiversx_sc::module]
pub trait MigrateModule:
    crate::config::ConfigModule
    + crate::events::EventsModule
    + crate::storage::StorageModule
    + crate::utils::generic::UtilsModule
    + crate::score::ScoreModule
    + crate::selection::SelectionModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[endpoint(migrate)]
    fn migrate(&self, virtual_egld_amount: &BigUint, original_caller: ManagedAddress) {
        let mut storage_cache = StorageCache::new(self);
        let migration_sc_address = self.migration_sc_address().get();

        // Check that the migration SC address is set
        require!(!migration_sc_address.is_zero(), ERROR_MIGRATION_SC_NOT_SET);

        let caller = self.blockchain().get_caller();

        // Check that the caller is the migration SC
        require!(caller == migration_sc_address, ERROR_MIGRATION_NOT_ALLOWED);

        // Double check that the caller is a smart contract
        require!(
            self.blockchain().is_smart_contract(&caller),
            ERROR_MIGRATION_NOT_ALLOWED
        );

        let ls_amount = self.pool_add_liquidity(virtual_egld_amount, &mut storage_cache);
        let user_payment = self.mint_ls_token(ls_amount);

        // Emit the add liquidity event
        self.emit_add_liquidity_event(&storage_cache, virtual_egld_amount, Some(original_caller));
        // Send the final amount to the user
        self.tx().to(&caller).esdt(user_payment).transfer();
    }

    #[payable("EGLD")]
    #[endpoint(migratePending)]
    fn migrate_pending(&self) {
        let mut storage_cache = StorageCache::new(self);
        let migration_sc_address = self.migration_sc_address().get();

        // Check that the migration SC address is set
        require!(!migration_sc_address.is_zero(), ERROR_MIGRATION_SC_NOT_SET);

        let caller = self.blockchain().get_caller();

        // Check that the caller is the migration SC
        require!(caller == migration_sc_address, ERROR_MIGRATION_NOT_ALLOWED);

        // Double check that the caller is a smart contract
        require!(
            self.blockchain().is_smart_contract(&caller),
            ERROR_MIGRATION_NOT_ALLOWED
        );

        let amount = self.call_value().egld_value();

        storage_cache.pending_egld += amount.clone_value();

        self.emit_general_liquidity_event(&storage_cache);
    }

    #[payable("EGLD")]
    #[endpoint(addRewards)]
    fn add_rewards(&self) {
        let mut storage_cache = StorageCache::new(self);

        let amount = self.call_value().egld_value();

        storage_cache.rewards_reserve += amount.clone_value();

        self.emit_general_liquidity_event(&storage_cache);
    }

    #[only_owner]
    #[endpoint(setMigrationScAddress)]
    fn add_migration_sc_address(&self, address: &ManagedAddress) {
        // Double check that the caller is a smart contract
        require!(
            self.blockchain().is_smart_contract(address),
            ERROR_MIGRATION_NOT_ALLOWED
        );
        self.migration_sc_address().set(address);
    }

    #[view(getMigrationScAddress)]
    #[storage_mapper("migrationScAddress")]
    fn migration_sc_address(&self) -> SingleValueMapper<ManagedAddress>;
}
