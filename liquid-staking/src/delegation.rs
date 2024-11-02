use crate::{
    errors::{
        ERROR_ALREADY_WHITELISTED, ERROR_DELEGATION_CAP, ERROR_NOT_WHITELISTED,
        ERROR_ONLY_DELEGATION_ADMIN,
    },
    structs::DelegationContractInfo,
    ERROR_MAX_DELEGATION_ADDRESSES,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait DelegationModule:
    crate::config::ConfigModule
    + crate::storage::StorageModule
    + crate::utils::UtilsModule
    + crate::events::EventsModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[endpoint(whitelistDelegationContract)]
    fn whitelist_delegation_contract(
        &self,
        contract_address: ManagedAddress,
        admin_address: ManagedAddress,
        total_staked: BigUint,
        delegation_contract_cap: BigUint,
        nr_nodes: u64,
        apy: u64,
    ) {
        require!(
            self.delegation_addresses_list().len() <= self.max_delegation_addresses().get(),
            ERROR_MAX_DELEGATION_ADDRESSES
        );

        self.is_manager(&self.blockchain().get_caller(), true);

        require!(
            self.delegation_contract_data(&contract_address).is_empty(),
            ERROR_ALREADY_WHITELISTED
        );

        require!(
            delegation_contract_cap >= total_staked || delegation_contract_cap == BigUint::zero(),
            ERROR_DELEGATION_CAP
        );

        let contract_data = DelegationContractInfo {
            admin_address,
            total_staked,
            delegation_contract_cap,
            nr_nodes,
            apy,
            total_staked_from_ls_contract: BigUint::zero(),
            total_unstaked_from_ls_contract: BigUint::zero(),
            eligible: true,
        };

        self.delegation_contract_data(&contract_address)
            .set(contract_data);
        self.add_delegation_address_in_list(contract_address);
    }

    #[endpoint(changeDelegationContractAdmin)]
    fn change_delegation_contract_admin(
        &self,
        contract_address: ManagedAddress,
        admin_address: ManagedAddress,
    ) {
        let delegation_address_mapper = self.delegation_contract_data(&contract_address);
        require!(!delegation_address_mapper.is_empty(), ERROR_NOT_WHITELISTED);
        self.is_manager(&self.blockchain().get_caller(), true);
        delegation_address_mapper.update(|contract_data| {
            contract_data.admin_address = admin_address;
        });
    }

    #[endpoint(changeDelegationContractParams)]
    fn change_delegation_contract_params(
        &self,
        contract_address: ManagedAddress,
        total_staked: BigUint,
        delegation_contract_cap: BigUint,
        nr_nodes: u64,
        apy: u64,
        is_eligible: bool,
    ) {
        let caller = self.blockchain().get_caller();
        let delegation_address_mapper = self.delegation_contract_data(&contract_address);
        let old_contract_data = delegation_address_mapper.get();

        require!(!delegation_address_mapper.is_empty(), ERROR_NOT_WHITELISTED);

        require!(
            old_contract_data.admin_address == caller || self.is_manager(&caller, false),
            ERROR_ONLY_DELEGATION_ADMIN
        );

        require!(
            delegation_contract_cap >= total_staked || delegation_contract_cap == BigUint::zero(),
            ERROR_DELEGATION_CAP
        );

        delegation_address_mapper.update(|contract_data| {
            contract_data.total_staked = total_staked;
            contract_data.delegation_contract_cap = delegation_contract_cap;
            contract_data.nr_nodes = nr_nodes;
            contract_data.apy = apy;
            contract_data.eligible = is_eligible;
        });
    }

    fn add_delegation_address_in_list(&self, contract_address: ManagedAddress) {
        let mut delegation_addresses_mapper = self.delegation_addresses_list();

        delegation_addresses_mapper.insert(contract_address);
    }

    fn remove_delegation_address_from_list(&self, contract_address: &ManagedAddress) {
        self.delegation_addresses_list()
            .swap_remove(contract_address);
    }

    fn move_delegation_contract_to_back(&self, delegation_contract: &ManagedAddress) {
        self.remove_delegation_address_from_list(delegation_contract);

        self.delegation_addresses_list()
            .insert(delegation_contract.clone());
    }
}
