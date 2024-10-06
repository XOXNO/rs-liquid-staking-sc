use crate::{
    errors::{
        ERROR_ALREADY_WHITELISTED, ERROR_DELEGATION_CAP, ERROR_NOT_WHITELISTED,
        ERROR_ONLY_DELEGATION_ADMIN,
    },
    structs::DelegationContractData,
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
    #[only_owner]
    #[endpoint(updateMaxDelegationAddressesNumber)]
    fn update_max_delegation_addresses_number(&self, number: usize) {
        self.max_delegation_addresses().set(number);
    }

    #[only_owner]
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
            "Maximum number of delegation addresses reached"
        );

        require!(
            self.delegation_contract_data(&contract_address).is_empty(),
            ERROR_ALREADY_WHITELISTED
        );

        require!(
            delegation_contract_cap >= total_staked || delegation_contract_cap == BigUint::zero(),
            ERROR_DELEGATION_CAP
        );

        let contract_data = DelegationContractData {
            admin_address,
            total_staked,
            delegation_contract_cap,
            nr_nodes,
            apy,
            total_staked_from_ls_contract: BigUint::zero(),
            total_unstaked_from_ls_contract: BigUint::zero(),
        };

        self.delegation_contract_data(&contract_address)
            .set(contract_data);
        self.add_and_order_delegation_address_in_list(contract_address, apy);
    }

    #[only_owner]
    #[endpoint(changeDelegationContractAdmin)]
    fn change_delegation_contract_admin(
        &self,
        contract_address: ManagedAddress,
        admin_address: ManagedAddress,
    ) {
        let delegation_address_mapper = self.delegation_contract_data(&contract_address);
        require!(!delegation_address_mapper.is_empty(), ERROR_NOT_WHITELISTED);

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
    ) {
        let caller = self.blockchain().get_caller();
        let delegation_address_mapper = self.delegation_contract_data(&contract_address);
        let old_contract_data = delegation_address_mapper.get();
        require!(!delegation_address_mapper.is_empty(), ERROR_NOT_WHITELISTED);
        require!(
            old_contract_data.admin_address == caller,
            ERROR_ONLY_DELEGATION_ADMIN
        );
        require!(
            delegation_contract_cap >= total_staked || delegation_contract_cap == BigUint::zero(),
            ERROR_DELEGATION_CAP
        );

        if old_contract_data.apy != apy {
            self.remove_delegation_address_from_list(&contract_address);
            self.add_and_order_delegation_address_in_list(contract_address, apy)
        }

        delegation_address_mapper.update(|contract_data| {
            contract_data.total_staked = total_staked;
            contract_data.delegation_contract_cap = delegation_contract_cap;
            contract_data.nr_nodes = nr_nodes;
            contract_data.apy = apy;
        });
    }

    fn add_and_order_delegation_address_in_list(&self, contract_address: ManagedAddress, apy: u64) {
        let mut delegation_addresses_mapper = self.delegation_addresses_list();
        if delegation_addresses_mapper.is_empty() {
            delegation_addresses_mapper.push_front(contract_address);
        } else {
            let mut check_if_added = false;
            for delegation_address_element in delegation_addresses_mapper.iter() {
                let node_id = delegation_address_element.get_node_id();
                let delegation_address = delegation_address_element.into_value();
                let delegation_contract_data =
                    self.delegation_contract_data(&delegation_address).get();
                if apy >= delegation_contract_data.apy {
                    self.delegation_addresses_list()
                        .push_before_node_id(node_id, contract_address.clone());
                    check_if_added = true;
                    break;
                }
            }
            if !check_if_added {
                delegation_addresses_mapper.push_back(contract_address);
            }
        }
    }

    fn remove_delegation_address_from_list(&self, contract_address: &ManagedAddress) {
        for delegation_address_element in self.delegation_addresses_list().iter() {
            let node_id = delegation_address_element.get_node_id();
            let delegation_address = delegation_address_element.into_value();
            if contract_address == &delegation_address {
                self.delegation_addresses_list().remove_node_by_id(node_id);
                break;
            }
        }
    }

    fn move_delegation_contract_to_back(&self, delegation_contract: &ManagedAddress) {
        self.remove_delegation_address_from_list(&delegation_contract);
        self.delegation_addresses_list()
            .push_back(delegation_contract.clone());
    }
}
